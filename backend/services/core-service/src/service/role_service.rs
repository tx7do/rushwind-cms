//! The role service — the complete role face: list/get plus create/
//! update/delete with the role-permission rebind, the user-role
//! assignments and their guards (CanAssignRole), mirroring the
//! reference's role_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::{role_metadata_repo, role_permission_repo, role_repo, user_role_repo};
use crate::service::context::{operator_of, operator_tenant_id};
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{sys_role_permissions, sys_roles, sys_user_roles, sys_users};
use store::paging::fetch_paged;

use proto::proto::permission::service::v1 as permissionv1;

/// The role-code constants of the reference's pkg/constants/role.go.
const PLATFORM_ADMIN_ROLE_CODE: &str = "platform:admin";

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
    let metadata = role_metadata_repo::role_metadata_by_role_id(db, role_id as i64)
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

pub(crate) fn role_proto(r: sys_roles::Model) -> permissionv1::Role {
    permissionv1::Role {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        is_protected: Some(r.is_protected),
        r#type: Some(1),
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
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

// ── Role writes ──────────────────────────────────────────────────────

pub struct RoleService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::role_service_server::RoleService for RoleService {
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
        let row = role_repo::roles_by_id(&self.state.db, id as i64).await?;
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
        let role_codes = role_repo::role_codes_by_ids(&self.state.db, &role_ids).await?;
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
            role_repo::role_ids_by_codes(&self.state.db, &req.role_codes, caller_tenant_id as i64)
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
                permission_ids =
                    role_permission_repo::permission_ids_by_role_ids(db, &[role_id as i64]).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::RoleCode(code)) => {
                if caller_tenant_id == 0 {
                    return Err(bad("tenant scope required to query role ids by codes"));
                }
                let role_ids =
                    role_repo::role_ids_by_codes(db, &[code], caller_tenant_id as i64).await?;
                permission_ids =
                    role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::UserId(user_id)) => {
                // 校验目标用户归属当前调用者租户，避免跨租户枚举他人权限 ID
                validate_target_user_tenant(db, caller_tenant_id, user_id).await?;
                // the reference keeps expired bindings on this path
                let role_ids = user_role_repo::user_role_ids(db, user_id as i64, true).await?;
                permission_ids =
                    role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            None => {
                if !req.role_ids.is_empty() {
                    let role_ids: Vec<i64> = req.role_ids.iter().map(|v| *v as i64).collect();
                    permission_ids.extend(
                        role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?,
                    );
                }
                if !req.role_codes.is_empty() {
                    if caller_tenant_id == 0 {
                        return Err(bad("tenant scope required to query role ids by codes"));
                    }
                    let role_ids =
                        role_repo::role_ids_by_codes(db, &req.role_codes, caller_tenant_id as i64)
                            .await?;
                    permission_ids.extend(
                        role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?,
                    );
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
        let role_ids = user_role_repo::user_role_ids(&self.state.db, user_id as i64, true).await?;
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
        let bindings = user_role_repo::user_roles_by_user_id(
            &self.state.db,
            req.user_id as i64,
            req.include_expired,
        )
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
        let operator_id = operator_of(&request)?;
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
