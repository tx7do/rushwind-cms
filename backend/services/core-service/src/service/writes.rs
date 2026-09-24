//! The identity write paths: user create/update/delete (with the
//! credential row), password edits, user/tenant existence checks, role
//! writes with role-permission bindings, tenant writes, the app-side
//! user profile face, and the translator passthrough.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::identity_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_role_permissions, sys_roles, sys_tenants, sys_user_credentials, sys_user_roles, sys_users,
};
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;
use proto::proto::permission::service::v1 as permissionv1;
use proto::proto::translator::service::v1 as translatorv1;

use crate::service::identity::role_proto;
use crate::service::identity::user_proto;

/// Insert a USERNAME credential for a user (bcrypt of the given
/// password; no AES layer — administrative sets arrive in plaintext).
async fn insert_credential(
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

// ── User writes ──────────────────────────────────────────────────────

pub struct UserWriteServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::user_service_server::UserService for UserWriteServiceImpl {
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
        let row = repo::users_by_id(&self.state.db, id as i64).await?;
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
        let req = request.into_inner();
        let Some(identityv1::delete_user_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_users(&self.state.db, id as i64).await?;
        sys_user_credentials::Entity::delete_many()
            .filter(sys_user_credentials::Column::UserId.eq(id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
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

// ── UserProfile (the app face) ───────────────────────────────────────

pub struct UserProfileServiceImpl {
    pub state: Arc<AppState>,
}

fn operator_user_id<T>(request: &Request<T>) -> Result<i64, Status> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| Status::unauthenticated("user identity required"))
}

#[async_trait::async_trait]
impl identityv1::user_profile_service_server::UserProfileService for UserProfileServiceImpl {
    async fn get_user(
        &self,
        request: Request<pbjson_types::Empty>,
    ) -> Result<Response<identityv1::User>, Status> {
        let uid = operator_user_id(&request)?;
        let row = repo::users_by_id(&self.state.db, uid).await?;
        Ok(Response::new(user_proto(row)))
    }

    async fn update_user(
        &self,
        request: Request<identityv1::UpdateUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let uid = operator_user_id(&request)?;
        let mut req = request.into_inner();
        req.id = uid as u32;
        <UserWriteServiceImpl as identityv1::user_service_server::UserService>::update(
            &UserWriteServiceImpl {
                state: Arc::clone(&self.state),
            },
            Request::new(req),
        )
        .await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Role writes ──────────────────────────────────────────────────────

pub struct RoleWriteServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::role_service_server::RoleService for RoleWriteServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListRoleResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_roles::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListRoleResponse {
            items: rows.into_iter().map(role_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountRoleResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_roles::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountRoleResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetRoleRequest>,
    ) -> Result<Response<permissionv1::Role>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_role_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::roles_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(role_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateRoleRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = sys_roles::ActiveModel {
            name: Set(Some(data.name.clone().unwrap_or_default())),
            code: Set(Some(data.code.clone().unwrap_or_default())),
            description: Set(data.description),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_protected: Set(data.is_protected.unwrap_or(false)),
            r#type: Set("SYSTEM".to_string()),
            status: Set("ON".to_string()),
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        for pid in data.permissions.iter().map(|v| *v as i64) {
            sys_role_permissions::ActiveModel {
                tenant_id: Set(row.tenant_id),
                role_id: Set(row.id),
                permission_id: Set(pid),
                status: Set("ON".to_string()),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<permissionv1::UpdateRoleRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_roles::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("role"))?;
        let mut a: sys_roles::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.description {
                a.description = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteRoleRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_role_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_roles(&self.state.db, id as i64).await?;
        sys_role_permissions::Entity::delete_many()
            .filter(sys_role_permissions::Column::RoleId.eq(id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn assign_roles_to_user(
        &self,
        request: Request<permissionv1::AssignRolesToUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        // Replace semantics: clear then bind.
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(req.user_id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        for role_id in req.role_ids.iter().map(|v| *v as i64) {
            sys_user_roles::ActiveModel {
                tenant_id: Set(Some(req.tenant_id as i64)),
                user_id: Set(req.user_id as i64),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Tenant writes ────────────────────────────────────────────────────

pub struct TenantWriteServiceImpl {
    pub state: Arc<AppState>,
}

use crate::service::identity::tenant_proto;

#[async_trait::async_trait]
impl identityv1::tenant_service_server::TenantService for TenantWriteServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListTenantResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_tenants::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListTenantResponse {
            items: rows.into_iter().map(tenant_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountTenantResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_tenants::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountTenantResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetTenantRequest>,
    ) -> Result<Response<identityv1::Tenant>, Status> {
        let req = request.into_inner();
        use identityv1::get_tenant_request::QueryBy;
        let row = match req.query_by {
            Some(QueryBy::Id(id)) => {
                sys_tenants::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Code(code)) => {
                sys_tenants::Entity::find()
                    .filter(sys_tenants::Column::Code.eq(code))
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Name(name)) => {
                sys_tenants::Entity::find()
                    .filter(sys_tenants::Column::Name.eq(name))
                    .one(&self.state.db)
                    .await
            }
            None => return Err(bad("query_by required")),
        }
        .map_err(db_status)?
        .ok_or_else(|| not_found("tenant"))?;
        Ok(Response::new(tenant_proto(row)))
    }

    async fn create(
        &self,
        request: Request<identityv1::CreateTenantRequest>,
    ) -> Result<Response<identityv1::Tenant>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = sys_tenants::ActiveModel {
            name: Set(Some(data.name.clone().unwrap_or_default())),
            code: Set(Some(data.code.clone().unwrap_or_default())),
            domain: Set(data.domain),
            industry: Set(data.industry),
            status: Set(Some("ON".to_string())),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(tenant_proto(row)))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdateTenantRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_tenants::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tenant"))?;
        let mut a: sys_tenants::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.domain {
                a.domain = Set(Some(v));
            }
            if let Some(v) = data.status {
                a.status = Set(Some(if v == 1 { "ON" } else { "OFF" }.to_string()));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeleteTenantRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(identityv1::delete_tenant_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_tenants(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn tenant_exists(
        &self,
        request: Request<identityv1::TenantExistsRequest>,
    ) -> Result<Response<identityv1::TenantExistsResponse>, Status> {
        let req = request.into_inner();
        let row = sys_tenants::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(sys_tenants::Column::Code.eq(req.code.clone()))
                    .add(sys_tenants::Column::Name.eq(req.name.clone())),
            )
            .one(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::TenantExistsResponse {
            exist: row.is_some(),
        }))
    }
}

// ── Translator ───────────────────────────────────────────────────────

pub struct TranslatorServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl translatorv1::translator_service_server::TranslatorService for TranslatorServiceImpl {
    async fn translate(
        &self,
        request: Request<translatorv1::TranslateRequest>,
    ) -> Result<Response<translatorv1::TranslateResponse>, Status> {
        // The provider bridge (google/baidu/alibaba/volc, the reference's
        // pkg/translator) lands with the outbound-API phase; the identity
        // passthrough keeps the contract live for same-language calls.
        let _ = &self.state;
        let req = request.into_inner();
        let content = req.content.unwrap_or_default();
        Ok(Response::new(translatorv1::TranslateResponse {
            translated_content: Some(content.clone()),
            raw_content: Some(content),
        }))
    }
}

// ts_to_proto kept referenced for the mapper-external callers.
#[allow(dead_code)]
fn _ts(v: chrono::DateTime<chrono::FixedOffset>) -> Option<pbjson_types::Timestamp> {
    ts_to_proto(v)
}
