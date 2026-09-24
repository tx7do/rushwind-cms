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
    async fn permission_ids(&self, ctx: &Ctx, uid: u32) -> Result<Vec<u32>, StatusError> {
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
    async fn my_menu_ids(&self, ctx: &Ctx, uid: u32) -> Result<Vec<u32>, StatusError> {
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
    async fn route_tree(&self, ctx: &Ctx, uid: u32) -> Result<Vec<MenuRouteItem>, StatusError> {
        let menu_ids: HashSet<u32> = self.my_menu_ids(ctx, uid).await?.into_iter().collect();
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
    async fn permission_codes(&self, ctx: &Ctx, uid: u32) -> Result<Vec<String>, StatusError> {
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
        Ok(ListPermissionCodeResponse { codes })
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
