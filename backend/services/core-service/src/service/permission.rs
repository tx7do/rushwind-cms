//! The permission cluster: Menu (tree), Api, PermissionGroup,
//! Permission — CRUD over the golden schema. The walk-route and sync
//! faces stay on their generated Unimplemented defaults (they rebuild
//! derived tables; the dedicated pipeline owns that).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::permission_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{sys_apis, sys_menus, sys_permission_groups, sys_permissions};
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
}
