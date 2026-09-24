//! The identity write paths: user create/update/delete (with the
//! credential row), password edits, user/tenant existence checks, role
//! writes with role-permission bindings, tenant writes, the app-side
//! user profile face, and the translator passthrough.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::identity_repo as repo;
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_role_metadata, sys_role_permissions, sys_roles, sys_tenants, sys_user_credentials,
    sys_user_roles, sys_users,
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

// ── operator context + shared role helpers ───────────────────────────
// (the metadata equivalents of the reference's viewer helpers)

/// The role-code constants of the reference's pkg/constants/role.go.
const PLATFORM_ADMIN_ROLE_CODE: &str = "platform:admin";
const TENANT_ADMIN_TEMPLATE_ROLE_CODE: &str = "template:tenant:manager";
const TENANT_ADMIN_ROLE_CODE: &str = "tenant:manager";
const TENANT_ADMIN_ROLE_NAME: &str = "租户管理员";
/// The reference's constants.DefaultAdminUserName.
const DEFAULT_ADMIN_USER_NAME: &str = "admin";

/// The operator's tenant scope off the verified claims (0 = platform).
fn operator_tenant_id<T>(request: &Request<T>) -> u32 {
    request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
}

/// The operator identity when present (the reference tolerates its
/// absence on some flows, defaulting to operator 0).
fn optional_operator_user_id<T>(request: &Request<T>) -> i64 {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(0)
}

/// validateTargetUserTenant — platform/system callers pass; a tenant
/// operator may only touch same-tenant users. The reference's lookup is
/// tenant-predicated, so a cross-tenant user reads as not-found there.
async fn validate_target_user_tenant(
    db: &sea_orm::DatabaseConnection,
    caller_tenant_id: u32,
    target_user_id: u32,
) -> Result<sys_users::Model, Status> {
    let target = sys_users::Entity::find_by_id(target_user_id as i64)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| bad("target user not found"))?;
    if caller_tenant_id > 0 && target.tenant_id.unwrap_or(0) != caller_tenant_id as i64 {
        return Err(forbidden("cannot access a user in another tenant"));
    }
    Ok(target)
}

/// CanAssignRole — the role must be visible in the operator's tenant
/// scope, enabled, non-template, non-blocked-sync, and not the
/// platform-admin template.
async fn can_assign_role(
    db: &sea_orm::DatabaseConnection,
    caller_tenant_id: u32,
    role_id: u32,
) -> Result<(), Status> {
    let role = sys_roles::Entity::find_by_id(role_id as i64)
        .one(db)
        .await
        .map_err(db_status)?;
    // the reference's tenant-predicated lookup hides cross-tenant roles
    if role.is_none()
        || (caller_tenant_id > 0
            && role
                .as_ref()
                .is_some_and(|r| r.tenant_id.unwrap_or(0) != caller_tenant_id as i64))
    {
        return Err(not_found("role"));
    }
    let role = role.unwrap();
    if role.status != "ON" {
        return Err(forbidden("role is not enabled"));
    }
    let metadata = repo::role_metadata_by_role_id(db, role_id as i64)
        .await?
        .ok_or_else(|| not_found("role metadata"))?;
    if metadata.is_template.unwrap_or(false) {
        return Err(forbidden("role template cannot be assigned"));
    }
    if metadata.sync_policy.as_deref() == Some("BLOCKED") {
        return Err(forbidden("blocked-sync role cannot be assigned"));
    }
    if metadata.template_for.as_deref() == Some(PLATFORM_ADMIN_ROLE_CODE) {
        return Err(forbidden("platform admin template role cannot be assigned"));
    }
    Ok(())
}

/// The binding row → protojson (status string → the proto enum ordinal).
fn user_role_proto(r: sys_user_roles::Model) -> permissionv1::UserRole {
    permissionv1::UserRole {
        id: Some(r.id as u32),
        user_id: Some(r.user_id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        role_id: Some(r.role_id as u32),
        is_primary: Some(r.is_primary),
        status: Some(match r.status.as_str() {
            "PENDING" => 1,
            "ACTIVE" => 2,
            "DISABLED" => 3,
            "EXPIRED" => 4,
            _ => 0,
        }),
        assigned_at: r.assigned_at.and_then(ts_to_proto),
        assigned_by: r.assigned_by.map(|v| v as u32),
        start_at: r.start_at.and_then(ts_to_proto),
        end_at: r.end_at.and_then(ts_to_proto),
        created_by: r.created_by.map(|v| v as u32),
        ..Default::default()
    }
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
        // The operator context is mandatory: an anonymous delete must
        // never pass (the reference errors before any lookup).
        let operator_id = operator_user_id(&request)?;
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
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        let Some(permissionv1::delete_role_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let id = id as i64;

        let txn = self.state.db.begin().await.map_err(db_status)?;
        // 租户作用域：仅查询本租户角色，避免跨租户删他人角色（系统角色
        // tenant 为 NULL，租户操作者读不到即视为不存在）。
        let role = sys_roles::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("role"))?;
        if caller_tenant_id > 0 && role.tenant_id.unwrap_or(0) != caller_tenant_id as i64 {
            return Err(not_found("role"));
        }

        // 保护角色禁止删除
        if role.is_protected {
            return Err(forbidden("protected role cannot be deleted"));
        }

        // 删除角色记录 + 清理 sys_role_permissions / sys_user_roles 绑定
        //（同一事务，失败回滚）。
        sys_roles::Entity::delete_by_id(id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        let mut perms = sys_role_permissions::Entity::delete_many()
            .filter(sys_role_permissions::Column::RoleId.eq(id));
        if caller_tenant_id > 0 {
            perms =
                perms.filter(sys_role_permissions::Column::TenantId.eq(caller_tenant_id as i64));
        }
        perms.exec(&txn).await.map_err(db_status)?;
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::RoleId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn list_role_codes_by_ids(
        &self,
        request: Request<permissionv1::ListRoleCodesByIdsRequest>,
    ) -> Result<Response<permissionv1::ListRoleCodesByIdsResponse>, Status> {
        let req = request.into_inner();
        let role_ids: Vec<i64> = req.role_ids.iter().map(|v| *v as i64).collect();
        let role_codes = repo::role_codes_by_ids(&self.state.db, &role_ids).await?;
        Ok(Response::new(permissionv1::ListRoleCodesByIdsResponse {
            role_codes,
        }))
    }

    async fn list_role_ids_by_codes(
        &self,
        request: Request<permissionv1::ListRoleIdsByCodesRequest>,
    ) -> Result<Response<permissionv1::ListRoleIdsByCodesResponse>, Status> {
        // role.code 仅在 (tenant_id, code) 维度唯一，平台上下文下按 code
        // 查询会跨租户返回所有匹配角色。仅允许具名租户上下文查询。
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        if caller_tenant_id == 0 {
            return Err(bad("tenant scope required to query role ids by codes"));
        }
        let role_ids =
            repo::role_ids_by_codes(&self.state.db, &req.role_codes, caller_tenant_id as i64)
                .await?;
        Ok(Response::new(permissionv1::ListRoleIdsByCodesResponse {
            role_ids: role_ids.iter().map(|v| *v as u32).collect(),
        }))
    }

    async fn list_permission_ids(
        &self,
        request: Request<permissionv1::ListPermissionIdsRequest>,
    ) -> Result<Response<permissionv1::ListPermissionIdsResponse>, Status> {
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        let db = &self.state.db;
        let mut permission_ids: Vec<i64> = Vec::new();
        match req.query_by {
            Some(permissionv1::list_permission_ids_request::QueryBy::RoleId(role_id)) => {
                permission_ids = repo::permission_ids_by_role_ids(db, &[role_id as i64]).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::RoleCode(code)) => {
                if caller_tenant_id == 0 {
                    return Err(bad("tenant scope required to query role ids by codes"));
                }
                let role_ids =
                    repo::role_ids_by_codes(db, &[code], caller_tenant_id as i64).await?;
                permission_ids = repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::UserId(user_id)) => {
                // 校验目标用户归属当前调用者租户，避免跨租户枚举他人权限 ID
                validate_target_user_tenant(db, caller_tenant_id, user_id).await?;
                // the reference keeps expired bindings on this path
                let role_ids = repo::user_role_ids(db, user_id as i64, true).await?;
                permission_ids = repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            None => {
                if !req.role_ids.is_empty() {
                    let role_ids: Vec<i64> = req.role_ids.iter().map(|v| *v as i64).collect();
                    permission_ids.extend(repo::permission_ids_by_role_ids(db, &role_ids).await?);
                }
                if !req.role_codes.is_empty() {
                    if caller_tenant_id == 0 {
                        return Err(bad("tenant scope required to query role ids by codes"));
                    }
                    let role_ids =
                        repo::role_ids_by_codes(db, &req.role_codes, caller_tenant_id as i64)
                            .await?;
                    permission_ids.extend(repo::permission_ids_by_role_ids(db, &role_ids).await?);
                }
            }
        }
        Ok(Response::new(permissionv1::ListPermissionIdsResponse {
            permission_ids: permission_ids.iter().map(|v| *v as u32).collect(),
        }))
    }

    async fn list_user_role_i_ds(
        &self,
        request: Request<permissionv1::ListUserRoleIDsRequest>,
    ) -> Result<Response<permissionv1::ListUserRoleIDsResponse>, Status> {
        // 校验目标用户归属当前调用者租户，避免跨租户枚举他人角色 ID。
        // 仅 id 分支可查：用户名分支在参照实现里退化为 id=0，同样被
        // 目标用户校验拦下。
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        let user_id = match req.query_by {
            Some(permissionv1::list_user_role_i_ds_request::QueryBy::UserId(id)) => id,
            _ => 0,
        };
        validate_target_user_tenant(&self.state.db, caller_tenant_id, user_id).await?;
        // the reference keeps expired bindings on this path
        let role_ids = repo::user_role_ids(&self.state.db, user_id as i64, true).await?;
        Ok(Response::new(permissionv1::ListUserRoleIDsResponse {
            role_ids: role_ids.iter().map(|v| *v as u32).collect(),
        }))
    }

    async fn get_user_roles(
        &self,
        request: Request<permissionv1::GetUserRolesRequest>,
    ) -> Result<Response<permissionv1::GetUserRolesResponse>, Status> {
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        if req.user_id == 0 {
            return Err(bad("invalid parameter"));
        }
        // 校验目标用户归属当前调用者租户，避免跨租户枚举他人角色绑定
        validate_target_user_tenant(&self.state.db, caller_tenant_id, req.user_id).await?;
        let bindings =
            repo::user_roles_by_user_id(&self.state.db, req.user_id as i64, req.include_expired)
                .await?;
        Ok(Response::new(permissionv1::GetUserRolesResponse {
            bindings: bindings.into_iter().map(user_role_proto).collect(),
        }))
    }

    async fn assign_roles_to_user(
        &self,
        request: Request<permissionv1::AssignRolesToUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // 操作人身份与租户必须从元数据推导，忽略客户端传入的
        // operator_id/tenant_id，防止越权将角色绑定写入他租户或伪造审计归属
        let operator_id = operator_user_id(&request)?;
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        if req.user_id == 0 || req.role_ids.is_empty() {
            return Err(bad("invalid parameter"));
        }
        // 校验目标用户归属当前调用者租户：平台上下文放行；租户用户仅可
        // 向本租户用户授权
        validate_target_user_tenant(&self.state.db, caller_tenant_id, req.user_id).await?;
        // 逐个校验角色可分配性（启用/非模板/非阻断/非平台管理员模板）
        for role_id in &req.role_ids {
            can_assign_role(&self.state.db, caller_tenant_id, *role_id).await?;
        }

        let now = store::now();
        let txn = self.state.db.begin().await.map_err(db_status)?;
        // Replace semantics: clear then bind（同事务，失败回滚）。
        sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(req.user_id as i64))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        for role_id in req.role_ids.iter().map(|v| *v as i64) {
            sys_user_roles::ActiveModel {
                tenant_id: Set(Some(caller_tenant_id as i64)),
                user_id: Set(req.user_id as i64),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                assigned_by: Set(Some(operator_id)),
                assigned_at: Set(Some(now)),
                start_at: Set(Some(now)),
                created_by: Set(Some(operator_id)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn unassign_roles_from_user(
        &self,
        request: Request<permissionv1::UnassignRolesFromUserRequest>,
    ) -> Result<Response<permissionv1::UnassignRolesFromUserResponse>, Status> {
        let caller_tenant_id = operator_tenant_id(&request);
        let req = request.into_inner();
        if req.user_id == 0 || req.role_ids.is_empty() {
            return Err(bad("invalid parameter"));
        }
        // 租户作用域：仅删除本租户的 user_role 关联，避免跨租户剥权
        let mut query = sys_user_roles::Entity::delete_many()
            .filter(sys_user_roles::Column::UserId.eq(req.user_id as i64))
            .filter(sys_user_roles::Column::RoleId.is_in(req.role_ids.iter().map(|v| *v as i64)));
        if caller_tenant_id > 0 {
            query = query.filter(sys_user_roles::Column::TenantId.eq(caller_tenant_id as i64));
        }
        query.exec(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(permissionv1::UnassignRolesFromUserResponse {
            removed_role_ids: req.role_ids,
            not_found_role_ids: Vec::new(),
        }))
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

    /// CreateTenantWithAdminUser — one transaction: create the tenant,
    /// copy the tenant-admin role template into it, create the admin
    /// user with that role, write the bcrypt credential, and point the
    /// tenant's admin_user_id at the new user. Any failure rolls the
    /// whole chain back.
    async fn create_tenant_with_admin_user(
        &self,
        request: Request<identityv1::CreateTenantWithAdminUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // 操作人身份必须从元数据推导，忽略客户端传入的 operator_user_id，
        // 防止伪造审计归属（缺失时按 0 记，与参照一致）
        let operator_id = optional_operator_user_id(&request);
        let req = request.into_inner();
        let Some(tenant) = req.tenant else {
            return Err(bad("invalid parameter"));
        };
        let Some(user) = req.user else {
            return Err(bad("invalid parameter"));
        };

        // The reference's duplicate check discards the flag (dead guard):
        // it only surfaces DB failures — the code unique index remains the
        // real conflict guard. Kept query-for-query.
        let _ = sys_tenants::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(sys_tenants::Column::Code.eq(tenant.code.clone().unwrap_or_default()))
                    .add(sys_tenants::Column::Name.eq(tenant.name.clone().unwrap_or_default())),
            )
            .one(&self.state.db)
            .await
            .map_err(db_status)?;

        let txn = self.state.db.begin().await.map_err(db_status)?;
        let now = store::now();

        // 1. The tenant row.
        let tenant_row = sys_tenants::ActiveModel {
            name: Set(tenant.name.clone()),
            code: Set(tenant.code.clone()),
            domain: Set(tenant.domain.clone()),
            logo_url: Set(tenant.logo_url.clone()),
            industry: Set(tenant.industry.clone()),
            remark: Set(tenant.remark.clone()),
            subscription_plan: Set(tenant.subscription_plan.clone()),
            status: Set(Some("ON".to_string())),
            created_by: Set(tenant.created_by.map(|v| v as i64)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        let tenant_id = tenant_row.id;

        // 2. Copy the tenant-admin role template (protected, platform
        //    owned, ON) into the tenant.
        let template = sys_roles::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(sys_roles::Column::Code.eq(TENANT_ADMIN_TEMPLATE_ROLE_CODE))
                    .add(sys_roles::Column::Status.eq("ON"))
                    .add(sys_roles::Column::IsProtected.eq(true))
                    .add(
                        sea_orm::Condition::any()
                            .add(sys_roles::Column::TenantId.is_null())
                            .add(sys_roles::Column::TenantId.eq(0)),
                    ),
            )
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tenant admin role template"))?;
        let template_permission_ids: Vec<i64> = sys_role_permissions::Entity::find()
            .filter(sys_role_permissions::Column::RoleId.eq(template.id))
            .all(&txn)
            .await
            .map_err(db_status)?
            .into_iter()
            .map(|p| p.permission_id)
            .collect();

        let role_row = sys_roles::ActiveModel {
            name: Set(Some(TENANT_ADMIN_ROLE_NAME.to_string())),
            code: Set(Some(TENANT_ADMIN_ROLE_CODE.to_string())),
            r#type: Set("TENANT".to_string()),
            is_protected: Set(true),
            tenant_id: Set(Some(tenant_id)),
            created_by: Set(Some(operator_id)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        // Role metadata: not a template, points back at the role code it
        // was minted from; auto-sync, tenant scope.
        sys_role_metadata::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            role_id: Set(Some(role_row.id)),
            is_template: Set(Some(false)),
            template_for: Set(Some(TENANT_ADMIN_ROLE_CODE.to_string())),
            sync_policy: Set(Some("AUTO".to_string())),
            scope: Set(Some("TENANT".to_string())),
            created_by: Set(Some(operator_id)),
            created_at: Set(Some(now)),
            custom_overrides: Set(serde_json::json!({})),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        for pid in template_permission_ids {
            sys_role_permissions::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                role_id: Set(role_row.id),
                permission_id: Set(pid),
                status: Set("ON".to_string()),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }

        // 3. The admin user, bound to the copied role (the reference
        //    appends user.role_id to user.role_ids and dedupes).
        let admin_row = sys_users::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            username: Set(user.username.clone()),
            nickname: Set(user.nickname.clone()),
            realname: Set(user.realname.clone()),
            email: Set(user.email.clone()),
            mobile: Set(user.mobile.clone()),
            avatar: Set(user.avatar.clone()),
            status: Set(Some("NORMAL".to_string())),
            created_by: Set(user.created_by.map(|v| v as i64)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        let mut role_ids: Vec<i64> = Vec::new();
        if let Some(role_id) = user.role_id {
            role_ids.push(role_id as i64);
        }
        for role_id in user.role_ids.iter().map(|v| *v as i64) {
            if !role_ids.contains(&role_id) {
                role_ids.push(role_id);
            }
        }
        for role_id in role_ids {
            sys_user_roles::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                user_id: Set(admin_row.id),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                assigned_at: Set(Some(now)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }

        // 4. The bcrypt credential (USERNAME / PASSWORD_HASH, primary).
        insert_credential(
            &txn,
            tenant_id,
            admin_row.id,
            &admin_row.username.clone().unwrap_or_default(),
            &req.password,
        )
        .await?;

        // 5. Point the tenant at its admin user.
        sys_tenants::ActiveModel {
            id: Set(tenant_id),
            admin_user_id: Set(Some(admin_row.id)),
            ..Default::default()
        }
        .update(&txn)
        .await
        .map_err(db_status)?;

        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    /// 按域名解析租户 id（app BFF 匿名链路的 Host 归属）：domain 精确
    /// 匹配，未中按去端口主机名回退一次（开发前端恒带端口）；未匹配
    /// 返回 0，调用方据此 fail-closed。
    async fn resolve_tenant_by_domain(
        &self,
        request: Request<identityv1::ResolveTenantByDomainRequest>,
    ) -> Result<Response<identityv1::ResolveTenantByDomainResponse>, Status> {
        let domain = request.into_inner().domain;
        let lookup = |d: String| {
            sys_tenants::Entity::find()
                .filter(sys_tenants::Column::Domain.eq(d))
                .one(&self.state.db)
        };
        let mut row = lookup(domain.clone()).await.map_err(db_status)?;
        if row.is_none() {
            if let Some((host, _port)) = domain.split_once(':') {
                if !host.is_empty() {
                    row = lookup(host.to_string()).await.map_err(db_status)?;
                }
            }
        }
        Ok(Response::new(identityv1::ResolveTenantByDomainResponse {
            tenant_id: row.map(|r| r.id as u32).unwrap_or(0),
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
