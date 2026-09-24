//! The audit-write layer — the post-handler persistence the reference's
//! logging middleware carries:
//!
//! * **api log** — every finished request (path/method/status/latency/
//!   operator/ip/request-id) into `sys_api_audit_logs`;
//! * **operation log** — the mutating methods only (POST/PUT/DELETE,
//!   login/captcha exempt) into `sys_operation_audit_logs`.
//!
//! Writes ride the core audit RPCs (fire-and-forget: audit failures
//! never fail the request), best-effort concurrent — the response
//! passes through untouched.

use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::state::AppState;
use crate::token::UserTokenPayload;

const STATUS_OK: i32 = 1;
const STATUS_FAILED: i32 = 2;

/// The audit layer entry — mounts outermost so it sees final statuses.
pub async fn layer(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| q.to_string()).unwrap_or_default();
    let query2 = query.clone();
    let headers = req.headers().clone();
    let claims = req
        .extensions()
        .get::<rushwind_authn::AuthClaims>()
        .cloned();
    let started = std::time::Instant::now();

    let response = next.run(req).await;

    // Fire-and-forget: never block or fail the response on audit writes.
    let status = response.status().as_u16().to_string();
    let latency_ms = started.elapsed().as_millis() as i64;
    let success = response.status().is_success();
    let operator = claims
        .as_ref()
        .and_then(|c| UserTokenPayload::from_claims(&c.0));
    let state2 = Arc::clone(&state);
    let method2 = method.clone();
    let path2 = path.clone();
    tokio::spawn(async move {
        let ip = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(',').next().unwrap_or("").trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let request_id = headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .unwrap_or_default();

        // The api log — every request.
        let api_log = proto::proto::audit::service::v1::ApiAuditLog {
            tenant_id: operator.as_ref().map(|o| o.tenant_id),
            user_id: operator.as_ref().map(|o| o.user_id),
            username: operator.as_ref().map(|o| o.username.clone()),
            ip_address: Some(ip.clone()),
            http_method: Some(method2.clone()),
            path: Some(path2.clone()),
            request_uri: if query2.is_empty() {
                Some(path2.clone())
            } else {
                Some(format!("{path2}?{query2}"))
            },
            api_module: Some(module_of(&path2)),
            request_id: Some(request_id.clone()),
            latency_ms: Some(latency_ms as u32),
            success: Some(success),
            status_code: Some(status.parse::<u32>().unwrap_or(500)),
            ..Default::default()
        };
        let mut core = proto::proto::audit::service::v1::api_audit_log_service_client::ApiAuditLogServiceClient::new(state2.core_channel.clone());
        let _ = core
            .create(tonic::Request::new(
                proto::proto::audit::service::v1::CreateApiAuditLogRequest {
                    data: Some(api_log),
                },
            ))
            .await;

        // The operation log — mutating methods only, session-only ops
        // exempt (login/captcha/refresh carry no operator yet).
        let is_mutating = matches!(method2.as_str(), "POST" | "PUT" | "DELETE" | "PATCH");
        let is_exempt = path2.contains("/login")
            || path2.contains("/captcha")
            || path2.contains("/refresh-token")
            || path2.contains("/logout");
        if is_mutating && !is_exempt {
            if let Some(op) = operator {
                let op_log = proto::proto::audit::service::v1::OperationAuditLog {
                    tenant_id: Some(op.tenant_id),
                    user_id: Some(op.user_id),
                    username: Some(op.username.clone()),
                    resource_type: Some(resource_of(&path2)),
                    action: Some(action_of(&method2)),
                    request_id: Some(request_id),
                    success: Some(success),
                    ip_address: Some(ip),
                    ..Default::default()
                };
                let mut core = proto::proto::audit::service::v1::operation_audit_log_service_client::OperationAuditLogServiceClient::new(state2.core_channel.clone());
                let _ = core
                    .create(tonic::Request::new(
                        proto::proto::audit::service::v1::CreateOperationAuditLogRequest {
                            data: Some(op_log),
                        },
                    ))
                    .await;
            }
        }
    });

    response
}

/// The login-attempt audit (called from the auth service after each
/// login verdict — success or failure alike).
pub struct LoginAudit<'a> {
    pub username: &'a str,
    pub user_id: u32,
    pub tenant_id: u32,
    pub success: bool,
    pub failure_reason: &'a str,
    pub ip: &'a str,
    pub request_id: &'a str,
}

pub async fn write_login_audit(state: &AppState, a: LoginAudit<'_>) {
    let log = proto::proto::audit::service::v1::LoginAuditLog {
        tenant_id: Some(a.tenant_id),
        user_id: Some(a.user_id),
        username: Some(a.username.to_string()),
        ip_address: Some(a.ip.to_string()),
        action_type: Some(1), // LOGIN
        status: Some(if a.success { STATUS_OK } else { STATUS_FAILED }),
        login_method: Some(1), // PASSWORD
        failure_reason: if a.success {
            None
        } else {
            Some(a.failure_reason.to_string())
        },
        request_id: Some(a.request_id.to_string()),
        ..Default::default()
    };
    let mut core = proto::proto::audit::service::v1::login_audit_log_service_client::LoginAuditLogServiceClient::new(state.core_channel.clone());
    let _ = core
        .create(tonic::Request::new(
            proto::proto::audit::service::v1::CreateLoginAuditLogRequest { data: Some(log) },
        ))
        .await;
}

fn module_of(path: &str) -> String {
    // /admin/v1/<module>/... → the module segment
    path.split('/')
        .filter(|s| !s.is_empty())
        .nth(2)
        .unwrap_or("admin")
        .to_string()
}

fn resource_of(path: &str) -> String {
    module_of(path)
}

fn action_of(method: &str) -> i32 {
    match method {
        "POST" => 1,
        "PUT" | "PATCH" => 2,
        "DELETE" => 3,
        _ => 0,
    }
}
