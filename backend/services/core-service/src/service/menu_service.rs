//! The menu service — the menu-tree CRUD with the meta JSON
//! round-trip (menu_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use proto::proto::permission::service::v1 as permissionv1;
use store::entities::sys_menus;
use store::paging::fetch_paged;

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

pub struct MenuService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::menu_service_server::MenuService for MenuService {
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
        let row = menu_repo::menus_by_id(&self.state.db, id as i64).await?;
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
        menu_repo::delete_menus(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::menu_repo;
