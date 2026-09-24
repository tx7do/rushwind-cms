//! The permission cluster: Menu (tree), Api, PermissionGroup,
//! Permission — CRUD over the golden schema. The walk-route face
//! stays on its generated Unimplemented default (it rebuilds derived
//! tables; the dedicated pipeline owns that); the sync face — the
//! api registry upsert off the BFF's shipped route table — lives at
//! the tail.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::identity_repo;
use crate::data::permission_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_apis, sys_menus, sys_permission_apis, sys_permission_groups, sys_permission_menus,
    sys_permissions,
};
use store::paging::fetch_paged;

use proto::proto::permission::service::v1 as permissionv1;

fn menu_meta(json: Option<sea_orm::JsonValue>) -> Option<permissionv1::MenuMeta> {
    let v = json?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_owned);
    let b = |k: &str| v.get(k).and_then(|x| x.as_bool());
    let n = |k: &str| v.get(k).and_then(|x| x.as_i64()).map(|x| x as i32);
    Some(permissionv1::MenuMeta {
        title: s("title"),
        icon: s("icon"),
        active_icon: s("activeIcon"),
        active_path: s("activePath"),
        hide_in_menu: b("hideInMenu"),
        hide_in_tab: b("hideInTab"),
        hide_in_breadcrumb: b("hideInBreadcrumb"),
        hide_children_in_menu: b("hideChildrenInMenu"),
        keep_alive: b("keepAlive"),
        iframe_src: s("iframeSrc"),
        link: s("link"),
        order: n("order"),
        ..Default::default()
    })
}

fn menu_meta_json(meta: Option<permissionv1::MenuMeta>) -> Option<sea_orm::JsonValue> {
    let m = meta?;
    serde_json::json!({
        "title": m.title,
        "icon": m.icon,
        "activeIcon": m.active_icon,
        "activePath": m.active_path,
        "hideInMenu": m.hide_in_menu,
        "hideInTab": m.hide_in_tab,
        "hideInBreadcrumb": m.hide_in_breadcrumb,
        "hideChildrenInMenu": m.hide_children_in_menu,
        "keepAlive": m.keep_alive,
        "iframeSrc": m.iframe_src,
        "link": m.link,
        "order": m.order,
    })
    .into()
}

fn menu_proto(r: sys_menus::Model, children: Vec<sys_menus::Model>) -> permissionv1::Menu {
    permissionv1::Menu {
        id: Some(r.id as u32),
        status: Some(1),
        path: r.path,
        redirect: r.redirect,
        alias: r.r#alias,
        name: r.name,
        component: r.component,
        meta: menu_meta(r.meta),
        parent_id: r.parent_id.map(|v| v as u32),
        children: children
            .into_iter()
            .map(|c| menu_proto(c, Vec::new()))
            .collect(),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct MenuServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::menu_service_server::MenuService for MenuServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListMenuResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_menus::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListMenuResponse {
            items: rows
                .into_iter()
                .map(|r| menu_proto(r, Vec::new()))
                .collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountMenuResponse>, Status> {
        let total = sys_menus::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountMenuResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetMenuRequest>,
    ) -> Result<Response<permissionv1::Menu>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_menu_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::menus_by_id(&self.state.db, id as i64).await?;
        let children = sys_menus::Entity::find()
            .filter(sys_menus::Column::ParentId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(menu_proto(row, children)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateMenuRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_menus::ActiveModel {
            r#type: Set(data.r#type.map(|v| v.to_string())),
            path: Set(data.path),
            redirect: Set(data.redirect),
            r#alias: Set(data.alias),
            name: Set(data.name),
            component: Set(data.component),
            meta: Set(menu_meta_json(data.meta)),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
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
        request: Request<permissionv1::UpdateMenuRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_menus::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("menu"))?;
        let mut a: sys_menus::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.path {
                a.path = Set(Some(v));
            }
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.component {
                a.component = Set(Some(v));
            }
            if let Some(v) = data.redirect {
                a.redirect = Set(Some(v));
            }
            if data.meta.is_some() {
                a.meta = Set(menu_meta_json(data.meta));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteMenuRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_menu_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_menus(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Api ──────────────────────────────────────────────────────────────

fn api_proto(r: sys_apis::Model) -> permissionv1::Api {
    permissionv1::Api {
        id: Some(r.id as u32),
        operation: r.operation,
        path: r.path,
        method: r.method,
        module: r.module,
        module_description: r.module_description,
        description: r.description,
        scope: None,
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct ApiServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::api_service_server::ApiService for ApiServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListApiResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_apis::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListApiResponse {
            items: rows.into_iter().map(api_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountApiResponse>, Status> {
        let total = sys_apis::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountApiResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetApiRequest>,
    ) -> Result<Response<permissionv1::Api>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_api_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::apis_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(api_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_apis::ActiveModel {
            operation: Set(Some(data.operation.clone().unwrap_or_default())),
            path: Set(Some(data.path.clone().unwrap_or_default())),
            method: Set(Some(data.method.clone().unwrap_or_default())),
            module: Set(data.module),
            module_description: Set(data.module_description),
            description: Set(data.description),
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
        request: Request<permissionv1::UpdateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_apis::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("api"))?;
        let mut a: sys_apis::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.description {
                a.description = Set(Some(v));
            }
            if let Some(v) = data.module_description {
                a.module_description = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_api_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_apis(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── PermissionGroup ──────────────────────────────────────────────────

fn group_proto(r: sys_permission_groups::Model) -> permissionv1::PermissionGroup {
    permissionv1::PermissionGroup {
        id: Some(r.id as u32),
        name: Some(r.name),
        path: r.path,
        module: r.module,
        sort_order: r.sort_order.map(|v| v as u32),
        status: Some(1),
        description: r.description,
        parent_id: r.parent_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct PermissionGroupServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::permission_group_service_server::PermissionGroupService
    for PermissionGroupServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListPermissionGroupResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_permission_groups::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListPermissionGroupResponse {
            items: rows.into_iter().map(group_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountPermissionGroupResponse>, Status> {
        let total = sys_permission_groups::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountPermissionGroupResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetPermissionGroupRequest>,
    ) -> Result<Response<permissionv1::PermissionGroup>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_permission_group_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::permission_groups_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(group_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreatePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_permission_groups::ActiveModel {
            name: Set(data.name.clone().unwrap_or_default()),
            path: Set(data.path),
            module: Set(data.module),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            status: Set("ON".to_string()),
            description: Set(data.description),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
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
        request: Request<permissionv1::UpdatePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_permission_groups::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("permission group"))?;
        let mut a: sys_permission_groups::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(v);
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
        request: Request<permissionv1::DeletePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_permission_group_request::QueryBy::Id(id)) = req.query_by
        else {
            return Err(bad("query_by required"));
        };
        repo::delete_permission_groups(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

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

pub struct PermissionServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::permission_service_server::PermissionService for PermissionServiceImpl {
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
        let row = repo::permissions_by_id(&self.state.db, id as i64).await?;
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
                repo::delete_permissions(&self.state.db, id as i64).await?;
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
        sync::sync_permissions(&self.state.db, req.operator_id).await?;
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
            let limit = identity_repo::permission_ids_by_role_ids(&self.state.db, &role_ids)
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

// ── sync (the api registry face) ────────────────────────────────────

/// The sync variant of the api service: the plain CRUD delegates to
/// [`ApiServiceImpl`]; `sync_apis` upserts the BFF's shipped route
/// table by operation id.
pub struct ApiSyncServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::api_service_server::ApiService for ApiSyncServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListApiResponse>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .list(request)
        .await
    }

    async fn count(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountApiResponse>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .count(request)
        .await
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetApiRequest>,
    ) -> Result<Response<permissionv1::Api>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .get(request)
        .await
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .create(request)
        .await
    }

    async fn update(
        &self,
        request: Request<permissionv1::UpdateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .update(request)
        .await
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        ApiServiceImpl {
            state: Arc::clone(&self.state),
        }
        .delete(request)
        .await
    }

    async fn sync_apis(
        &self,
        request: Request<permissionv1::SyncApisRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        // Upsert by operation: insert missing, refresh path/method of
        // existing (the BFF ships the route table it generated).
        let now = store::now();
        for api in req.apis {
            match repo::apis_by_operation(
                &self.state.db,
                &api.operation.clone().unwrap_or_default(),
            )
            .await?
            {
                Some(row) => {
                    let mut a: sys_apis::ActiveModel = row.into();
                    a.path = Set(api.path.clone());
                    a.method = Set(api.method.clone());
                    a.updated_at = Set(Some(now));
                    repo::update_apis(&self.state.db, a).await?;
                }
                None => {
                    repo::insert_apis(
                        &self.state.db,
                        sys_apis::ActiveModel {
                            operation: Set(Some(api.operation.clone().unwrap_or_default())),
                            path: Set(Some(api.path.clone().unwrap_or_default())),
                            method: Set(Some(api.method.clone().unwrap_or_default())),
                            module: Set(api.module.clone()),
                            description: Set(api.description.clone()),
                            status: Set("ON".to_string()),
                            tenant_id: Set(Some(0)),
                            created_at: Set(Some(now)),
                            updated_at: Set(Some(now)),
                            ..Default::default()
                        },
                    )
                    .await?;
                }
            }
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── the permission rebuild (SyncPermissions) ────────────────────────

mod sync {
    //! The derived-permission rebuild: the enabled menu set becomes
    //! groups + permissions, the api registry appends its own
    //! resource:action codes, and everything rebuilds wholesale —
    //! truncate (biz permissions + biz groups) → batch-create groups →
    //! backfill group ids → batch-create permissions → re-assign the
    //! derived api/menu tables. The menu→code conversion and the group
    //! construction walk the reference's converters line by line.

    use std::collections::{HashMap, HashSet};

    use sea_orm::{DatabaseConnection, Set};
    use tonic::Status;

    use crate::data::permission_repo as repo;
    use crate::state::bad;
    use store::entities::{sys_menus, sys_permission_groups, sys_permissions};

    /// constants.UncategorizedPermissionGroup
    const UNCATEGORIZED: &str = "uncategorized";
    /// constants.DefaultBizPermissionModule
    const DEFAULT_MODULE: &str = "biz";

    struct PermDraft {
        name: String,
        code: String,
        menu_ids: Vec<i64>,
        api_ids: Vec<i64>,
        group_id: Option<i64>,
    }

    /// module → perm 下标（保持插入序，供组回填与前缀归属判定）
    fn map_entry<'a>(map: &'a mut Vec<(String, Vec<usize>)>, module: &str) -> &'a mut Vec<usize> {
        if let Some(pos) = map.iter().position(|(m, _)| m == module) {
            return &mut map[pos].1;
        }
        map.push((module.to_string(), Vec::new()));
        &mut map.last_mut().expect("just pushed").1
    }

    pub(super) async fn sync_permissions(
        db: &DatabaseConnection,
        operator_id: Option<u32>,
    ) -> Result<(), Status> {
        let operator = operator_id.map(|v| v as i64);

        // 输入：启用菜单集（id 降序基序 → 稳定按 parent_id 升序）
        let mut menus = repo::menus_enabled_desc(db).await?;
        compose_menu_paths(&mut menus);
        menus.sort_by_key(|m| m.parent_id.unwrap_or(0));

        // 未分类组打头；目录菜单升级为权限组；每个菜单落一个权限
        let mut groups: Vec<(String, String, i64)> = Vec::new(); // (name, module, sort_order)
        groups.push(("未分类".to_string(), UNCATEGORIZED.to_string(), 1));
        let mut perms: Vec<PermDraft> = Vec::new();
        let mut map_permissions: Vec<(String, Vec<usize>)> = Vec::new();

        for menu in &menus {
            let title = menu.name.clone().unwrap_or_default();
            let full_path = menu.path.clone().unwrap_or_default();
            let menu_type = menu.r#type.as_deref().unwrap_or("MENU");
            let Some(code) = convert_menu_code(&full_path, &title, menu_type) else {
                continue;
            };
            let module = menu_path_to_module_name(&full_path);

            // 以目录类型的菜单作为权限组
            if menu_type == "CATALOG" {
                groups.push((title.clone(), module.clone(), groups.len() as i64 + 1));
            }

            perms.push(PermDraft {
                name: title,
                code,
                menu_ids: vec![menu.id],
                api_ids: Vec::new(),
                group_id: None,
            });
            map_entry(&mut map_permissions, &module).push(perms.len() - 1);
        }

        // 清理菜单相关权限——失败必须返回错误，否则后续批量创建会产生重复数据
        repo::truncate_biz_permissions(db).await?;
        repo::truncate_biz_groups(db).await?;

        // 为权限追加对应的 API 资源 ID 列表
        append_apis(db, &mut perms, &mut map_permissions).await?;

        // 批量建组
        if groups.is_empty() {
            return Err(bad("invalid parameter"));
        }
        let group_rows = repo::insert_permission_groups_bulk(
            db,
            groups
                .into_iter()
                .map(
                    |(name, module, sort_order)| sys_permission_groups::ActiveModel {
                        name: Set(name),
                        module: Set(Some(module)),
                        status: Set("ON".to_string()),
                        sort_order: Set(Some(sort_order)),
                        created_by: Set(operator),
                        created_at: Set(Some(store::now())),
                        ..Default::default()
                    },
                )
                .collect(),
        )
        .await?;

        // 为权限分配权限组 ID
        for pg in &group_rows {
            let module = pg.module.as_deref().unwrap_or_default();
            if let Some((_, idxs)) = map_permissions.iter().find(|(m, _)| m == module) {
                for &i in idxs {
                    perms[i].group_id = Some(pg.id);
                }
            }
        }

        // 批量建权限 + 派生表重建
        if perms.is_empty() {
            return Err(bad("invalid parameter"));
        }
        let perm_rows = repo::insert_permissions_bulk(
            db,
            perms
                .iter()
                .map(|p| sys_permissions::ActiveModel {
                    name: Set(p.name.clone()),
                    code: Set(p.code.clone()),
                    status: Set("ON".to_string()),
                    group_id: Set(p.group_id),
                    tenant_id: Set(Some(0)),
                    created_by: Set(operator),
                    created_at: Set(Some(store::now())),
                    ..Default::default()
                })
                .collect(),
        )
        .await?;

        for (p, row) in perms.iter().zip(&perm_rows) {
            if !p.api_ids.is_empty() {
                repo::assign_permission_apis(db, row.id, &p.api_ids).await?;
            }
            if !p.menu_ids.is_empty() {
                repo::assign_permission_menus(db, row.id, &p.menu_ids).await?;
            }
        }

        Ok(())
    }

    /// appendAPis：启用 API 按 module/path/operation 排序后折叠为
    /// resource:action code；既有权限按 code 吸收 API，剩余 code 生成
    /// 新权限（挂到 code 前缀命中的模块，否则未分类）。
    async fn append_apis(
        db: &DatabaseConnection,
        perms: &mut Vec<PermDraft>,
        map_permissions: &mut Vec<(String, Vec<usize>)>,
    ) -> Result<(), Status> {
        let mut apis = repo::apis_enabled_by_operation(db).await?;
        apis.sort_by(|a, b| {
            a.module
                .as_deref()
                .unwrap_or_default()
                .cmp(b.module.as_deref().unwrap_or_default())
                .then_with(|| {
                    b.path
                        .as_deref()
                        .unwrap_or_default()
                        .cmp(a.path.as_deref().unwrap_or_default())
                })
                .then_with(|| {
                    b.operation
                        .as_deref()
                        .unwrap_or_default()
                        .cmp(a.operation.as_deref().unwrap_or_default())
                })
        });

        struct ModuleApis {
            module: String,
            apis: Vec<i64>,
        }
        let mut codes: Vec<(String, ModuleApis)> = Vec::new();

        for api in &apis {
            let code = convert_api_code_by_path(
                api.method.as_deref().unwrap_or_default(),
                api.path.as_deref().unwrap_or_default(),
            );
            if code.is_empty() {
                continue;
            }
            if let Some((_, entry)) = codes.iter_mut().find(|(c, _)| *c == code) {
                entry.apis.push(api.id);
                continue;
            }
            let mut module = String::new();
            'outer: for (m, idxs) in map_permissions.iter() {
                for &i in idxs {
                    let code1_prefix = perms[i].code.split(':').next().unwrap_or_default();
                    let code2_prefix = code.split(':').next().unwrap_or_default();
                    if code2_prefix.starts_with(code1_prefix) {
                        module = m.clone();
                        break 'outer;
                    }
                }
            }
            if module.is_empty() {
                module = UNCATEGORIZED.to_string();
            }
            codes.push((
                code,
                ModuleApis {
                    module,
                    apis: vec![api.id],
                },
            ));
        }

        for p in perms.iter_mut() {
            if let Some(pos) = codes.iter().position(|(c, _)| *c == p.code) {
                let (_, entry) = codes.remove(pos);
                p.api_ids.extend(entry.apis);
            }
        }

        for (code, entry) in codes {
            let name = pascal_case(&code.replace(':', "_"));
            perms.push(PermDraft {
                name,
                code,
                menu_ids: Vec::new(),
                api_ids: entry.apis,
                group_id: None,
            });
            map_entry(map_permissions, &entry.module).push(perms.len() - 1);
        }

        Ok(())
    }

    // ── MenuPermissionConverter ─────────────────────────────────────

    /// ComposeMenuPaths：递归拼接每个菜单的完整 path 写回（含环检测、
    /// 父节点缺失回退）。
    fn compose_menu_paths(menus: &mut [sys_menus::Model]) {
        let lookup: HashMap<i64, (i64, String)> = menus
            .iter()
            .map(|m| {
                (
                    m.id,
                    (m.parent_id.unwrap_or(0), m.path.clone().unwrap_or_default()),
                )
            })
            .collect();
        let mut memo: HashMap<i64, String> = HashMap::new();
        for m in menus.iter_mut() {
            let mut seen = HashSet::new();
            let full = compute_menu_path(m.id, &lookup, &mut memo, &mut seen);
            m.path = Some(full);
        }
    }

    fn compute_menu_path(
        id: i64,
        lookup: &HashMap<i64, (i64, String)>,
        memo: &mut HashMap<i64, String>,
        seen: &mut HashSet<i64>,
    ) -> String {
        if let Some(v) = memo.get(&id) {
            return v.clone();
        }
        let Some(&(parent_id, ref part)) = lookup.get(&id) else {
            memo.insert(id, String::new());
            return String::new();
        };
        if seen.contains(&id) {
            // 环：仅使用自身 path
            let v = part.trim_matches('/').to_string();
            memo.insert(id, v.clone());
            return v;
        }
        if parent_id == 0 || parent_id == id {
            memo.insert(id, part.clone());
            return part.clone();
        }
        if !lookup.contains_key(&parent_id) {
            memo.insert(id, part.clone());
            return part.clone();
        }
        seen.insert(id);
        let parent_full = compute_menu_path(parent_id, lookup, memo, seen);
        seen.remove(&id);
        let full = if parent_full.is_empty() {
            part.clone()
        } else if part.is_empty() {
            parent_full
        } else {
            format!("{parent_full}/{part}")
        };
        memo.insert(id, full.clone());
        full
    }

    /// ConvertCode：完整 path + 类型 → 权限代码。
    fn convert_menu_code(path: &str, title: &str, menu_type: &str) -> Option<String> {
        let path = path.trim();
        if path.is_empty() {
            return None;
        }
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            return None;
        }
        let segments: Vec<&str> = trimmed.split('/').collect();
        let segments = if segments.len() > 1 {
            &segments[1..]
        } else {
            &segments[..]
        };
        let mut new_paths: Vec<String> = Vec::new();
        for seg in segments {
            let seg = singular_word(seg.trim());
            if seg.is_empty() || seg.starts_with(':') {
                continue;
            }
            new_paths.push(seg);
        }
        let perm_base = new_paths.join(":");
        match menu_type_to_action(title, menu_type) {
            Some(action) => Some(format!("{perm_base}:{action}")),
            None => Some(perm_base),
        }
    }

    /// typeToAction：菜单类型 → 动作后缀（未知类型无后缀）。
    fn menu_type_to_action(title: &str, menu_type: &str) -> Option<String> {
        match menu_type {
            "CATALOG" => Some("dir".to_string()),
            "MENU" => Some("view".to_string()),
            "BUTTON" => Some(button_action(title)),
            "EMBEDDED" => Some("view".to_string()),
            "LINK" => Some("jump".to_string()),
            _ => None,
        }
    }

    /// buttonAction：按钮标题关键词 → 动作。
    fn button_action(title: &str) -> String {
        let title = title.trim();
        if title.is_empty() {
            return "act".to_string();
        }
        let lower = title.to_lowercase();
        const ADD: &[&str] = &[
            "add", "addto", "add+", "create", "new", "plus", "append", "新增", "添加", "创建",
        ];
        const EDIT: &[&str] = &[
            "edit", "update", "modify", "save", "patch", "保存", "修改", "更新", "编辑",
        ];
        const DELETE: &[&str] = &[
            "delete", "del", "remove", "destroy", "drop", "discard", "trash", "删除", "移除",
            "弃用", "清除",
        ];
        const EXPORT: &[&str] = &[
            "export",
            "download",
            "exportcsv",
            "exportexcel",
            "导出",
            "下载",
            "导出为",
        ];
        const IMPORT: &[&str] = &["import", "importcsv", "importexcel", "导入", "导入为"];
        if match_any_keyword(&lower, ADD) {
            return "create".into();
        }
        if match_any_keyword(&lower, EDIT) {
            return "edit".into();
        }
        if match_any_keyword(&lower, DELETE) {
            return "delete".into();
        }
        if match_any_keyword(&lower, IMPORT) {
            return "import".into();
        }
        if match_any_keyword(&lower, EXPORT) {
            return "export".into();
        }
        "act".into()
    }

    /// matchAnyKeyword：先按 token 精确/前缀匹配，再回退 substring。
    fn match_any_keyword(title: &str, keys: &[&str]) -> bool {
        let tokens = tokenize(title);
        for k in keys {
            for tk in &tokens {
                if tk == k || tk.starts_with(k) {
                    return true;
                }
            }
        }
        keys.iter().any(|k| title.contains(k))
    }

    /// tokenize：按非字母数字分段并小写。
    fn tokenize(s: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut buf = String::new();
        for ch in s.chars() {
            if ch.is_alphanumeric() {
                buf.extend(ch.to_lowercase());
            } else if !buf.is_empty() {
                out.push(std::mem::take(&mut buf));
            }
        }
        if !buf.is_empty() {
            out.push(buf);
        }
        out
    }

    /// MenuPathToModuleName：完整 path 第二段 → 模块名。
    fn menu_path_to_module_name(menu_path: &str) -> String {
        let parts: Vec<&str> = menu_path.split('/').collect();
        let mut module = String::new();
        if parts.len() > 1 {
            module = parts[1].trim().to_string();
        }
        if module.is_empty() {
            module = DEFAULT_MODULE.to_string();
        }
        module
    }

    // ── ApiPermissionConverter ──────────────────────────────────────

    /// ConvertCodeByPath：HTTP 方法 + 路径 → resource:action。
    fn convert_api_code_by_path(method: &str, path: &str) -> String {
        let resource = api_path_to_resource(path);
        let action = api_method_to_action(method, path);
        format!("{resource}:{action}")
    }

    fn api_method_to_action(method: &str, path: &str) -> String {
        if path.ends_with("/list") {
            return "view".to_string();
        }
        match method.to_ascii_uppercase().as_str() {
            "GET" => "view".into(),
            "POST" => "create".into(),
            "PUT" | "PATCH" => "edit".into(),
            "DELETE" => "delete".into(),
            _ => method.to_lowercase(),
        }
    }

    fn api_path_to_resource(path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        let stripped = strip_version_prefix(path);
        let cleaned = remove_path_params(&stripped);
        if cleaned.is_empty() {
            return String::new();
        }
        let parts: Vec<&str> = cleaned.split('/').collect();
        let mut segs: Vec<String> = Vec::new();
        if let Some(first) = parts.first() {
            let raw = first.trim();
            if raw.is_empty() {
                return String::new();
            }
            let singularized = singularize_segments(raw);
            if let Some(head) = singularized.split(':').next() {
                segs.push(head.to_string());
            }
        }
        segs.join(":")
    }

    /// ^v[0-9]+(\.[0-9]+)?$（大小写不敏感）
    fn is_version_segment(seg: &str) -> bool {
        let rest = match seg.strip_prefix('v').or_else(|| seg.strip_prefix('V')) {
            Some(r) => r,
            None => return false,
        };
        let (major, minor) = match rest.split_once('.') {
            Some((a, b)) => (a, Some(b)),
            None => (rest, None),
        };
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        digits(major) && minor.map(digits).unwrap_or(true)
    }

    /// stripVersionPrefix：移除路径开头的 /api/ 与 /vN/。
    fn strip_version_prefix(p: &str) -> String {
        if p.trim().is_empty() {
            return String::new();
        }
        let trimmed = p.trim_matches('/');
        if trimmed.is_empty() {
            return String::new();
        }
        let mut parts: Vec<&str> = trimmed.split('/').collect();
        if parts.first().is_some_and(|s| s.eq_ignore_ascii_case("api")) {
            parts.remove(0);
        }
        if let Some(idx) = parts.iter().position(|s| is_version_segment(s)) {
            if idx <= 1 {
                parts.drain(..=idx);
            }
        }
        parts
            .into_iter()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("api"))
            .collect::<Vec<_>>()
            .join("/")
    }

    /// ^\{\s*[^/{}]+\s*}$
    fn is_param_segment(seg: &str) -> bool {
        if !(seg.len() >= 2 && seg.starts_with('{') && seg.ends_with('}')) {
            return false;
        }
        let inner = seg[1..seg.len() - 1].trim();
        !inner.is_empty() && !inner.contains(['/', '{', '}'])
    }

    /// removePathParams：丢弃 {id} 形态的参数段。
    fn remove_path_params(p: &str) -> String {
        if p.trim().is_empty() {
            return String::new();
        }
        let trimmed = p.trim_matches('/');
        if trimmed.is_empty() {
            return String::new();
        }
        trimmed
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty() && !is_param_segment(s))
            .collect::<Vec<_>>()
            .join("/")
    }

    /// singularizeSegments：按非字母数字分段，逐段单数化后按原分隔符拼回。
    fn singularize_segments(s: &str) -> String {
        if s.trim().is_empty() {
            return s.to_string();
        }
        let mut out = String::with_capacity(s.len());
        let mut token = String::new();
        for ch in s.chars() {
            if ch.is_alphanumeric() {
                token.push(ch);
            } else {
                flush_token(&mut token, &mut out);
                out.push(ch);
            }
        }
        flush_token(&mut token, &mut out);
        out
    }

    fn flush_token(token: &mut String, out: &mut String) {
        if !token.is_empty() {
            out.push_str(&singular_word(token));
            token.clear();
        }
    }

    // ── stringcase.ToPascalCase ─────────────────────────────────────

    /// stringcase.Split：按非字母数字分段后按驼峰边界再拆。
    fn split_words(input: &str) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        for token in input.split(|c: char| !c.is_alphanumeric()) {
            if token.is_empty() {
                continue;
            }
            words.extend(split_camel_word(token));
        }
        words
    }

    /// 驼峰词界读取器（readNumber / readWord 的直译）。
    fn split_camel_word(token: &str) -> Vec<String> {
        let chars: Vec<char> = token.chars().collect();
        let is_upper = |c: char| c.is_uppercase();
        let is_digit = |c: char| c.is_ascii_digit();
        let mut out: Vec<String> = Vec::new();
        let mut pos = 0usize;
        while pos < chars.len() {
            let start = pos;
            pos += 1; // readRune
            if is_digit(chars[start]) {
                if pos < chars.len() && is_digit(chars[pos]) {
                    while pos < chars.len() && is_digit(chars[pos]) {
                        pos += 1;
                    }
                }
            } else if pos < chars.len() && is_upper(chars[pos]) {
                while pos < chars.len() && is_upper(chars[pos]) {
                    pos += 1;
                }
                if pos < chars.len() && !is_upper(chars[pos]) && !is_digit(chars[pos]) {
                    pos -= 1; // unreadRune
                }
            } else {
                while pos < chars.len() && !is_upper(chars[pos]) && !is_digit(chars[pos]) {
                    pos += 1;
                }
            }
            out.push(chars[start..pos].iter().collect());
        }
        out
    }

    /// stringcase.ToPascalCase。
    fn pascal_case(input: &str) -> String {
        let words = split_words(input.trim());
        let mut out = String::new();
        for w in words {
            let mut cs = w.chars();
            if let Some(f) = cs.next() {
                out.extend(f.to_uppercase());
                out.extend(cs.flat_map(|c| c.to_lowercase()));
            }
        }
        out
    }

    // ── inflection.Singular ─────────────────────────────────────────

    #[derive(Clone, Copy, PartialEq)]
    enum CaseMode {
        /// 大写变体（UPPER 形式命中，替换字面量同样大写）
        Upper,
        /// 原样
        Exact,
        /// 大小写不敏感
        Fold,
    }

    fn title_word(s: &str) -> String {
        let mut cs = s.chars();
        match cs.next() {
            Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
            None => String::new(),
        }
    }

    /// 一条常规规则：`[start_anchor] g1 lit [g2(required?)] lit2 [end_anchor]`，
    /// 命中段替换为「组1捕获 + repl」。
    struct SRule {
        start_anchor: bool,
        g1: &'static [Alt],
        lit: &'static str,
        g2: Option<(&'static [&'static str], bool)>,
        lit2: &'static str,
        end_anchor: bool,
        repl: &'static str,
    }

    enum Alt {
        Lit(&'static str),
        CharsAny(&'static str), // [..]
        CharsNot(&'static str), // [^..]
    }

    use Alt::Lit as A;

    /// 组/字面量在 pos 处的一次匹配，返回消耗的字节长度。
    fn alt_match_len(word: &str, pos: usize, alt: &Alt, mode: CaseMode) -> Option<usize> {
        if !word.is_char_boundary(pos) {
            return None;
        }
        let rest = &word[pos..];
        match alt {
            Alt::Lit(lit) => {
                let hit = match mode {
                    CaseMode::Upper => rest.starts_with(&(*lit).to_ascii_uppercase()),
                    CaseMode::Exact => rest.starts_with(lit),
                    CaseMode::Fold => {
                        rest.len() >= lit.len() && rest[..lit.len()].eq_ignore_ascii_case(lit)
                    }
                };
                hit.then_some(lit.len())
            }
            Alt::CharsAny(set) => {
                let c = rest.chars().next()?;
                let hit = match mode {
                    CaseMode::Upper => (*set).to_ascii_uppercase().contains(c),
                    CaseMode::Exact => (*set).contains(c),
                    CaseMode::Fold => (*set).contains(c) || (*set).to_ascii_uppercase().contains(c),
                };
                hit.then_some(c.len_utf8())
            }
            Alt::CharsNot(set) => {
                let c = rest.chars().next()?;
                let in_set = match mode {
                    CaseMode::Upper => (*set).to_ascii_uppercase().contains(c),
                    CaseMode::Exact => (*set).contains(c),
                    CaseMode::Fold => (*set).contains(c) || (*set).to_ascii_uppercase().contains(c),
                };
                (!in_set).then_some(c.len_utf8())
            }
        }
    }

    fn match_lit(word: &str, pos: usize, lit: &str, mode: CaseMode) -> Option<usize> {
        if lit.is_empty() {
            return Some(0);
        }
        if !word.is_char_boundary(pos) {
            return None;
        }
        let rest = &word[pos..];
        let hit = match mode {
            CaseMode::Upper => rest.starts_with(&lit.to_ascii_uppercase()),
            CaseMode::Exact => rest.starts_with(lit),
            CaseMode::Fold => {
                rest.len() >= lit.len() && rest[..lit.len()].eq_ignore_ascii_case(lit)
            }
        };
        hit.then_some(lit.len())
    }

    fn rule_try(word: &str, mode: CaseMode, r: &SRule) -> Option<String> {
        // 左端优先：从最左起点尝试组1的每个选项（^ 锚定时仅起点 0）
        let scan_to = if r.start_anchor { 0 } else { word.len() };
        let mut start = 0usize;
        while start <= scan_to {
            if !word.is_char_boundary(start) {
                start += 1;
                continue;
            }
            for alt in r.g1 {
                let Some(g1_len) = alt_match_len(word, start, alt, mode) else {
                    continue;
                };
                let mut pos = start + g1_len;
                let Some(len) = match_lit(word, pos, r.lit, mode) else {
                    continue;
                };
                pos += len;
                if let Some((alts, required)) = r.g2 {
                    let mut hit = None;
                    for a in alts {
                        if let Some(len) = match_lit(word, pos, a, mode) {
                            hit = Some(len);
                            break;
                        }
                    }
                    match hit {
                        Some(len) => pos += len,
                        None if required => continue,
                        None => {}
                    }
                }
                let Some(len) = match_lit(word, pos, r.lit2, mode) else {
                    continue;
                };
                pos += len;
                if r.end_anchor && pos != word.len() {
                    continue;
                }
                let g1_text = &word[start..start + g1_len];
                let repl = match mode {
                    CaseMode::Upper => r.repl.to_ascii_uppercase(),
                    _ => r.repl.to_string(),
                };
                return Some(format!("{}{g1_text}{repl}{}", &word[..start], &word[pos..]));
            }
            start += 1;
        }
        None
    }

    /// ^(ox)en —— 前缀替换（无尾锚）。
    fn rule_oxen(word: &str, mode: CaseMode) -> Option<String> {
        let head = word.get(..4)?;
        let hit = match mode {
            CaseMode::Upper => head == "OXEN",
            CaseMode::Exact => head == "oxen",
            CaseMode::Fold => head.eq_ignore_ascii_case("oxen"),
        };
        if !hit {
            return None;
        }
        Some(format!("{}{}", &word[..2], &word[4..]))
    }

    /// ^(a)x[ie]s$ —— 整串匹配。
    fn rule_axis(word: &str, mode: CaseMode) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() != 4 {
            return None;
        }
        let hit = |c: char, set: &str| match mode {
            CaseMode::Upper => (*set).to_ascii_uppercase().contains(c),
            CaseMode::Exact => (*set).contains(c),
            CaseMode::Fold => (*set).contains(c) || (*set).to_ascii_uppercase().contains(c),
        };
        if !hit(chars[0], "a") || !hit(chars[1], "x") || !hit(chars[2], "ie") || !hit(chars[3], "s")
        {
            return None;
        }
        let repl = match mode {
            CaseMode::Upper => "XIS",
            _ => "xis",
        };
        Some(format!("{}{repl}", chars[0]))
    }

    /// ([^aeiouy]|qu)ies$ —— 双分支组 + 尾锚。
    fn rule_not_vowel_qu_ies(word: &str, mode: CaseMode) -> Option<String> {
        let len = word.len();
        let tail = |lit: &str| match mode {
            CaseMode::Upper => lit.to_ascii_uppercase(),
            _ => lit.to_string(),
        };
        let excluded = match mode {
            CaseMode::Upper => "AEIOUY",
            CaseMode::Exact => "aeiouy",
            CaseMode::Fold => "aeiouyAEIOUY",
        };
        // 5 字符段（g1 = "qu"）起点更靠左，先试
        if len >= 5 {
            if let (Some(g1), Some(t)) = (word.get(len - 5..len - 3), word.get(len - 3..)) {
                let g1_hit = match mode {
                    CaseMode::Upper => g1 == "QU",
                    CaseMode::Exact => g1 == "qu",
                    CaseMode::Fold => g1.eq_ignore_ascii_case("qu"),
                };
                if g1_hit && t == tail("ies") {
                    return Some(format!("{}{g1}y", &word[..len - 5]));
                }
            }
        }
        // 4 字符段：g1 = 单个非元音字符
        if len >= 4 {
            if let (Some(g1), Some(t)) = (word.get(len - 4..len - 3), word.get(len - 3..)) {
                if let Some(c) = g1.chars().next() {
                    if g1.chars().count() == 1 && !excluded.contains(c) && t == tail("ies") {
                        let repl = match mode {
                            CaseMode::Upper => "Y",
                            _ => "y",
                        };
                        return Some(format!("{}{g1}{repl}", &word[..len - 4]));
                    }
                }
            }
        }
        None
    }

    const BARE: SRule = SRule {
        start_anchor: false,
        g1: &[],
        lit: "",
        g2: None,
        lit2: "",
        end_anchor: true,
        repl: "",
    };

    /// 编译序的常规规则（声明倒序 × Upper/Exact/Fold）。
    fn regular_rules() -> Vec<(&'static str, SRule)> {
        vec![
            (
                "db",
                SRule {
                    g1: &[A("database")],
                    lit: "s",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "quiz",
                SRule {
                    g1: &[A("quiz")],
                    lit: "zes",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "matr",
                SRule {
                    g1: &[A("matr")],
                    lit: "ices",
                    repl: "ix",
                    ..BARE
                },
            ),
            (
                "vertind",
                SRule {
                    g1: &[A("vert"), A("ind")],
                    lit: "ices",
                    repl: "ex",
                    ..BARE
                },
            ),
            (
                "alias",
                SRule {
                    g1: &[A("alias"), A("status")],
                    g2: Some((&["es"], false)),
                    repl: "",
                    ..BARE
                },
            ),
            (
                "octop",
                SRule {
                    g1: &[A("octop"), A("vir")],
                    g2: Some((&["us", "i"], true)),
                    repl: "us",
                    ..BARE
                },
            ),
            (
                "cristest",
                SRule {
                    g1: &[A("cris"), A("test")],
                    g2: Some((&["is", "es"], true)),
                    repl: "is",
                    ..BARE
                },
            ),
            (
                "shoe",
                SRule {
                    g1: &[A("shoe")],
                    lit: "s",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "oes",
                SRule {
                    g1: &[A("o")],
                    lit: "es",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "bus",
                SRule {
                    g1: &[A("bus")],
                    g2: Some((&["es"], false)),
                    repl: "",
                    ..BARE
                },
            ),
            (
                "mlouse",
                SRule {
                    start_anchor: true,
                    g1: &[A("m"), A("l")],
                    lit: "ice",
                    repl: "ouse",
                    ..BARE
                },
            ),
            (
                "xchsssh",
                SRule {
                    g1: &[A("x"), A("ch"), A("ss"), A("sh")],
                    lit: "es",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "cookies",
                SRule {
                    g1: &[A("c")],
                    lit: "ookies",
                    repl: "ookie",
                    ..BARE
                },
            ),
            (
                "movies",
                SRule {
                    g1: &[A("m")],
                    lit: "ovies",
                    repl: "ovie",
                    ..BARE
                },
            ),
            (
                "series",
                SRule {
                    g1: &[A("s")],
                    lit: "eries",
                    repl: "eries",
                    ..BARE
                },
            ),
            (
                "lrves",
                SRule {
                    g1: &[A("l"), A("r")],
                    lit: "ves",
                    repl: "f",
                    ..BARE
                },
            ),
            (
                "tive",
                SRule {
                    g1: &[A("tive")],
                    lit: "s",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "hive",
                SRule {
                    g1: &[A("hive")],
                    lit: "s",
                    repl: "",
                    ..BARE
                },
            ),
            (
                "notfves",
                SRule {
                    g1: &[Alt::CharsNot("f")],
                    lit: "ves",
                    repl: "fe",
                    ..BARE
                },
            ),
            (
                "analy",
                SRule {
                    start_anchor: true,
                    g1: &[A("analy")],
                    g2: Some((&["sis", "ses"], true)),
                    repl: "sis",
                    ..BARE
                },
            ),
            (
                "analygroup",
                SRule {
                    g1: &[
                        A("analy"),
                        A("ba"),
                        A("diagno"),
                        A("parenthe"),
                        A("progno"),
                        A("synop"),
                        A("the"),
                    ],
                    g2: Some((&["sis", "ses"], true)),
                    repl: "sis",
                    ..BARE
                },
            ),
            (
                "tia",
                SRule {
                    g1: &[Alt::CharsAny("ti")],
                    lit: "a",
                    repl: "um",
                    ..BARE
                },
            ),
            (
                "news",
                SRule {
                    g1: &[A("n")],
                    lit: "ews",
                    repl: "ews",
                    ..BARE
                },
            ),
            (
                "ss",
                SRule {
                    g1: &[A("ss")],
                    repl: "",
                    ..BARE
                },
            ),
            (
                "s",
                SRule {
                    g1: &[A("")],
                    lit: "s",
                    repl: "",
                    ..BARE
                },
            ),
        ]
    }

    /// inflection.Singular：uncountable → irregular → 常规规则表
    /// （Upper/Exact/Fold 三变体依次尝试，首个命中即返回）。
    fn singular_word(word: &str) -> String {
        const UNCOUNTABLE: &[&str] = &[
            "equipment",
            "information",
            "rice",
            "money",
            "species",
            "series",
            "fish",
            "sheep",
            "jeans",
            "police",
        ];
        if UNCOUNTABLE.iter().any(|u| word.eq_ignore_ascii_case(u)) {
            return word.to_string();
        }

        const IRREGULAR: &[(&str, &str)] = &[
            ("person", "people"),
            ("man", "men"),
            ("child", "children"),
            ("sex", "sexes"),
            ("move", "moves"),
            ("mombie", "mombies"),
        ];
        for (sing, plur) in IRREGULAR {
            let variants = [
                ((*plur).to_ascii_uppercase(), (*sing).to_ascii_uppercase()),
                (title_word(plur), title_word(sing)),
                ((*plur).to_string(), (*sing).to_string()),
            ];
            for (p, s) in variants {
                if let Some(rest) = word.strip_suffix(p.as_str()) {
                    return format!("{rest}{s}");
                }
            }
        }

        for (name, rule) in regular_rules() {
            for mode in [CaseMode::Upper, CaseMode::Exact, CaseMode::Fold] {
                if let Some(out) = rule_try(word, mode, &rule) {
                    return out;
                }
            }
            // 三个特例规则穿插在编译序里
            match name {
                "vertind" => {
                    // ^(ox)en 在 (vert|ind)ices 与 (alias|status) 之间
                    for mode in [CaseMode::Upper, CaseMode::Exact, CaseMode::Fold] {
                        if let Some(out) = rule_oxen(word, mode) {
                            return out;
                        }
                    }
                }
                "octop" => {
                    // ^(a)x[ie]s$ 在 (octop|vir) 与 (cris|test) 之间
                    for mode in [CaseMode::Upper, CaseMode::Exact, CaseMode::Fold] {
                        if let Some(out) = rule_axis(word, mode) {
                            return out;
                        }
                    }
                }
                "series" => {
                    // ([^aeiouy]|qu)ies$ 在 (s)eries 与 ([lr])ves 之间
                    for mode in [CaseMode::Upper, CaseMode::Exact, CaseMode::Fold] {
                        if let Some(out) = rule_not_vowel_qu_ies(word, mode) {
                            return out;
                        }
                    }
                }
                _ => {}
            }
        }

        word.to_string()
    }

    // The converter tests live inside `mod sync` so the private pure
    // helpers stay private — the module is this file's tail.
    #[cfg(test)]
    mod tests {
        use super::*;

        /// A minimal menu row carrying only the path-composition fields.
        fn menu(id: i64, parent_id: i64, path: &str) -> sys_menus::Model {
            sys_menus::Model {
                id,
                created_at: None,
                updated_at: None,
                deleted_at: None,
                created_by: None,
                updated_by: None,
                deleted_by: None,
                remark: None,
                status: "ON".to_string(),
                tenant_id: Some(0),
                r#type: Some("MENU".to_string()),
                path: Some(path.to_string()),
                redirect: None,
                alias: None,
                name: None,
                component: None,
                meta: None,
                parent_id: Some(parent_id),
            }
        }

        fn paths_of(menus: &[sys_menus::Model]) -> Vec<String> {
            menus
                .iter()
                .map(|m| m.path.clone().unwrap_or_default())
                .collect()
        }

        // ── ComposeMenuPaths ────────────────────────────────────────

        #[test]
        fn compose_menu_paths_joins_the_ancestor_chain() {
            let mut menus = vec![menu(1, 0, "admin"), menu(2, 1, "users"), menu(3, 2, "list")];
            compose_menu_paths(&mut menus);
            assert_eq!(
                paths_of(&menus),
                vec!["admin", "admin/users", "admin/users/list"]
            );
        }

        #[test]
        fn compose_menu_paths_keeps_root_part_verbatim() {
            let mut menus = vec![menu(1, 0, "/admin/")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["/admin/"]);
        }

        #[test]
        fn compose_menu_paths_falls_back_to_own_part_without_parent() {
            let mut menus = vec![menu(7, 99, "orphan")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["orphan"]);

            // Self-parent counts as a root.
            let mut menus = vec![menu(5, 5, "solo")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["solo"]);
        }

        #[test]
        fn compose_menu_paths_skips_empty_parts() {
            // Parent's empty path contributes nothing.
            let mut menus = vec![menu(1, 0, ""), menu(2, 1, "child")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["", "child"]);

            // A child's empty part inherits the parent path whole.
            let mut menus = vec![menu(1, 0, "p"), menu(2, 1, "")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["p", "p"]);
        }

        #[test]
        fn compose_menu_paths_breaks_cycles_on_the_seen_set() {
            // 1 → 2 → 1: the cycle detection returns the own part for
            // the re-entered node, so id 1 composes onto it once more.
            let mut menus = vec![menu(1, 2, "a"), menu(2, 1, "b")];
            compose_menu_paths(&mut menus);
            assert_eq!(paths_of(&menus), vec!["a/b/a", "a/b"]);
        }

        // ── ConvertCode ─────────────────────────────────────────────
        // Mirrors TestMenuPermissionConverter_ConvertCode
        // (go-wind-cms pkg/utils/converter/menu_test.go).

        #[test]
        fn convert_menu_code_matches_reference_table() {
            let table = [
                ("catalog dir", "/users", "", "CATALOG", "user:dir"),
                ("menu view", "/orders/", "", "MENU", "order:view"),
                ("embedded view", "/foo", "", "EMBEDDED", "foo:view"),
                ("link jump", "/admin/settings", "", "LINK", "setting:jump"),
                (
                    "button default act",
                    "admin/button",
                    "",
                    "BUTTON",
                    "button:act",
                ),
                (
                    "button create (中文)",
                    "/users",
                    "新增",
                    "BUTTON",
                    "user:create",
                ),
                (
                    "button export (中文)",
                    "/reports",
                    "导出",
                    "BUTTON",
                    "report:export",
                ),
                ("complex keep", "/admin/inner", "", "MENU", "inner:view"),
            ];
            for (name, path, title, typ, want) in table {
                assert_eq!(
                    convert_menu_code(path, title, typ).as_deref(),
                    Some(want),
                    "{name}"
                );
            }
        }

        #[test]
        fn convert_menu_code_blank_path_yields_none() {
            assert_eq!(convert_menu_code("", "", "MENU"), None);
            assert_eq!(convert_menu_code("   ", "", "MENU"), None);
            assert_eq!(convert_menu_code("/", "", "MENU"), None);
            assert_eq!(convert_menu_code("//", "", "MENU"), None);
        }

        #[test]
        fn convert_menu_code_drops_first_segment_and_param_segments() {
            // The head module segment never enters the code; `:id`
            // param segments are skipped (empty base + action suffix,
            // same as the reference).
            assert_eq!(
                convert_menu_code("/users/:id", "", "MENU").as_deref(),
                Some(":view")
            );
            assert_eq!(
                convert_menu_code("/opa/users/:id", "", "MENU").as_deref(),
                Some("user:view")
            );
        }

        // ── typeToAction / buttonAction ─────────────────────────────
        // Mirrors TestMenuPermissionConverter_typeToAction.

        #[test]
        fn menu_type_to_action_matches_reference_table() {
            let table = [
                ("catalog -> dir", "", "CATALOG", Some("dir")),
                ("menu -> access", "", "MENU", Some("view")),
                ("embedded -> view", "", "EMBEDDED", Some("view")),
                ("link -> jump", "", "LINK", Some("jump")),
                ("unknown type -> empty", "", "TYPE(999)", None),
            ];
            for (name, title, typ, want) in table {
                assert_eq!(menu_type_to_action(title, typ).as_deref(), want, "{name}");
            }
        }

        #[test]
        fn button_action_matches_reference_table() {
            let table = [
                ("button empty -> act", "", "act"),
                ("button trim space -> act", "   ", "act"),
                ("button add Chinese -> create", "新增", "create"),
                ("button add English -> create", "Add User", "create"),
                ("button edit -> edit", "编辑", "edit"),
                ("button save -> edit", "Save", "edit"),
                ("button delete -> delete", "删除", "delete"),
                ("button import -> import", "导入", "import"),
                ("button export -> export", "导出", "export"),
                (
                    "button mixed -> create (prefix match)",
                    "Add-to-list",
                    "create",
                ),
                (
                    "button substring fallback -> export",
                    "一键导出为Excel",
                    "export",
                ),
            ];
            for (name, title, want) in table {
                assert_eq!(button_action(title), want, "{name}");
            }
        }

        // ── MenuPathToModuleName ────────────────────────────────────

        #[test]
        fn menu_path_to_module_name_takes_the_second_segment() {
            assert_eq!(menu_path_to_module_name("/admin/v1/users"), "admin");
            assert_eq!(menu_path_to_module_name("/users"), "users");
            assert_eq!(menu_path_to_module_name("/admin/"), "admin");
        }

        #[test]
        fn menu_path_to_module_name_falls_back_to_default_module() {
            // No second segment — the DefaultBizPermissionModule.
            assert_eq!(menu_path_to_module_name("users"), DEFAULT_MODULE);
            assert_eq!(menu_path_to_module_name(""), DEFAULT_MODULE);
            assert_eq!(menu_path_to_module_name("/"), DEFAULT_MODULE);
            assert_eq!(menu_path_to_module_name("//x"), DEFAULT_MODULE);
        }

        // ── ConvertCodeByPath ───────────────────────────────────────
        // Mirrors TestApiPermissionConverter_ConvertByPath
        // (go-wind-cms pkg/utils/converter/api_test.go).

        #[test]
        fn convert_api_code_by_path_matches_reference_table() {
            let cases = [
                ("get list users", "GET", "/v1/users", "user:view"),
                ("get single user", "GET", "/v1/users/{id}", "user:view"),
                ("create user", "POST", "/v1/users", "user:create"),
                ("update user", "PUT", "/v1/users/{id}", "user:edit"),
                ("delete user", "DELETE", "/v1/users/{id}", "user:delete"),
                (
                    "nested admin settings",
                    "GET",
                    "/api/v1/admin/settings",
                    "admin:view",
                ),
                ("hyphen group", "GET", "/v1/user-groups", "user-group:view"),
                (
                    "get task by typeNames",
                    "GET",
                    "/admin/v1/tasks:type-names",
                    "task:view",
                ),
                ("walk route", "GET", "/admin/v1/apis/walk-route", "api:view"),
            ];
            for (name, method, path, want) in cases {
                assert_eq!(convert_api_code_by_path(method, path), want, "{name}");
            }
        }

        // ── methodToAction ──────────────────────────────────────────

        #[test]
        fn api_method_to_action_matches_reference_table() {
            let cases = [
                ("GET -> view", "GET", "/v1/users", "view"),
                ("get lowercase -> view", "get", "/v1/users", "view"),
                ("POST -> create", "POST", "/v1/users", "create"),
                ("mixed case POST -> create", "PoSt", "/v1/users", "create"),
                ("PUT -> edit", "PUT", "/v1/users/1", "edit"),
                ("PATCH -> edit", "PATCH", "/v1/users/1", "edit"),
                ("DELETE -> delete", "DELETE", "/v1/users/1", "delete"),
                (
                    "unknown method -> lowercase",
                    "OPTIONS",
                    "/v1/users",
                    "options",
                ),
                (
                    "path ends with /list overrides method",
                    "POST",
                    "/v1/users/list",
                    "view",
                ),
                (
                    "path ends with /list and GET",
                    "GET",
                    "/v1/users/list",
                    "view",
                ),
            ];
            for (name, method, path, want) in cases {
                assert_eq!(api_method_to_action(method, path), want, "{name}");
            }
        }

        // ── stripVersionPrefix ──────────────────────────────────────

        #[test]
        fn strip_version_prefix_matches_reference_table() {
            let cases = [
                ("empty", "", ""),
                ("root slash", "/", ""),
                ("no leading slash v1/users", "v1/users", "users"),
                ("v1 users", "/v1/users", "users"),
                ("api v1 admin", "/api/v1/admin/settings", "admin/settings"),
                ("v2 only", "/v2", ""),
                ("v10 without slash", "v10", ""),
                ("api v10", "/api/v10", ""),
                ("double slash after version", "/api/v1//admin", "admin"),
                ("minor version after version", "/api/v1.9/admin", "admin"),
                ("no version path", "/foo/bar", "foo/bar"),
                (
                    "version mid path stays",
                    "/admin/v1/tasks:type-names",
                    "tasks:type-names",
                ),
            ];
            for (name, input, want) in cases {
                assert_eq!(strip_version_prefix(input), want, "{name}");
            }
        }

        #[test]
        fn is_version_segment_is_version_shaped() {
            assert!(is_version_segment("v1"));
            assert!(is_version_segment("V10"));
            assert!(is_version_segment("v1.9"));
            assert!(!is_version_segment("videos"));
            assert!(!is_version_segment("v"));
            assert!(!is_version_segment("v1.x"));
            assert!(!is_version_segment("1"));
        }

        // ── removePathParams ────────────────────────────────────────

        #[test]
        fn remove_path_params_matches_reference_table() {
            let cases = [
                ("empty", "", ""),
                ("root slash", "/", ""),
                ("only param", "/{id}", ""),
                ("only param no slash", "{id}", ""),
                ("param with spaces", "/{ id }/", ""),
                ("simple resource", "/users", "users"),
                ("resource with param", "/users/{id}", "users"),
                (
                    "nested resources",
                    "/users/{id}/posts/{postId}",
                    "users/posts",
                ),
                ("trailing slash", "/users/{id}/posts/", "users/posts"),
                ("leading param", "/{id}/users", "users"),
                ("all params", "/{id}/{pid}", ""),
                ("double slashes", "/users//{id}//posts", "users/posts"),
                (
                    "colon segment",
                    "/tasks:type-names/{id}",
                    "tasks:type-names",
                ),
            ];
            for (name, input, want) in cases {
                assert_eq!(remove_path_params(input), want, "{name}");
            }
        }

        #[test]
        fn is_param_segment_is_braced_shape() {
            assert!(is_param_segment("{id}"));
            assert!(is_param_segment("{ postId }"));
            assert!(!is_param_segment("{}"));
            assert!(!is_param_segment("{ }"));
            assert!(!is_param_segment("{a/b}"));
            assert!(!is_param_segment("{a}b"));
            assert!(!is_param_segment("id"));
        }

        // ── singularizeSegments ─────────────────────────────────────

        #[test]
        fn singularize_segments_matches_reference_table() {
            let cases = [
                ("empty", "", ""),
                ("already singular", "user:profile", "user:profile"),
                ("simple plural", "tasks", "task"),
                ("colon segments", "tasks:type-names", "task:type-name"),
                ("hyphen segments", "tasks-users", "task-user"),
                ("underline segments", "tasks_users", "task_user"),
                ("slash and underscore", "tasks/type_names", "task/type_name"),
                (
                    "mixed separators",
                    "tasks::names--items",
                    "task::name--item",
                ),
                ("irregular plural", "statuses:passes", "status:pass"),
                (
                    "multiple separators preserved",
                    "tasks--and__more::names",
                    "task--and__more::name",
                ),
            ];
            for (name, input, want) in cases {
                assert_eq!(singularize_segments(input), want, "{name}");
            }
        }

        // ── inflection.Singular ─────────────────────────────────────

        #[test]
        fn singular_word_handles_the_regular_table() {
            let cases = [
                ("users", "user"),
                ("orders", "order"),
                ("settings", "setting"),
                ("tasks", "task"),
                ("groups", "group"),
                ("statuses", "status"),
                ("alias", "alias"),
                ("passes", "pass"),
                ("movies", "movie"),
                ("cookies", "cookie"),
                ("wives", "wife"),
                ("lives", "life"),
                ("analyses", "analysis"),
                ("matrices", "matrix"),
                ("vertices", "vertex"),
                ("indices", "index"),
                ("quizzes", "quiz"),
                ("bacteria", "bacterium"),
                ("data", "datum"),
                ("oxen", "ox"),
                ("axes", "axis"),
                ("databases", "database"),
                // Already singular passes through.
                ("user", "user"),
                ("profile", "profile"),
                ("", ""),
            ];
            for (word, want) in cases {
                assert_eq!(singular_word(word), want, "{word}");
            }
        }

        #[test]
        fn singular_word_irregulars_map_by_suffix() {
            let cases = [
                ("people", "person"),
                ("children", "child"),
                ("men", "man"),
                ("sexes", "sex"),
                ("moves", "move"),
                ("mombies", "mombie"),
                // The title-case variant is tried before the exact one.
                ("Men", "Man"),
                ("People", "Person"),
            ];
            for (word, want) in cases {
                assert_eq!(singular_word(word), want, "{word}");
            }
        }

        #[test]
        fn singular_word_uncountables_pass_through() {
            for word in [
                "equipment",
                "information",
                "rice",
                "money",
                "species",
                "series",
                "fish",
                "sheep",
                "jeans",
                "police",
            ] {
                assert_eq!(singular_word(word), word, "{word}");
            }
        }

        #[test]
        fn singular_word_tries_upper_then_exact_case() {
            assert_eq!(singular_word("USERS"), "USER");
            assert_eq!(singular_word("Users"), "User");
            assert_eq!(singular_word("STATUSES"), "STATUS");
        }

        // ── stringcase.ToPascalCase ─────────────────────────────────

        #[test]
        fn split_words_splits_on_non_alphanumerics() {
            assert_eq!(split_words("hello_world"), vec!["hello", "world"]);
            assert_eq!(split_words("task_view"), vec!["task", "view"]);
            assert_eq!(split_words(""), Vec::<String>::new());
            assert_eq!(split_words("__"), Vec::<String>::new());
        }

        #[test]
        fn split_camel_word_walks_case_and_digit_boundaries() {
            assert_eq!(split_camel_word("adminAPI"), vec!["admin", "API"]);
            assert_eq!(split_camel_word("HTTPServer"), vec!["HTTP", "Server"]);
            assert_eq!(split_camel_word("user"), vec!["user"]);
            assert_eq!(split_camel_word("v2"), vec!["v", "2"]);
        }

        #[test]
        fn pascal_case_titles_each_word() {
            // The sync face derives api-derived permission names this
            // way: code ':' → '_' → PascalCase.
            assert_eq!(pascal_case(&"task:view".replace(':', "_")), "TaskView");
            assert_eq!(pascal_case("hello_world"), "HelloWorld");
            assert_eq!(pascal_case("user"), "User");
            assert_eq!(pascal_case("HTTPServer"), "HttpServer");
            assert_eq!(pascal_case("admin_api"), "AdminApi");
            assert_eq!(pascal_case(""), "");
        }
    }
}
