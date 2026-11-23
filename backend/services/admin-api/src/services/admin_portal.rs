//! AdminPortalService — the admin BFF's aggregation face: the
//! role-pruned route tree, the operator's permission codes, and the
//! boot context (menus + codes in one call). The chain mirrors the
//! reference: the operator's roles → permission ids (core RoleService,
//! the UserId branch off the verified claims) → the MENU resource id
//! set / the permission codes (core PermissionService); the menu list
//! then prunes onto that id set. No platform-admin shortcut — the
//! reference grants whatever the role bindings carry.

use std::collections::HashSet;
use std::sync::Arc;

use pbjson_types::Empty;

use crate::services::{map_status, with_operator};
use crate::state::{operator_of, AppState, StatusError};

use proto::proto::admin::service::v1::{
    InitialContextResponse, ListPermissionCodeResponse, ListRouteResponse,
};
use proto::proto::permission::service::v1::{
    list_permission_ids_request, list_permission_resources_request, menu, menu_service_client,
    permission_service_client, role_service_client, ListPermissionCodesByIdsRequest,
    ListPermissionIdsRequest, ListPermissionResourcesRequest, Menu, MenuMeta, MenuRouteItem,
};

type Ctx = rushwind_http_binding::ctx::RequestContext;

/// The unfiltered paging request (the aggregation wants everything).
fn paging_all() -> proto::proto::pagination::PagingRequest {
    proto::proto::pagination::PagingRequest {
        no_paging: Some(true),
        ..Default::default()
    }
}

pub struct AdminPortalService {
    pub state: Arc<AppState>,
}

impl AdminPortalService {
    /// The operator's permission ids off the role bindings — core
    /// RoleService.list_permission_ids, the UserId branch (the
    /// reference's GetMyPermissionCode path; GetNavigation rides the
    /// user → roleIds detour instead).
    async fn permission_ids(&self, ctx: &Ctx, uid: i64) -> Result<Vec<i64>, StatusError> {
        let mut roles =
            role_service_client::RoleServiceClient::new(self.state.core_channel.clone());
        let ids = roles
            .list_permission_ids(with_operator(
                ctx,
                ListPermissionIdsRequest {
                    role_ids: Vec::new(),
                    role_codes: Vec::new(),
                    query_by: Some(list_permission_ids_request::QueryBy::UserId(uid)),
                },
            ))
            .await
            .map_err(map_status)?
            .into_inner()
            .permission_ids;
        Ok(ids)
    }

    /// The operator's menu-resource id set — queryMultipleRolesMenusByRoleIds
    /// with the UserId branch: permission ids → ListPermissionResources(MENU).
    /// An empty permission set stays empty (the reference's "user has no
    /// roles assigned" warn path, before the resources call).
    async fn my_menu_ids(&self, ctx: &Ctx, uid: i64) -> Result<Vec<i64>, StatusError> {
        let permission_ids = self.permission_ids(ctx, uid).await?;
        if permission_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut perms = permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        let mut resp = perms
            .list_permission_resources(with_operator(
                ctx,
                ListPermissionResourcesRequest {
                    role_ids: Vec::new(),
                    permission_ids,
                    resource_types: vec![
                        list_permission_resources_request::ResourceType::Menu as i32,
                    ],
                },
            ))
            .await
            .map_err(map_status)?
            .into_inner();
        let mut ids = resp
            .resources
            .remove(&(list_permission_resources_request::ResourceType::Menu as i32))
            .map(|set| set.ids)
            .unwrap_or_default();
        // sliceutil.Unique（保持首现序去重）
        let mut seen = HashSet::new();
        ids.retain(|v| seen.insert(*v));
        Ok(ids)
    }

    /// The role-pruned route tree: the ON menu list keeps only the
    /// granted menu resources, then fillRouteItem's mapping (path /
    /// component / name / redirect / alias / meta with the title+icon
    /// pair; non-ON rows and buttons skipped). An empty grant set is
    /// the empty tree.
    async fn route_tree(&self, ctx: &Ctx, uid: i64) -> Result<Vec<MenuRouteItem>, StatusError> {
        let menu_ids: HashSet<i64> = self.my_menu_ids(ctx, uid).await?.into_iter().collect();
        if menu_ids.is_empty() {
            return Ok(Vec::new());
        }

        let menus = menu_service_client::MenuServiceClient::new(self.state.core_channel.clone())
            .list(tonic::Request::new(paging_all()))
            .await
            .map_err(map_status)?
            .into_inner()
            .items;

        // Roots first, children ride flat after them — the reference's
        // core List stays flat (treeTravel off), so fillRouteItem never
        // nests either; the golden tree is two-level.
        let mut roots: Vec<MenuRouteItem> = Vec::new();
        let mut children: Vec<MenuRouteItem> = Vec::new();
        for m in menus {
            // fillRouteItem's skips: non-ON rows and buttons.
            if m.status != Some(menu::Status::On as i32)
                || m.r#type == Some(menu::Type::Button as i32)
            {
                continue;
            }
            // the role pruning: only the granted menu ids survive
            if !m.id.map(|id| menu_ids.contains(&id)).unwrap_or(false) {
                continue;
            }
            let item = fill_route_item(&m);
            if m.parent_id.unwrap_or(0) == 0 {
                roots.push(item);
            } else {
                children.push(item);
            }
        }
        roots.extend(children);
        Ok(roots)
    }

    /// The operator's permission codes — GetMyPermissionCode's chain:
    /// role → permission ids → ListPermissionCodesByIds.
    async fn permission_codes(&self, ctx: &Ctx, uid: i64) -> Result<Vec<String>, StatusError> {
        let permission_ids = self.permission_ids(ctx, uid).await?;
        let mut perms = permission_service_client::PermissionServiceClient::new(
            self.state.core_channel.clone(),
        );
        let resp = perms
            .list_permission_codes_by_ids(with_operator(
                ctx,
                ListPermissionCodesByIdsRequest { permission_ids },
            ))
            .await
            .map_err(map_status)?
            .into_inner();
        Ok(resp.permission_codes)
    }
}

/// fillRouteItem's field mapping for one menu row (the meta carries the
/// title/icon pair the frontend renders).
fn fill_route_item(m: &Menu) -> MenuRouteItem {
    MenuRouteItem {
        children: Vec::new(),
        path: m.path.clone(),
        redirect: m.redirect.clone(),
        alias: m.r#alias.clone(),
        name: m.name.clone(),
        component: m.component.clone(),
        meta: m.meta.clone().map(|meta| MenuMeta {
            title: meta.title,
            icon: meta.icon,
            ..Default::default()
        }),
    }
}

#[async_trait::async_trait]
impl proto::gen_admin::services::AdminPortalServiceHandlers for AdminPortalService {
    async fn get_navigation(
        &self,
        ctx: Ctx,
        _req: Empty,
    ) -> Result<ListRouteResponse, StatusError> {
        let operator = operator_of(&ctx)?;
        let items = self.route_tree(&ctx, operator.user_id).await?;
        Ok(ListRouteResponse { items })
    }

    async fn get_my_permission_code(
        &self,
        ctx: Ctx,
        _req: Empty,
    ) -> Result<ListPermissionCodeResponse, StatusError> {
        let operator = operator_of(&ctx)?;
        let codes = self.permission_codes(&ctx, operator.user_id).await?;
        Ok(ListPermissionCodeResponse {
            codes,
            // 字段权限的隐藏字段集（"资源.字段" 串数组）随字段权限特性批次
            // 落地；聚合面暂回空集（不做额外隐藏）。
            hidden_fields: Vec::new(),
        })
    }

    async fn get_initial_context(
        &self,
        ctx: Ctx,
        _req: Empty,
    ) -> Result<InitialContextResponse, StatusError> {
        let operator = operator_of(&ctx)?;
        let menus = self.route_tree(&ctx, operator.user_id).await?;
        let permissions = self.permission_codes(&ctx, operator.user_id).await?;
        Ok(InitialContextResponse { menus, permissions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_item_keeps_the_route_fields_and_narrows_the_meta() {
        let menu = Menu {
            id: Some(3),
            status: Some(1),
            path: Some("/system".into()),
            redirect: Some("/system/user".into()),
            alias: Some("sys".into()),
            name: Some("System".into()),
            component: Some("BasicLayout".into()),
            parent_id: Some(0),
            meta: Some(MenuMeta {
                title: Some("系统管理".into()),
                icon: Some("settings".into()),
                link: Some("https://example.com".into()),
                order: Some(2),
                ..Default::default()
            }),
            ..Default::default()
        };
        let item = fill_route_item(&menu);
        assert!(item.children.is_empty());
        assert_eq!(item.path.as_deref(), Some("/system"));
        assert_eq!(item.redirect.as_deref(), Some("/system/user"));
        assert_eq!(item.alias.as_deref(), Some("sys"));
        assert_eq!(item.name.as_deref(), Some("System"));
        assert_eq!(item.component.as_deref(), Some("BasicLayout"));
        // Only the title + icon pair survives the narrowing.
        let meta = item.meta.unwrap();
        assert_eq!(meta.title.as_deref(), Some("系统管理"));
        assert_eq!(meta.icon.as_deref(), Some("settings"));
        assert_eq!(meta.link, None);
        assert_eq!(meta.order, None);

        // The route tree stays flat: children never nest here, and an
        // absent meta column carries no meta.
        let nested = Menu {
            children: vec![Menu {
                id: Some(4),
                ..Default::default()
            }],
            ..menu
        };
        assert!(fill_route_item(&nested).children.is_empty());
        assert!(fill_route_item(&Menu::default()).meta.is_none());
    }

    #[test]
    fn the_aggregation_pages_without_limits() {
        assert_eq!(paging_all().no_paging, Some(true));
    }
}
