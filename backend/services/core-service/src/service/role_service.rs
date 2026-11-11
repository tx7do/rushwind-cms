//! The role service — the complete role face: list/get plus create/
//! update/delete with the role-permission rebind, the parent-role
//! binding (the inheritance validation: child permissions ⊆ parent's
//! granted set, cycle detection, the three-state parent_id update,
//! child detach on delete), the user-role assignments and their guards
//! (CanAssignRole), mirroring the reference's role_service.go.

use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::{role_metadata_repo, role_permission_repo, role_repo, user_role_repo};
use crate::service::constants::{is_reserved_role_code, PLATFORM_ADMIN_ROLE_CODE};
use crate::service::context::{operator_of, operator_tenant_id};
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{sys_role_permissions, sys_roles, sys_user_roles, sys_users};
use store::paging::fetch_paged;

use proto::proto::permission::service::v1 as permissionv1;

/// validateTargetUserTenant — platform/system callers pass; a tenant
/// operator may only touch same-tenant users. The reference's lookup is
/// tenant-predicated, so a cross-tenant user reads as not-found there.
async fn validate_target_user_tenant(
    db: &sea_orm::DatabaseConnection,
    caller_tenant_id: i64,
    target_user_id: i64,
) -> Result<sys_users::Model, Status> {
    let target = sys_users::Entity::find_by_id(target_user_id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| bad("target user not found"))?;
    if caller_tenant_id > 0 && target.tenant_id.unwrap_or(0) != caller_tenant_id {
        return Err(forbidden("cannot access a user in another tenant"));
    }
    Ok(target)
}

/// CanAssignRole — the role must be visible in the operator's tenant
/// scope, enabled, non-template, non-blocked-sync, and not the
/// platform-admin template.
async fn can_assign_role(
    db: &sea_orm::DatabaseConnection,
    caller_tenant_id: i64,
    role_id: i64,
) -> Result<(), Status> {
    let role = sys_roles::Entity::find_by_id(role_id)
        .one(db)
        .await
        .map_err(db_status)?;
    // the reference's tenant-predicated lookup hides cross-tenant roles
    if role.is_none()
        || (caller_tenant_id > 0
            && role
                .as_ref()
                .is_some_and(|r| r.tenant_id.unwrap_or(0) != caller_tenant_id))
    {
        return Err(not_found("role"));
    }
    let role = role.unwrap();
    if role.status != "ON" {
        return Err(forbidden("role is not enabled"));
    }
    let metadata = role_metadata_repo::role_metadata_by_role_id(db, role_id)
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

/// List/Get 回填 parentName 的批量底表：一次取齐本批角色的父角色。
async fn parent_names_of(
    db: &sea_orm::DatabaseConnection,
    rows: &[sys_roles::Model],
) -> HashMap<i64, String> {
    let parent_ids: Vec<i64> = rows
        .iter()
        .filter_map(|r| r.parent_id.filter(|v| *v > 0))
        .collect();
    if parent_ids.is_empty() {
        return HashMap::new();
    }
    let parents = role_repo::roles_by_ids(db, &parent_ids)
        .await
        .unwrap_or_default();
    parents
        .into_iter()
        .filter_map(|p| p.name.map(|n| (p.id, n)))
        .collect()
}

pub(crate) fn role_proto(r: sys_roles::Model, parent_name: Option<String>) -> permissionv1::Role {
    permissionv1::Role {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        is_protected: Some(r.is_protected),
        r#type: Some(1),
        status: Some(1),
        parent_id: r.parent_id,
        parent_name,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// The binding row → protojson (status string → the proto enum ordinal).
fn user_role_proto(r: sys_user_roles::Model) -> permissionv1::UserRole {
    permissionv1::UserRole {
        id: Some(r.id),
        user_id: Some(r.user_id),
        tenant_id: r.tenant_id,
        role_id: Some(r.role_id),
        is_primary: Some(r.is_primary),
        status: Some(match r.status.as_str() {
            "PENDING" => 1,
            "ACTIVE" => 2,
            "DISABLED" => 3,
            "EXPIRED" => 4,
            _ => 0,
        }),
        assigned_at: r.assigned_at.and_then(ts_to_proto),
        assigned_by: r.assigned_by,
        start_at: r.start_at.and_then(ts_to_proto),
        end_at: r.end_at.and_then(ts_to_proto),
        created_by: r.created_by,
        ..Default::default()
    }
}

// ── Role writes ──────────────────────────────────────────────────────

pub struct RoleService {
    pub state: Arc<AppState>,
}

impl RoleService {
    /// validateParentBinding — 父子角色绑定的合法性（权限继承核心约束，
    /// 编辑时校验语义）。parent_id 为 0/空表示根角色，直接放行。校验：
    /// 1. 父角色存在；2. 父角色与本角色同租户（tenantID 为本角色归属租户，
    ///    Create 侧以操作者上下文折算）；3. wantPerms ⊆ 父角色已绑定权限集。
    ///
    /// role_id 为 0 表示 Create 场景（尚无自身 ID）。
    async fn validate_parent_binding(
        db: &sea_orm::DatabaseConnection,
        role_id: i64,
        parent_id: i64,
        tenant_id: i64,
        want_perms: &[i64],
    ) -> Result<(), Status> {
        let _ = role_id;
        if parent_id <= 0 {
            return Ok(());
        }
        let parent = role_repo::roles_by_id(db, parent_id)
            .await
            .map_err(|_| forbidden("parent role not found"))?;
        if parent.tenant_id.unwrap_or(0) != tenant_id {
            return Err(forbidden("parent role must belong to the same tenant"));
        }
        if want_perms.is_empty() {
            return Ok(());
        }
        let cap_set = role_permission_repo::permission_ids_by_role_ids(db, &[parent_id]).await?;
        let exceeding = want_perms.iter().filter(|id| !cap_set.contains(id)).count();
        if exceeding > 0 {
            return Err(forbidden(&format!(
                "{exceeding} permission(s) exceed the parent role's granted permission set"
            )));
        }
        Ok(())
    }

    /// ensureNoRoleCycle — 沿候选父角色上溯检测层级环（Update 改父时
    /// 调用）：若从候选父出发能回到自身，说明会成环，拒绝。上溯深度设
    /// 上限防脏数据死循环。
    async fn ensure_no_role_cycle(
        db: &sea_orm::DatabaseConnection,
        role_id: i64,
        parent_id: i64,
    ) -> Result<(), Status> {
        const MAX_DEPTH: usize = 32;
        let mut current = parent_id;
        for _ in 0..MAX_DEPTH {
            if current <= 0 {
                return Ok(());
            }
            if current == role_id {
                return Err(forbidden("circular role hierarchy detected"));
            }
            // 上溯链断裂（父角色不存在）不在此处报错，交给
            // validate_parent_binding。
            let Ok(parent) = role_repo::roles_by_id(db, current).await else {
                return Ok(());
            };
            current = parent.parent_id.unwrap_or(0);
        }
        Ok(())
    }
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
        let parent_names = parent_names_of(&self.state.db, &rows).await;
        Ok(Response::new(permissionv1::ListRoleResponse {
            items: rows
                .into_iter()
                .map(|r| {
                    let name = r.parent_id.and_then(|pid| parent_names.get(&pid).cloned());
                    role_proto(r, name)
                })
                .collect(),
            total: total as i64,
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
            count: total as i64,
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
        let row = role_repo::roles_by_id(&self.state.db, id).await?;
        let parent_name = match row.parent_id.filter(|v| *v > 0) {
            Some(pid) => role_repo::roles_by_id(&self.state.db, pid)
                .await
                .ok()
                .and_then(|p| p.name),
            None => None,
        };
        Ok(Response::new(role_proto(row, parent_name)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateRoleRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // 租户操作者的归属租户以操作者上下文为准，载荷里的 tenant_id
        // 不可信——先取（into_inner 会消费 request）。
        let operator_tenant = operator_tenant_id(&request);
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 角色码保留字校验：内置身份码（platform:* / template:* /
        // tenant:manager）在登录链路与鉴权引擎里按码值承载身份判定语义，
        // 用户自建角色不得占用，否则自建 platform:admin 再登录即可拿到
        // 平台管理员标志（越权）。
        if is_reserved_role_code(&data.code.clone().unwrap_or_default()) {
            return Err(bad("role code is reserved"));
        }
        // 父角色绑定校验（权限继承）。
        let role_tenant_id = match operator_tenant {
            v if v > 0 => v,
            _ => data.tenant_id.unwrap_or(0),
        };
        Self::validate_parent_binding(
            &self.state.db,
            0,
            data.parent_id.unwrap_or(0),
            role_tenant_id,
            &data.permissions,
        )
        .await?;

        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = sys_roles::ActiveModel {
            name: Set(Some(data.name.clone().unwrap_or_default())),
            code: Set(Some(data.code.clone().unwrap_or_default())),
            description: Set(data.description),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_protected: Set(data.is_protected.unwrap_or(false)),
            r#type: Set("SYSTEM".to_string()),
            status: Set("ON".to_string()),
            tenant_id: Set(data.tenant_id),
            parent_id: Set(data.parent_id.filter(|v| *v > 0)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        for pid in &data.permissions {
            sys_role_permissions::ActiveModel {
                tenant_id: Set(row.tenant_id),
                role_id: Set(row.id),
                permission_id: Set(*pid),
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
        let row = sys_roles::Entity::find_by_id(req.id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("role"))?;

        // mask 声明探测：proto 字段名与 json_name 双探测；mask 为空/缺省
        // 视为全量更新（FilterByFieldMask 语义）——标量列全按载荷写，
        // 权限集按权威口径整表替换（与参照的 fullUpdate 分支一致）；
        // parent_id：显式声明走三态（mask 声明 + 载荷空 ⇒ 显式清空），
        // 全量更新走 SetNillable 口径（载荷 None 不动旧值），与参照的
        // updateParentID / hasMaskPath(nil)=true 组合一致。
        let paths = req
            .update_mask
            .as_ref()
            .map(|m| m.paths.as_slice())
            .unwrap_or_default();
        let full_update = paths.is_empty();
        let declares =
            |names: &[&str]| full_update || paths.iter().any(|p| names.contains(&p.as_str()));
        let perms_declared = declares(&["permissions"]);
        let parent_declared = paths.iter().any(|p| p == "parent_id" || p == "parentId");

        if let Some(data) = &req.data {
            // 角色码保留字校验：与 Create 同口径——普通角色的 code 可改，
            // 必须拦住改成内置身份码的路径。
            if let Some(code) = &data.code {
                if is_reserved_role_code(code) {
                    return Err(bad("role code is reserved"));
                }
            }
        }

        // 父角色绑定校验（权限继承，编辑时校验语义）：本次请求生效的父
        // 角色与权限集分别按"mask 是否声明"取舍——声明了取载荷，未声明
        // 取库中现值（改权限不动父、或换父不动权限，都要重新对齐约束）。
        // prost 的 repeated 字段不携带 presence：全量更新形态（编辑抽屉
        // 整表单提交）载荷权限集即权威口径（显式空数组=清空）。
        let effective_parent_id = if parent_declared || full_update {
            req.data.as_ref().and_then(|d| d.parent_id).unwrap_or(0)
        } else {
            row.parent_id.unwrap_or(0)
        };
        let effective_perms = if perms_declared {
            req.data
                .as_ref()
                .map(|d| d.permissions.clone())
                .unwrap_or_default()
        } else {
            role_permission_repo::permission_ids_by_role_ids(&self.state.db, &[row.id]).await?
        };
        if effective_parent_id > 0 {
            if parent_declared || full_update {
                Self::ensure_no_role_cycle(&self.state.db, req.id, effective_parent_id).await?;
            }
            Self::validate_parent_binding(
                &self.state.db,
                req.id,
                effective_parent_id,
                row.tenant_id.unwrap_or(0),
                &effective_perms,
            )
            .await?;
        }

        let txn = self.state.db.begin().await.map_err(db_status)?;

        // 权限绑定重写（ReplacePermissions）：mask 声明（或全量更新）即
        // 整表替换——先清后插，同一事务失败回滚。
        if perms_declared {
            sys_role_permissions::Entity::delete_many()
                .filter(sys_role_permissions::Column::RoleId.eq(req.id))
                .exec(&txn)
                .await
                .map_err(db_status)?;
            for pid in &effective_perms {
                sys_role_permissions::ActiveModel {
                    tenant_id: Set(row.tenant_id),
                    role_id: Set(req.id),
                    permission_id: Set(*pid),
                    status: Set("ON".to_string()),
                    created_at: Set(Some(store::now())),
                    ..Default::default()
                }
                .insert(&txn)
                .await
                .map_err(db_status)?;
            }
        }

        let mut a: sys_roles::ActiveModel = row.into();
        if let Some(data) = &req.data {
            if declares(&["name"]) {
                if let Some(v) = &data.name {
                    a.name = Set(Some(v.clone()));
                }
            }
            if declares(&["description"]) {
                if let Some(v) = &data.description {
                    a.description = Set(Some(v.clone()));
                }
            }
            if declares(&["sort_order", "sortOrder"]) {
                if let Some(v) = data.sort_order {
                    a.sort_order = Set(Some(v as i64));
                }
            }
            // parent_id：mask 显式声明走三态——载荷为 nil ⇒ 显式清空
            // （解除父子关系；SetNillable 对 nil 不动旧值，不特殊处理会
            // 导致"清空父角色"静默失效），载荷显式 0 表示降级为根角色，
            // 照常落库（本表无自引用外键）；全量更新走 SetNillable 口径
            // （载荷 None 不动旧值），与参照的分支组合一致。
            a.parent_id = if parent_declared {
                Set(data.parent_id.filter(|v| *v > 0))
            } else if full_update {
                match data.parent_id {
                    Some(v) => Set(Some(v)),
                    None => sea_orm::ActiveValue::NotSet,
                }
            } else {
                sea_orm::ActiveValue::NotSet
            };
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&txn).await.map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
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

        let txn = self.state.db.begin().await.map_err(db_status)?;
        // 租户作用域：仅查询本租户角色，避免跨租户删他人角色（系统角色
        // tenant 为 NULL，租户操作者读不到即视为不存在）。行不存在（并发
        // 删除/重复删除）按 404 语义返回。
        let role = sys_roles::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("role"))?;
        if caller_tenant_id > 0 && role.tenant_id.unwrap_or(0) != caller_tenant_id {
            return Err(not_found("role"));
        }

        // 保护角色禁止删除
        if role.is_protected {
            return Err(forbidden("protected role cannot be deleted"));
        }

        // 子角色自动解除父子关系（parent_id 置空），绑定权限保持不变——
        // 编辑时校验语义下子角色既有权限不因父角色删除而扩大，删除父角色
        // 无需先处理子角色。
        sys_roles::Entity::update_many()
            .filter(sys_roles::Column::ParentId.eq(id))
            .col_expr(
                sys_roles::Column::ParentId,
                sea_orm::sea_query::Expr::value(None::<i64>),
            )
            .exec(&txn)
            .await
            .map_err(db_status)?;

        // 删除角色记录 + 清理 sys_role_permissions / sys_user_roles 绑定
        //（同一事务，失败回滚）。
        sys_roles::Entity::delete_by_id(id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        let mut perms = sys_role_permissions::Entity::delete_many()
            .filter(sys_role_permissions::Column::RoleId.eq(id));
        if caller_tenant_id > 0 {
            perms = perms.filter(sys_role_permissions::Column::TenantId.eq(caller_tenant_id));
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
        let role_codes = role_repo::role_codes_by_ids(&self.state.db, &req.role_ids).await?;
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
            role_repo::role_ids_by_codes(&self.state.db, &req.role_codes, caller_tenant_id).await?;
        Ok(Response::new(permissionv1::ListRoleIdsByCodesResponse {
            role_ids,
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
                    role_permission_repo::permission_ids_by_role_ids(db, &[role_id]).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::RoleCode(code)) => {
                if caller_tenant_id == 0 {
                    return Err(bad("tenant scope required to query role ids by codes"));
                }
                let role_ids = role_repo::role_ids_by_codes(db, &[code], caller_tenant_id).await?;
                permission_ids =
                    role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            Some(permissionv1::list_permission_ids_request::QueryBy::UserId(user_id)) => {
                // 校验目标用户归属当前调用者租户，避免跨租户枚举他人权限 ID
                validate_target_user_tenant(db, caller_tenant_id, user_id).await?;
                // the reference keeps expired bindings on this path
                let role_ids = user_role_repo::user_role_ids(db, user_id, true).await?;
                permission_ids =
                    role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?;
            }
            None => {
                if !req.role_ids.is_empty() {
                    permission_ids.extend(
                        role_permission_repo::permission_ids_by_role_ids(db, &req.role_ids).await?,
                    );
                }
                if !req.role_codes.is_empty() {
                    if caller_tenant_id == 0 {
                        return Err(bad("tenant scope required to query role ids by codes"));
                    }
                    let role_ids =
                        role_repo::role_ids_by_codes(db, &req.role_codes, caller_tenant_id).await?;
                    permission_ids.extend(
                        role_permission_repo::permission_ids_by_role_ids(db, &role_ids).await?,
                    );
                }
            }
        }
        Ok(Response::new(permissionv1::ListPermissionIdsResponse {
            permission_ids,
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
        let role_ids = user_role_repo::user_role_ids(&self.state.db, user_id, true).await?;
        Ok(Response::new(permissionv1::ListUserRoleIDsResponse {
            role_ids,
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
            user_role_repo::user_roles_by_user_id(&self.state.db, req.user_id, req.include_expired)
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
            .filter(sys_user_roles::Column::UserId.eq(req.user_id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        for role_id in &req.role_ids {
            sys_user_roles::ActiveModel {
                tenant_id: Set(Some(caller_tenant_id)),
                user_id: Set(req.user_id),
                role_id: Set(*role_id),
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
            .filter(sys_user_roles::Column::UserId.eq(req.user_id))
            .filter(sys_user_roles::Column::RoleId.is_in(req.role_ids.iter().copied()));
        if caller_tenant_id > 0 {
            query = query.filter(sys_user_roles::Column::TenantId.eq(caller_tenant_id));
        }
        query.exec(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(permissionv1::UnassignRolesFromUserResponse {
            removed_role_ids: req.role_ids,
            not_found_role_ids: Vec::new(),
        }))
    }
}
