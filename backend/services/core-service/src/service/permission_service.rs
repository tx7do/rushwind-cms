//! The permission service — the permission-point CRUD plus the
//! SyncPermissions trigger into the derived-permission rebuild
//! (permission_service.go; the rebuild itself lives in
//! permission_sync.rs).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::{permission_repo, role_permission_repo};
use crate::service::permission_sync;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use proto::proto::permission::service::v1 as permissionv1;
use store::entities::{sys_permission_apis, sys_permission_menus, sys_permissions};
use store::paging::fetch_paged;

// ── Permission ───────────────────────────────────────────────────────

fn permission_proto(r: sys_permissions::Model) -> permissionv1::Permission {
    permissionv1::Permission {
        id: Some(r.id as u32),
        name: Some(r.name),
        code: Some(r.code),
        description: r.description,
        status: Some(1),
        group_id: r.group_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct PermissionService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::permission_service_server::PermissionService for PermissionService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListPermissionResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_permissions::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListPermissionResponse {
            items: rows.into_iter().map(permission_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountPermissionResponse>, Status> {
        let total = sys_permissions::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountPermissionResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetPermissionRequest>,
    ) -> Result<Response<permissionv1::Permission>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_permission_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = permission_repo::permissions_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(permission_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreatePermissionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_permissions::ActiveModel {
            name: Set(data.name.clone().unwrap_or_default()),
            code: Set(data.code.clone().unwrap_or_default()),
            description: Set(data.description),
            group_id: Set(data.group_id.map(|v| v as i64)),
            status: Set("ON".to_string()),
            tenant_id: Set(Some(0)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<permissionv1::UpdatePermissionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_permissions::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("permission"))?;
        let mut a: sys_permissions::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(v);
            }
            if let Some(v) = data.description {
                a.description = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeletePermissionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        match req.query_by {
            Some(permissionv1::delete_permission_request::QueryBy::Id(id)) => {
                permission_repo::delete_permissions(&self.state.db, id as i64).await?;
            }
            Some(permissionv1::delete_permission_request::QueryBy::GroupId(gid)) => {
                sys_permissions::Entity::delete_many()
                    .filter(sys_permissions::Column::GroupId.eq(gid as i64))
                    .exec(&self.state.db)
                    .await
                    .map_err(db_status)?;
            }
            Some(permissionv1::delete_permission_request::QueryBy::Code(code)) => {
                sys_permissions::Entity::delete_many()
                    .filter(sys_permissions::Column::Code.eq(code))
                    .exec(&self.state.db)
                    .await
                    .map_err(db_status)?;
            }
            None => return Err(bad("query_by required")),
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn sync_permissions(
        &self,
        request: Request<permissionv1::SyncPermissionsRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        // The enabled menu set is the input — the caller's menu
        // scan runs here (the reference splits it across its BFF);
        // only the operator identity rides the request.
        permission_sync::sync_permissions(&self.state.db, req.operator_id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    /// ListPermissionResources — 按权限 id（或角色 id 归并出的权限 id）
    /// 查关联资源 id，按资源类型分组返回（参照 permission_service.go 的
    /// RoleIds 分支 + permission_repo.go 的分组查询）。
    async fn list_permission_resources(
        &self,
        request: Request<permissionv1::ListPermissionResourcesRequest>,
    ) -> Result<Response<permissionv1::ListPermissionResourcesResponse>, Status> {
        let mut req = request.into_inner();
        if req.permission_ids.is_empty() && req.role_ids.is_empty() {
            return Err(bad("permission_ids and role_ids cannot be both empty"));
        }

        // RoleIds 分支：角色 → 权限 id，并入后按 Unique 去重（保持首现序）。
        if !req.role_ids.is_empty() {
            let role_ids: Vec<i64> = req.role_ids.iter().map(|v| *v as i64).collect();
            let limit = role_permission_repo::permission_ids_by_role_ids(&self.state.db, &role_ids)
                .await?
                .into_iter()
                .map(|v| v as u32);
            req.permission_ids.extend(limit);
            let mut seen = std::collections::HashSet::new();
            req.permission_ids.retain(|v| seen.insert(*v));
        }

        // repo 层守卫：归并后仍无权限 id 即拒（参照的 "invalid parameter"）。
        if req.permission_ids.is_empty() {
            return Err(bad("invalid parameter"));
        }
        let ids: Vec<i64> = req.permission_ids.iter().map(|v| *v as i64).collect();

        // 逐请求的类型查询：命中类型即使空集也入 map（参照语义），
        // 未知类型跳过。
        let mut resources = std::collections::HashMap::new();
        for ty in &req.resource_types {
            match permissionv1::list_permission_resources_request::ResourceType::try_from(*ty) {
                Ok(permissionv1::list_permission_resources_request::ResourceType::Api) => {
                    let rows = sys_permission_apis::Entity::find()
                        .filter(sys_permission_apis::Column::PermissionId.is_in(ids.clone()))
                        .all(&self.state.db)
                        .await
                        .map_err(db_status)?;
                    resources.insert(
                        *ty,
                        permissionv1::PermissionResourceIds {
                            ids: rows.into_iter().map(|r| r.api_id as u32).collect(),
                        },
                    );
                }
                Ok(permissionv1::list_permission_resources_request::ResourceType::Menu) => {
                    let rows = sys_permission_menus::Entity::find()
                        .filter(sys_permission_menus::Column::PermissionId.is_in(ids.clone()))
                        .all(&self.state.db)
                        .await
                        .map_err(db_status)?;
                    resources.insert(
                        *ty,
                        permissionv1::PermissionResourceIds {
                            ids: rows.into_iter().map(|r| r.menu_id as u32).collect(),
                        },
                    );
                }
                _ => continue,
            }
        }
        Ok(Response::new(
            permissionv1::ListPermissionResourcesResponse { resources },
        ))
    }

    /// ListPermissionCodesByIds — 按权限 id 查 code 列表（空集直接返回
    /// 空：参照的 ent `IDIn()` 空集即空结果）。
    async fn list_permission_codes_by_ids(
        &self,
        request: Request<permissionv1::ListPermissionCodesByIdsRequest>,
    ) -> Result<Response<permissionv1::ListPermissionCodesByIdsResponse>, Status> {
        let req = request.into_inner();
        if req.permission_ids.is_empty() {
            return Ok(Response::new(
                permissionv1::ListPermissionCodesByIdsResponse::default(),
            ));
        }
        let ids: Vec<i64> = req.permission_ids.iter().map(|v| *v as i64).collect();
        let rows = sys_permissions::Entity::find()
            .filter(sys_permissions::Column::Id.is_in(ids))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(
            permissionv1::ListPermissionCodesByIdsResponse {
                permission_codes: rows.into_iter().map(|r| r.code).collect(),
            },
        ))
    }
}
