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
        id: Some(r.id),
        status: Some(1),
        path: r.path,
        redirect: r.redirect,
        alias: r.r#alias,
        name: r.name,
        component: r.component,
        meta: menu_meta(r.meta),
        parent_id: r.parent_id,
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
            total: total as i64,
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
            count: total as i64,
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
        let row = menu_repo::menus_by_id(&self.state.db, id).await?;
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
            parent_id: Set(data.parent_id),
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
        let row = sys_menus::Entity::find_by_id(req.id)
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
        menu_repo::delete_menus(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::menu_repo;

#[cfg(test)]
mod tests {
    use super::*;

    /// A menu row: an ON directory entry with the given meta column.
    fn row(id: i64, parent_id: Option<i64>, meta: Option<sea_orm::JsonValue>) -> sys_menus::Model {
        sys_menus::Model {
            id,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
            remark: None,
            status: "ON".into(),
            tenant_id: None,
            r#type: Some("MENU".into()),
            path: Some(format!("/p{id}")),
            redirect: None,
            alias: None,
            name: Some(format!("n{id}")),
            component: Some("BasicLayout".into()),
            meta,
            parent_id,
        }
    }

    #[test]
    fn menu_meta_round_trips_the_golden_json() {
        let stored = serde_json::json!({
            "title": "系统管理",
            "icon": "settings",
            "activeIcon": "settings-fill",
            "activePath": "/system",
            "hideInMenu": true,
            "hideInTab": false,
            "hideInBreadcrumb": true,
            "hideChildrenInMenu": false,
            "keepAlive": true,
            "iframeSrc": "https://example.com",
            "link": "https://example.org",
            "order": 3
        });
        let meta = menu_meta(Some(stored.clone())).unwrap();
        assert_eq!(meta.title.as_deref(), Some("系统管理"));
        assert_eq!(meta.icon.as_deref(), Some("settings"));
        assert_eq!(meta.active_icon.as_deref(), Some("settings-fill"));
        assert_eq!(meta.active_path.as_deref(), Some("/system"));
        assert_eq!(meta.hide_in_menu, Some(true));
        assert_eq!(meta.hide_in_tab, Some(false));
        assert_eq!(meta.hide_in_breadcrumb, Some(true));
        assert_eq!(meta.hide_children_in_menu, Some(false));
        assert_eq!(meta.keep_alive, Some(true));
        assert_eq!(meta.iframe_src.as_deref(), Some("https://example.com"));
        assert_eq!(meta.link.as_deref(), Some("https://example.org"));
        assert_eq!(meta.order, Some(3));
        assert_eq!(menu_meta_json(Some(meta)).unwrap(), stored);
        // Absent in, absent out.
        assert!(menu_meta(None).is_none());
        assert!(menu_meta_json(None).is_none());
    }

    #[test]
    fn menu_meta_survives_a_partial_round_trip() {
        // A sparse meta writes its absent keys as null — and reads back
        // equal to what went in.
        let sparse = permissionv1::MenuMeta {
            title: Some("根".into()),
            order: Some(0),
            ..Default::default()
        };
        let json = menu_meta_json(Some(sparse.clone())).unwrap();
        assert_eq!(json["title"], "根");
        assert_eq!(json["order"], 0);
        assert!(json["icon"].is_null());
        assert!(json["link"].is_null());
        assert_eq!(menu_meta(Some(json)), Some(sparse));
        // A fully empty meta keeps every key, all null.
        assert!(menu_meta_json(Some(permissionv1::MenuMeta::default()))
            .unwrap()
            .as_object()
            .is_some_and(|m| m.len() == 12 && m.values().all(serde_json::Value::is_null)));
    }

    #[test]
    fn menu_meta_reads_missing_and_mistyped_keys_as_absent() {
        let meta = menu_meta(Some(serde_json::json!({
            "title": "t",
            "order": "3",
            "hideInMenu": "yes",
            "icon": null
        })))
        .unwrap();
        assert_eq!(meta.title.as_deref(), Some("t"));
        // A string where a number or a bool is expected reads as absent.
        assert_eq!(meta.order, None);
        assert_eq!(meta.hide_in_menu, None);
        assert_eq!(meta.icon, None);
        // A non-object column yields an all-absent meta — no key matches.
        let meta = menu_meta(Some(serde_json::json!("x"))).unwrap();
        assert_eq!(meta, permissionv1::MenuMeta::default());
    }

    #[test]
    fn menu_proto_nests_its_children_and_reads_the_meta() {
        let root = row(1, None, Some(serde_json::json!({"title": "根"})));
        let child = row(2, Some(1), Some(serde_json::json!({"title": "子"})));
        let p = menu_proto(root, vec![child]);
        assert_eq!(p.id, Some(1));
        assert_eq!(p.parent_id, None);
        assert_eq!(p.status, Some(1));
        assert_eq!(p.path.as_deref(), Some("/p1"));
        assert_eq!(p.name.as_deref(), Some("n1"));
        assert_eq!(p.component.as_deref(), Some("BasicLayout"));
        assert_eq!(p.meta.unwrap().title.as_deref(), Some("根"));
        assert_eq!(p.children.len(), 1);
        let child = &p.children[0];
        assert_eq!(child.id, Some(2));
        assert_eq!(child.parent_id, Some(1));
        assert_eq!(child.meta.as_ref().unwrap().title.as_deref(), Some("子"));
        // The children ride flat: a child never nests further.
        assert!(child.children.is_empty());

        // Without a meta column the entry carries no meta.
        let p = menu_proto(row(3, Some(0), None), Vec::new());
        assert!(p.meta.is_none());
        assert!(p.children.is_empty());
        // A root's parent_id passes through verbatim.
        assert_eq!(p.parent_id, Some(0));
    }
}
