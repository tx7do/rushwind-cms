//! The user service — the complete user face (list/count/get plus the
//! write paths: create/update/delete with the credential row, the
//! password and existence checks), mirroring the reference's
//! user_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::{user_repo};
use crate::service::context::{
    operator_of, operator_tenant_id,
};
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_roles, sys_user_credentials, sys_user_roles, sys_users,
};
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;

/// Insert a USERNAME credential for a user (bcrypt of the given
/// password; no AES layer — administrative sets arrive in plaintext).
pub(crate) async fn insert_credential(
    db: &sea_orm::DatabaseTransaction,
    tenant_id: i64,
    user_id: i64,
    username: &str,
    password: &str,
) -> Result<(), Status> {
    let hashed = store::crypto::hash_password(password).map_err(Status::internal)?;
    sys_user_credentials::ActiveModel {
        tenant_id: Set(Some(tenant_id)),
        user_id: Set(Some(user_id)),
        identity_type: Set(Some("USERNAME".to_string())),
        identifier: Set(Some(username.to_string())),
        credential_type: Set(Some("PASSWORD_HASH".to_string())),
        credential: Set(Some(hashed)),
        is_primary: Set(Some(true)),
        status: Set(Some("ENABLED".to_string())),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}

/// The reference's constants.DefaultAdminUserName.
const DEFAULT_ADMIN_USER_NAME: &str = "admin";

pub(crate) fn user_proto(r: sys_users::Model) -> identityv1::User {
    identityv1::User {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        username: r.username,
        nickname: r.nickname,
        realname: r.realname,
        email: r.email,
        mobile: r.mobile,
        telephone: r.telephone,
        avatar: r.avatar,
        address: r.address,
        region: r.region,
        description: r.description,
        gender: r.gender.as_deref().map(|_| 1),
        last_login_at: r.last_login_at.and_then(ts_to_proto),
        last_login_ip: r.last_login_ip,
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

// ── User writes ──────────────────────────────────────────────────────

pub struct UserService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::user_service_server::UserService for UserService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListUserResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_users::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListUserResponse {
            items: rows.into_iter().map(user_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountUserResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_users::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountUserResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetUserRequest>,
    ) -> Result<Response<identityv1::User>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_user_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = user_repo::users_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(user_proto(row)))
    }

    async fn create(
        &self,
        request: Request<identityv1::CreateUserRequest>,
    ) -> Result<Response<identityv1::User>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let password = req.password.unwrap_or_else(|| "12345678".to_string());
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = sys_users::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            username: Set(Some(data.username.clone().unwrap_or_default())),
            nickname: Set(data.nickname),
            realname: Set(data.realname),
            email: Set(data.email),
            mobile: Set(data.mobile),
            avatar: Set(data.avatar),
            status: Set(Some("NORMAL".to_string())),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        insert_credential(
            &txn,
            row.tenant_id.unwrap_or(0),
            row.id,
            &row.username.clone().unwrap_or_default(),
            &password,
        )
        .await?;
        // Role bindings (default: the tenant:user role, so created
        // users can actually sign in).
        let role_ids: Vec<i64> = if data.role_ids.is_empty() {
            sys_roles::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(sys_roles::Column::TenantId.eq(row.tenant_id.unwrap_or(0)))
                        .add(sys_roles::Column::Code.eq("tenant:user")),
                )
                .one(&txn)
                .await
                .map_err(db_status)?
                .map(|r| vec![r.id])
                .unwrap_or_default()
        } else {
            data.role_ids.iter().map(|v| *v as i64).collect()
        };
        for role_id in role_ids {
            sys_user_roles::ActiveModel {
                tenant_id: Set(row.tenant_id),
                user_id: Set(row.id),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(user_proto(row)))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdateUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_users::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("user"))?;
        let mut a: sys_users::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.nickname {
                a.nickname = Set(Some(v));
            }
            if let Some(v) = data.realname {
                a.realname = Set(Some(v));
            }
            if let Some(v) = data.email {
                a.email = Set(Some(v));
            }
            if let Some(v) = data.mobile {
                a.mobile = Set(Some(v));
            }
            if let Some(v) = data.avatar {
                a.avatar = Set(Some(v));
            }
            if let Some(v) = data.status {
                a.status = Set(Some(if v == 1 { "NORMAL" } else { "DISABLED" }.to_string()));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(password) = req.password.filter(|p| !p.is_empty()) {
            let cred = sys_user_credentials::Entity::find()
                .filter(
                    sea_orm::Condition::all()
                        .add(sys_user_credentials::Column::UserId.eq(row.id))
                        .add(sys_user_credentials::Column::IdentityType.eq("USERNAME")),
                )
                .one(&self.state.db)
                .await
                .map_err(db_status)?
                .ok_or_else(|| not_found("credential"))?;
            let hashed = store::crypto::hash_password(&password).map_err(Status::internal)?;
            let mut ca: sys_user_credentials::ActiveModel = cred.into();
            ca.credential = Set(Some(hashed));
            ca.updated_at = Set(Some(store::now()));
            ca.update(&self.state.db).await.map_err(db_status)?;
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeleteUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // The operator context is mandatory: an anonymous delete must
        // never pass (the reference errors before any lookup).
        let operator_id = operator_of(&request)?;
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        let Some(identityv1::delete_user_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let id = id as i64;

        let txn = self.state.db.begin().await.map_err(db_status)?;
        let target = sys_users::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("user"))?;
        // Tenant scope: the reference's viewer-predicated lookup hides
        // cross-tenant users from tenant operators.
        if caller_tenant_id > 0 && target.tenant_id.unwrap_or(0) != caller_tenant_id as i64 {
            return Err(not_found("user"));
        }

        // 禁止删除默认超级管理员：初始化时创建的平台级 admin（恒为 id=1，
        // 即便后续改名也由 id 兜底保护）。
        if target.id == 1
            || (target.username.as_deref() == Some(DEFAULT_ADMIN_USER_NAME)
                && target.tenant_id.unwrap_or(0) == 0)
        {
            return Err(bad("default admin cannot be deleted"));
        }
        // 禁止删除自己：误删自身账号将导致当前会话立即失去管理能力。
        if target.id == operator_id {
            return Err(bad("cannot delete yourself"));
        }

        // The reference cascades via its ent edges inside one tx; here
        // the relation rows go explicitly, same transaction.
        sys_users::Entity::delete_by_id(id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        sys_user_credentials::Entity::delete_many()
            .filter(sys_user_credentials::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn user_exists(
        &self,
        request: Request<identityv1::UserExistsRequest>,
    ) -> Result<Response<identityv1::UserExistsResponse>, Status> {
        let req = request.into_inner();
        let row = match req.query_by {
            Some(identityv1::user_exists_request::QueryBy::Id(id)) => {
                sys_users::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(identityv1::user_exists_request::QueryBy::Username(username)) => {
                sys_users::Entity::find()
                    .filter(sys_users::Column::Username.eq(username))
                    .one(&self.state.db)
                    .await
            }
            _ => return Err(bad("query_by required")),
        }
        .map_err(db_status)?;
        Ok(Response::new(identityv1::UserExistsResponse {
            exist: row.is_some(),
        }))
    }
}
