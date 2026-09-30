//! The SSE notification server — the deployment face (:6701). The
//! wire contract (preflight, token extraction, the error line, the
//! stream filtering, the transport headers) lives in
//! `rushwind-transport-sse`; this file carries the app BFF's authorize
//! sequence as a Gate adapter — signature + expiry via the local JWT
//! engine, then the core service's remote ValidateToken.

use authenticationv1::authentication_service_client::AuthenticationServiceClient;
use proto::proto::authentication::service::v1 as authenticationv1;
use std::sync::Arc;

use crate::state::{status_error, AppState, StatusError};
use rushwind_transport_sse::{Failure, Gate, SseApp};

pub use rushwind_transport_sse::Hub;

/// The app BFF authorize sequence: signature + expiry via the engine,
/// then the core service's ValidateToken (client_type app, the Access
/// category).
struct CoreGate {
    authenticator: Arc<dyn rushwind_authn::Authenticator>,
    core: AuthenticationServiceClient<tonic::transport::Channel>,
}

impl Gate for CoreGate {
    async fn authorize(&self, token: &str) -> Result<u32, Failure> {
        let Ok(claims) = self.authenticator.authenticate_token(token) else {
            return Err(Failure::new("UNAUTHORIZED", "invalid token"));
        };
        let Some(uid) = claims
            .0
            .get("uid")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
        else {
            return Err(Failure::new("UNAUTHORIZED", "invalid token"));
        };
        if claims.get_jwt_id().is_err() {
            return Err(Failure::new("UNAUTHORIZED", "invalid token"));
        }
        let mut core = self.core.clone();
        let valid = match core
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
        {
            Ok(resp) => resp.into_inner().is_valid,
            Err(_) => false,
        };
        if !valid {
            return Err(Failure::new(
                "UNAUTHORIZED",
                "access token is revoked or expired",
            ));
        }
        Ok(uid)
    }
}

/// The status-table constructor the transport calls for every failure
/// envelope (reason → the app error table's HTTP status).
fn sse_status_error(reason: &'static str, message: String) -> StatusError {
    status_error(reason, message)
}

/// The sse transport factory: the framework router under the wire's
/// address and path, with the deployment's listener defaults.
pub fn factory(
    state: std::sync::Arc<AppState>,
) -> impl Fn(
    serde_json::Value,
    rushwind_bootstrap::RouteInput,
) -> rushwind_bootstrap::BoxFuture<
    'static,
    Result<std::sync::Arc<dyn rushwind_transport::Server>, rushwind_bootstrap::BootstrapError>,
> + Send
       + Sync
       + 'static {
    let app = Arc::new(SseApp {
        hub: state.hub.clone(),
        gate: Arc::new(CoreGate {
            authenticator: Arc::clone(&state.authenticator),
            core: state.core.clone(),
        }),
        status_error: sse_status_error,
    });
    rushwind_transport_sse::factory(
        app,
        rushwind_transport_sse::SseWire {
            addr: Some(rushwind_bootstrap::BindWire(std::net::SocketAddr::from((
                [0, 0, 0, 0],
                6601,
            )))),
            path: Some("/events".to_string()),
        },
    )
}
