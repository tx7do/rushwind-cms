//! The admin REST route pack — mounts the full admin BFF route surface
//! behind the service implementations (thin gRPC proxies to the core
//! domain service) (Phase 0: the generator's null
//! placeholders; each module lands as its real implementation replaces
//! its null), splits the auth-free public subtree from the gated one
//! (the auth-free set), composes the per-route layer stack, and hands
//! the router to the lifecycle assembler — whose edge block (the CORS
//! policy and the request budget from the server document) wraps the
//! whole thing.
//!
//! Per-route layer composition (the `wrap` closure below): the framework
//! bind layer — body and query binding — outermost on EVERY route, and
//! the auth gate (the auth crate) composed INSIDE it on gated routes
//! only. That order is the wire contract: codec/binding failures answer
//! 400 ahead of any 401.
//!
//! The audit-write layer and the authorization engine wire here when
//! the storage phase lands; until then the gate is the protected
//! subtree's only defense.

use std::sync::Arc;

use axum::routing::MethodRouter;

use crate::state::AppState;
use auth::auth_gate;
use proto::pool;
use rushwind_http_binding::bindgate::bind_run;
use rushwind_http_binding::wire::RouteWire;

/// The deployment's registered codec subtypes — the packages
/// its binary imports (the compatibility spec §2.1 register).
const REGISTERED_SUBTYPES: &[&str] = &["json", "proto", "x-www-form-urlencoded"];

/// The owning BFF package — the error tables' anchor for the gate's
/// envelope.
const PACKAGE: &str = "admin.service.v1";

/// The adapter wiring the core service's ValidateToken RPC into the
/// auth gate's server-side check stage (the reference's TokenChecker
/// dials core the same way).
struct CoreTokenChecker(std::sync::Arc<crate::state::AppState>);

#[async_trait::async_trait]
impl auth::AccessTokenChecker for CoreTokenChecker {
    async fn is_valid_access_token(&self, _uid: u32, _jti: &str, token: &str) -> bool {
        let mut core = self.0.core.clone();
        let Ok(resp) = core
            .validate_token(tonic::Request::new(
                proto::proto::authentication::service::v1::ValidateTokenRequest {
                    token: token.to_string(),
                    client_type: 0,
                    token_category: proto::proto::authentication::service::v1::TokenCategory::Access
                        as i32,
                    ..Default::default()
                },
            ))
            .await
        else {
            return false;
        };
        resp.into_inner().is_valid
    }
    async fn is_blocked_access_token(&self, _jti: &str) -> bool {
        // The whitelist check subsumes the blacklist (core returns a
        // single validity verdict over both families).
        false
    }
}

/// The admin route pack: the mounted router (see the module docs).
pub fn pack(
    state: Arc<AppState>,
) -> impl Fn(
    serde_json::Value,
    rushwind_bootstrap::RouteInput,
) -> Result<rushwind_bootstrap::RouteSurface, rushwind_bootstrap::BootstrapError>
       + Send
       + Sync
       + 'static {
    move |_settings, _input| {
        Ok(rushwind_bootstrap::RouteSurface::new(build_router(
            Arc::clone(&state),
        )))
    }
}

/// Builds the mounted router. `state` carries the verification engine
/// and the server-side session store.
pub fn build_router(state: Arc<AppState>) -> axum::Router {
    let descriptor_pool = pool();
    let authenticator = Arc::clone(&state.authenticator);
    let checker = Arc::new(CoreTokenChecker(Arc::clone(&state)))
        as Arc<dyn auth::AccessTokenChecker + 'static>;

    // The per-route layer composition (see the module docs): bind layer
    // outermost always, auth gate inside it on gated routes.
    let wrap = |mr: MethodRouter, wire: &RouteWire, gated: bool| -> MethodRouter {
        let mut out = mr;
        if gated {
            let auth = Arc::clone(&authenticator);
            let checker = Arc::clone(&checker);
            let gate = axum::middleware::from_fn(move |req, next| {
                let auth = Arc::clone(&auth);
                let checker = Arc::clone(&checker);
                async move { auth_gate(auth, checker, PACKAGE, req, next).await }
            });
            out = out.layer(gate);
        }
        let input_fq = wire.input_fq;
        let body_star = wire.body_star;
        let bind = axum::middleware::from_fn(move |req, next| {
            let (fq, body) = (input_fq, body_star);
            async move { bind_run(descriptor_pool, fq, body, REGISTERED_SUBTYPES, req, next).await }
        });
        out = out.layer(bind);
        out
    };

    // The full mounted surface — the real service implementations where
    // landed, the generator's null placeholders behind the rest. Every
    // mount call threads both routers; the generator's auth-free table
    // classifies each of its route bindings.
    let mut router_pub = axum::Router::new();
    let mut router_gate = axum::Router::new();
    /// The uniform service mount: every service follows the same
    /// router-threading, service-construction, and wrap handshake.
    macro_rules! mount_services {
    ($($mount:ident => $ctor:expr),* $(,)?) => {
        $(
            (router_pub, router_gate) = proto::gen_admin::mounts::$mount(
                router_pub,
                router_gate,
                $ctor,
                &wrap,
            );
        )*
    };
}

    macro_rules! null {
        ($svc:ident) => {
            proto::gen_admin::nulls::$svc()
        };
    }

    mount_services!(
        mount_admin_portal_service => std::sync::Arc::new(
            crate::services::admin_portal::AdminPortalService {
                state: std::sync::Arc::clone(&state),
            },
        ),
        mount_api_audit_log_service => std::sync::Arc::new(crate::services::proxies::ApiAuditLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_api_service => std::sync::Arc::new(crate::services::proxies::ApiProxy { state: std::sync::Arc::clone(&state) }),
        mount_authentication_service => Arc::new(
            crate::services::authentication::AuthenticationService {
                state: Arc::clone(&state),
            },
        ),
        mount_category_service => std::sync::Arc::new(crate::services::proxies::CategoryProxy { state: std::sync::Arc::clone(&state) }),
        mount_comment_service => std::sync::Arc::new(crate::services::proxies::CommentProxy { state: std::sync::Arc::clone(&state) }),
        mount_content_model_service => std::sync::Arc::new(crate::services::proxies::ContentModelProxy { state: std::sync::Arc::clone(&state) }),
        mount_data_access_audit_log_service => std::sync::Arc::new(crate::services::proxies::DataAccessAuditLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_dict_entry_service => std::sync::Arc::new(crate::services::proxies::DictEntryProxy { state: std::sync::Arc::clone(&state) }),
        mount_dict_type_service => std::sync::Arc::new(crate::services::proxies::DictTypeProxy { state: std::sync::Arc::clone(&state) }),
        mount_file_service => std::sync::Arc::new(crate::services::proxies::FileProxy { state: std::sync::Arc::clone(&state) }),
        mount_file_transfer_service => proto::gen_admin::nulls::null_file_transfer_service(),
        mount_interaction_admin_service => std::sync::Arc::new(crate::services::proxies::InteractionAdminProxy { state: std::sync::Arc::clone(&state) }),
        mount_internal_message_category_service => null!(
            null_internal_message_category_service
        ),
        mount_internal_message_recipient_service => null!(
            null_internal_message_recipient_service
        ),
        mount_internal_message_service => std::sync::Arc::new(crate::services::proxies::InternalMessageProxy { state: std::sync::Arc::clone(&state) }),
        mount_language_service => std::sync::Arc::new(crate::services::proxies::LanguageProxy { state: std::sync::Arc::clone(&state) }),
        mount_login_audit_log_service => std::sync::Arc::new(crate::services::proxies::LoginAuditLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_login_policy_service => std::sync::Arc::new(crate::services::proxies::LoginPolicyProxy { state: std::sync::Arc::clone(&state) }),
        mount_media_asset_service => std::sync::Arc::new(crate::services::proxies::MediaAssetProxy { state: std::sync::Arc::clone(&state) }),
        mount_menu_service => std::sync::Arc::new(crate::services::proxies::MenuProxy { state: std::sync::Arc::clone(&state) }),
        mount_navigation_item_service => std::sync::Arc::new(crate::services::proxies::NavigationItemProxy { state: std::sync::Arc::clone(&state) }),
        mount_navigation_service => std::sync::Arc::new(crate::services::proxies::NavigationProxy { state: std::sync::Arc::clone(&state) }),
        mount_operation_audit_log_service => std::sync::Arc::new(crate::services::proxies::OperationAuditLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_org_unit_service => std::sync::Arc::new(crate::services::proxies::OrgUnitProxy { state: std::sync::Arc::clone(&state) }),
        mount_page_service => std::sync::Arc::new(crate::services::proxies::PageProxy { state: std::sync::Arc::clone(&state) }),
        mount_permission_audit_log_service => std::sync::Arc::new(crate::services::proxies::PermissionAuditLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_permission_group_service => std::sync::Arc::new(crate::services::proxies::PermissionGroupProxy { state: std::sync::Arc::clone(&state) }),
        mount_permission_service => std::sync::Arc::new(crate::services::proxies::PermissionProxy { state: std::sync::Arc::clone(&state) }),
        mount_policy_evaluation_log_service => std::sync::Arc::new(crate::services::proxies::PolicyEvaluationLogProxy { state: std::sync::Arc::clone(&state) }),
        mount_position_service => std::sync::Arc::new(crate::services::proxies::PositionProxy { state: std::sync::Arc::clone(&state) }),
        mount_post_service => std::sync::Arc::new(crate::services::proxies::PostProxy { state: std::sync::Arc::clone(&state) }),
        mount_role_service => std::sync::Arc::new(crate::services::proxies::RoleProxy { state: std::sync::Arc::clone(&state) }),
        mount_site_service => std::sync::Arc::new(crate::services::proxies::SiteProxy { state: std::sync::Arc::clone(&state) }),
        mount_site_setting_service => std::sync::Arc::new(crate::services::proxies::SiteSettingProxy { state: std::sync::Arc::clone(&state) }),
        mount_stats_service => std::sync::Arc::new(crate::services::proxies::StatsProxy { state: std::sync::Arc::clone(&state) }),
        mount_tag_service => std::sync::Arc::new(crate::services::proxies::TagProxy { state: std::sync::Arc::clone(&state) }),
        mount_task_service => std::sync::Arc::new(crate::services::proxies::TaskProxy { state: std::sync::Arc::clone(&state) }),
        mount_tenant_service => std::sync::Arc::new(crate::services::proxies::TenantProxy { state: std::sync::Arc::clone(&state) }),
        mount_translator_service => std::sync::Arc::new(crate::services::proxies::TranslatorProxy { state: std::sync::Arc::clone(&state) }),
        mount_user_profile_service => std::sync::Arc::new(crate::services::proxies::UserProfileProxy { state: std::sync::Arc::clone(&state) }),
        mount_user_service => std::sync::Arc::new(crate::services::proxies::UserProxy { state: std::sync::Arc::clone(&state) }),
    );

    router_pub.merge(router_gate)
}
