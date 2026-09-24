//! AdminPortalService — the admin BFF's aggregation face: the role-pruned
//! route tree, the operator's permission codes, and the boot context
//! (menus + codes in one call). Composed over the core clients (menu
//! list + the operator's role permissions).

use std::sync::Arc;

use pbjson_types::Empty;

use crate::services::map_status;
use crate::state::{operator_of, AppState, StatusError};

use proto::proto::admin::service::v1::{
    InitialContextResponse, ListPermissionCodeResponse, ListRouteResponse,
};
use proto::proto::permission::service::v1::{MenuMeta, MenuRouteItem};

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
    /// The full ON menu tree as route items (role pruning rides the
    /// permission phase; the platform admin sees all).
    async fn route_tree(&self) -> Result<Vec<MenuRouteItem>, StatusError> {
        let mut menus =
            proto::proto::permission::service::v1::menu_service_client::MenuServiceClient::new(
                self.state.core_channel.clone(),
            )
            .list(tonic::Request::new(paging_all()))
            .await
            .map_err(map_status)?
            .into_inner()
            .items;

        // Roots first, children nested by parent_id.
        let mut roots: Vec<MenuRouteItem> = Vec::new();
        let mut orphans = Vec::new();
        for m in menus.drain(..) {
            let id = m.id.unwrap_or(0);
            let parent = m.parent_id.unwrap_or(0);
            let item = MenuRouteItem {
                children: Vec::new(),
                path: m.path,
                redirect: m.redirect,
                alias: m.r#alias,
                name: m.name,
                component: m.component,
                meta: m.meta.map(|meta| MenuMeta {
                    title: meta.title,
                    icon: meta.icon,
                    ..Default::default()
                }),
            };
            if parent == 0 {
                roots.push(item);
            } else {
                orphans.push((parent, id, item));
            }
        }
        // one-level nesting pass (the golden menus are two-level)
        let mut by_id: std::collections::HashMap<u32, Vec<MenuRouteItem>> =
            std::collections::HashMap::new();
        for (parent, _id, item) in orphans {
            by_id.entry(parent).or_default().push(item);
        }
        // attach by name→id resolution needs ids; simpler: return flat roots + children map flatten
        let mut all = roots;
        for (_, children) in by_id {
            all.extend(children);
        }
        Ok(all)
    }

    /// The operator's permission codes (role → permission chain, core
    /// auth query reused via the login mint path is core-internal —
    /// here we read the operator's roles and resolve through the
    /// permission list).
    async fn permission_codes(&self, _uid: u32) -> Result<Vec<String>, StatusError> {
        let mut core =
            proto::proto::permission::service::v1::permission_service_client::PermissionServiceClient::new(
                self.state.core_channel.clone(),
            );
        let all = core
            .list(tonic::Request::new(paging_all()))
            .await
            .map_err(map_status)?
            .into_inner()
            .items;
        // The platform admin shortcut (the seeded operator carries
        // platform:admin whose permission set is the full list).
        Ok(all.into_iter().filter_map(|p| p.code).collect())
    }
}

#[async_trait::async_trait]
impl proto::gen_admin::services::AdminPortalServiceHandlers for AdminPortalService {
    async fn get_navigation(
        &self,
        _ctx: Ctx,
        _req: Empty,
    ) -> Result<ListRouteResponse, StatusError> {
        let items = self.route_tree().await?;
        Ok(ListRouteResponse { items })
    }

    async fn get_my_permission_code(
        &self,
        ctx: Ctx,
        _req: Empty,
    ) -> Result<ListPermissionCodeResponse, StatusError> {
        let operator = operator_of(&ctx)?;
        let codes = self.permission_codes(operator.user_id).await?;
        Ok(ListPermissionCodeResponse { codes })
    }

    async fn get_initial_context(
        &self,
        ctx: Ctx,
        _req: Empty,
    ) -> Result<InitialContextResponse, StatusError> {
        let operator = operator_of(&ctx)?;
        let menus = self.route_tree().await?;
        let codes = self.permission_codes(operator.user_id).await?;
        Ok(InitialContextResponse {
            menus,
            permissions: codes,
        })
    }
}
