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

use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::services::proxies;
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

    mount_services!(
        mount_admin_portal_service => Arc::new(
            crate::services::admin_portal::AdminPortalService {
                state: Arc::clone(&state),
            },
        ),
        mount_api_audit_log_service => Arc::new(proxies::ApiAuditLogProxy {
            state: Arc::clone(&state),
        }),
        mount_api_service => Arc::new(proxies::ApiProxy { state: Arc::clone(&state) }),
        mount_authentication_service => Arc::new(
            crate::services::authentication::AuthenticationService {
                state: Arc::clone(&state),
            },
        ),
        mount_category_service => Arc::new(proxies::CategoryProxy { state: Arc::clone(&state) }),
        mount_comment_service => Arc::new(proxies::CommentProxy { state: Arc::clone(&state) }),
        mount_content_model_service => Arc::new(proxies::ContentModelProxy {
            state: Arc::clone(&state),
        }),
        mount_data_access_audit_log_service => Arc::new(proxies::DataAccessAuditLogProxy {
            state: Arc::clone(&state),
        }),
        mount_dict_entry_service => Arc::new(proxies::DictEntryProxy { state: Arc::clone(&state) }),
        mount_dict_type_service => Arc::new(proxies::DictTypeProxy { state: Arc::clone(&state) }),
        mount_file_service => Arc::new(proxies::FileProxy { state: Arc::clone(&state) }),
        mount_interaction_admin_service => Arc::new(proxies::InteractionAdminProxy {
            state: Arc::clone(&state),
        }),
        mount_internal_message_category_service => Arc::new(proxies::InternalMessageCategoryProxy {
            state: Arc::clone(&state),
        }),
        mount_internal_message_recipient_service => Arc::new(
            proxies::InternalMessageRecipientProxy { state: Arc::clone(&state) },
        ),
        // The hand impl (not the generated proxy): send_message fans the
        // notification payload out over the SSE hub after the core RPC.
        mount_internal_message_service => Arc::new(
            crate::services::internal_message::InternalMessageService { state: Arc::clone(&state) },
        ),
        mount_language_service => Arc::new(proxies::LanguageProxy { state: Arc::clone(&state) }),
        mount_login_audit_log_service => Arc::new(proxies::LoginAuditLogProxy {
            state: Arc::clone(&state),
        }),
        mount_login_policy_service => Arc::new(proxies::LoginPolicyProxy {
            state: Arc::clone(&state),
        }),
        mount_media_asset_service => Arc::new(proxies::MediaAssetProxy {
            state: Arc::clone(&state),
        }),
        mount_menu_service => Arc::new(proxies::MenuProxy { state: Arc::clone(&state) }),
        mount_navigation_item_service => Arc::new(proxies::NavigationItemProxy {
            state: Arc::clone(&state),
        }),
        mount_navigation_service => Arc::new(proxies::NavigationProxy {
            state: Arc::clone(&state),
        }),
        mount_operation_audit_log_service => Arc::new(proxies::OperationAuditLogProxy {
            state: Arc::clone(&state),
        }),
        mount_org_unit_service => Arc::new(proxies::OrgUnitProxy { state: Arc::clone(&state) }),
        mount_page_service => Arc::new(proxies::PageProxy { state: Arc::clone(&state) }),
        mount_permission_audit_log_service => Arc::new(proxies::PermissionAuditLogProxy {
            state: Arc::clone(&state),
        }),
        mount_permission_group_service => Arc::new(proxies::PermissionGroupProxy {
            state: Arc::clone(&state),
        }),
        mount_permission_service => Arc::new(proxies::PermissionProxy {
            state: Arc::clone(&state),
        }),
        mount_policy_evaluation_log_service => Arc::new(proxies::PolicyEvaluationLogProxy {
            state: Arc::clone(&state),
        }),
        mount_position_service => Arc::new(proxies::PositionProxy { state: Arc::clone(&state) }),
        mount_post_service => Arc::new(proxies::PostProxy { state: Arc::clone(&state) }),
        mount_role_service => Arc::new(proxies::RoleProxy { state: Arc::clone(&state) }),
        mount_site_service => Arc::new(proxies::SiteProxy { state: Arc::clone(&state) }),
        mount_site_setting_service => Arc::new(proxies::SiteSettingProxy {
            state: Arc::clone(&state),
        }),
        mount_stats_service => Arc::new(proxies::StatsProxy { state: Arc::clone(&state) }),
        mount_tag_service => Arc::new(proxies::TagProxy { state: Arc::clone(&state) }),
        mount_task_service => Arc::new(proxies::TaskProxy { state: Arc::clone(&state) }),
        mount_tenant_service => Arc::new(proxies::TenantProxy { state: Arc::clone(&state) }),
        mount_translator_service => Arc::new(proxies::TranslatorProxy {
            state: Arc::clone(&state),
        }),
        mount_user_profile_service => Arc::new(proxies::UserProfileProxy {
            state: Arc::clone(&state),
        }),
        mount_user_service => Arc::new(proxies::UserProxy { state: Arc::clone(&state) }),
    );

    let mut app = router_pub.merge(router_gate);

    // The file-transfer face: multipart upload + streaming download,
    // hand-mounted (the bind layer parses protojson, not multipart —
    // the same reason the reference registers these by hand, and its
    // generated registration for the face is skipped above). Every
    // route of the face is gated: the same auth gate the generated
    // protected subtree rides — the reference's hand registration sits
    // inside the same server middleware chain.
    // The hand faces' gate — a factory, since each layered router
    // consumes its own middleware instance.
    let mk_hand_gate = || {
        let auth = Arc::clone(&authenticator);
        let checker = Arc::clone(&checker);
        axum::middleware::from_fn(move |req, next| {
            let auth = Arc::clone(&auth);
            let checker = Arc::clone(&checker);
            async move { auth::auth_gate(auth, checker, PACKAGE, req, next).await }
        })
    };
    let hand = crate::services::file_transfer::router(Arc::clone(&state)).layer(mk_hand_gate());
    app = app.merge(hand);

    // The walk-route debug dump: the generated registration of this
    // route is skipped (its static segment loses the generated mux's
    // first-match analysis to `/apis/{id}`), while the reference's radix
    // router serves it — hand-mounted here; matchit resolves the static
    // segment ahead of the parameter route. Gated like the faces above.
    let walk_state = Arc::clone(&state);
    let walk = axum::Router::new()
        .route(
            "/admin/v1/apis/walk-route",
            axum::routing::get(move |req: axum::extract::Request| {
                let state = Arc::clone(&walk_state);
                async move {
                    let ctx = rushwind_http_binding::ctx::RequestContext {
                        claims: req
                            .extensions()
                            .get::<auth::AuthClaims>()
                            .map(|c| c.0.clone()),
                        ..Default::default()
                    };
                    match crate::services::behaviors::walk_route_data(
                        &state,
                        &ctx,
                        pbjson_types::Empty {},
                    )
                    .await
                    {
                        Ok(resp) => {
                            let body = rushwind_http_binding::codec::serialize_response(
                                pool(),
                                "permission.service.v1.ListApiResponse",
                                &resp,
                                // The framework's redaction plan rides the
                                // generated mounts; this hand-mounted face
                                // answers a menu tree with nothing sensitive.
                                None,
                            )
                            .map_err(rushwind_http_binding::envelope::error_response)
                            .unwrap_or_default();
                            (
                                [(axum::http::header::CONTENT_TYPE, "application/json")],
                                body,
                            )
                                .into_response()
                        }
                        Err(e) => rushwind_http_binding::envelope::error_response(e),
                    }
                }
            }),
        )
        .layer(mk_hand_gate());
    app = app.merge(walk);

    // The audit-write layer: post-handler persistence (api + operation
    // logs), outermost so it sees final statuses.
    app.layer(axum::middleware::from_fn_with_state(
        Arc::clone(&state),
        crate::audit::layer,
    ))
}
