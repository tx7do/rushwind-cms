//! The app (C-side) REST route pack — mounts the full app BFF route
//! surface behind the service implementations (Phase 0: the generator's
//! null placeholders), splits the auth-free public subtree from the
//! gated one (the auth-free set), composes the per-route layer stack,
//! hand-mounts the C-side register route (the one app operation the
//! proto contract does not declare — the reference registers it the
//! same way, by hand), and hands the router to the lifecycle assembler
//! — whose edge block (the CORS policy and the request budget from the
//! server document) wraps the whole thing.
//!
//! Per-route layer composition (the `wrap` closure below): the framework
//! bind layer — body and query binding — outermost on EVERY route, and
//! the auth gate (the auth crate) composed INSIDE it on gated routes
//! only. That order is the wire contract: codec/binding failures answer
//! 400 ahead of any 401.

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
const PACKAGE: &str = "app.service.v1";

/// The adapter wiring the core service's ValidateToken RPC into the
/// auth gate's server-side check stage (client type 1 — the app
/// profile's key family).
struct CoreTokenChecker(std::sync::Arc<crate::state::AppState>);

#[async_trait::async_trait]
impl auth::AccessTokenChecker for CoreTokenChecker {
    async fn is_valid_access_token(&self, _uid: u32, _jti: &str, token: &str) -> bool {
        let mut core = self.0.core.clone();
        let Ok(resp) = core
            .validate_token(tonic::Request::new(
                proto::proto::authentication::service::v1::ValidateTokenRequest {
                    token: token.to_string(),
                    client_type: 1,
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
        false
    }
}

/// The app route pack: the mounted router (see the module docs).
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
    let checker = Arc::new(CoreTokenChecker(std::sync::Arc::clone(&state)))
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

    // The full mounted surface, null placeholder services behind it. Every
    // mount call threads both routers; the generator's auth-free table
    // classifies each of its route bindings.
    let mut router_pub = axum::Router::new();
    let mut router_gate = axum::Router::new();
    /// The uniform service mount: every service follows the same
    /// router-threading, null-construction, and wrap handshake.
    macro_rules! mount_services {
    ($($mount:ident => $ctor:expr),* $(,)?) => {
        $(
            (router_pub, router_gate) = proto::gen_app::mounts::$mount(
                router_pub,
                router_gate,
                $ctor,
                &wrap,
            );
        )*
    };
}

    mount_services!(
        mount_authentication_service => std::sync::Arc::new(
            crate::services::authentication::AuthenticationService {
                state: std::sync::Arc::clone(&state),
            },
        ),
        mount_file_transfer_service => proto::gen_app::nulls::null_file_transfer_service(),
        mount_user_profile_service => proto::gen_app::nulls::null_user_profile_service(),
        mount_navigation_service => std::sync::Arc::new(crate::services::proxies::NavigationProxy { state: std::sync::Arc::clone(&state) }),
        mount_site_service => std::sync::Arc::new(crate::services::proxies::SiteProxy { state: std::sync::Arc::clone(&state) }),
        mount_post_service => std::sync::Arc::new(crate::services::proxies::PostProxy { state: std::sync::Arc::clone(&state) }),
        mount_category_service => std::sync::Arc::new(crate::services::proxies::CategoryProxy { state: std::sync::Arc::clone(&state) }),
        mount_tag_service => std::sync::Arc::new(crate::services::proxies::TagProxy { state: std::sync::Arc::clone(&state) }),
        mount_page_service => std::sync::Arc::new(crate::services::proxies::PageProxy { state: std::sync::Arc::clone(&state) }),
        mount_comment_service => std::sync::Arc::new(crate::services::proxies::CommentProxy { state: std::sync::Arc::clone(&state) }),
        mount_interaction_service => std::sync::Arc::new(crate::services::proxies::InteractionProxy { state: std::sync::Arc::clone(&state) }),
    );

    // ── Hand-mounted: the C-side register route (POST /app/v1/register) ──
    // The one app operation the proto contract does not declare; the
    // reference mounts it by hand next to the generated registrations,
    // whitelisted ahead of the gate. The bind layer rides the same
    // input contract (`RegisterUserRequest`, body `*`); the Phase-0
    // handler answers the Unknown shape until the authentication
    // service implementation lands.
    let register_bind = axum::middleware::from_fn(move |req, next| {
        let fq = "authentication.service.v1.RegisterUserRequest";
        async move { bind_run(pool(), fq, true, REGISTERED_SUBTYPES, req, next).await }
    });
    let register_state = std::sync::Arc::clone(&state);
    let register = axum::Router::new().route(
        "/app/v1/register",
        axum::routing::post(
            move |axum::Extension(bound): axum::Extension<
                rushwind_http_binding::bindgate::BoundMessage,
            >| {
                let state = std::sync::Arc::clone(&register_state);
                async move {
                    use rushwind_http_binding::envelope::StatusError;
                    let err = |status: i32, reason: &'static str, msg: String| {
                        rushwind_http_binding::envelope::error_response(StatusError::new(
                            status, reason, msg,
                        ))
                    };
                    // The bind layer's DynamicMessage → the typed prost
                    // request (prost-reflect transcode).
                    let req: proto::proto::authentication::service::v1::RegisterUserRequest =
                        match bound.0.transcode_to() {
                            Ok(req) => req,
                            Err(_) => return err(400, "CODEC", "body unmarshal".into()),
                        };
                    let mut core = state.core.clone();
                    match core.register_user(tonic::Request::new(req)).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            let body = format!("{{\"userId\":{}}}", resp.user_id);
                            axum::response::IntoResponse::into_response((
                                [(axum::http::header::CONTENT_TYPE, "application/json")],
                                body,
                            ))
                        }
                        Err(e) => {
                            let reason = match e.code() {
                                tonic::Code::InvalidArgument => "BAD_REQUEST",
                                tonic::Code::AlreadyExists | tonic::Code::Aborted => "CONFLICT",
                                _ => "",
                            };
                            if reason.is_empty() {
                                err(500, "", e.message().into())
                            } else {
                                err(400, reason, e.message().into())
                            }
                        }
                    }
                }
            },
        ),
    );
    let register = register.layer(register_bind);

    router_pub.merge(router_gate).merge(register)
}
