//! The SSE notification server: the `/events` transport handler plus
//! the in-process notification hub (:6601).
//!
//! Wire behavior (the kratos sse transport the reference rides):
//! * `OPTIONS` preflight — 204 with a fixed CORS header set, answered
//!   before any authorization check;
//! * every authorize failure — 401 with the plain-text error line
//!   (`error: code = … reason = … message = … metadata = map[] cause =
//!   <nil>`, `text/plain; charset=utf-8`, `X-Content-Type-Options:
//!   nosniff`) and no CORS headers;
//! * token from `Authorization: Bearer`, `X-Token`, or `?token=`;
//! * access-token validation rides the same gate primitives: signature
//!   + expiry via the engine, Redis whitelist/blacklist via the store;
//! * `?stream=` must equal the token's userId (anti cross-user
//!   subscription);
//! * the live stream carries the transport-level CORS pair
//!   (`Access-Control-Allow-Origin: *`,
//!   `Access-Control-Allow-Headers: Content-Type`) on top of the
//!   content-type/cache-control the SSE body sets. The stream is silent
//!   when idle (no keep-alive pings);
//! * events: `notification`, id = GUIDv4 — each connection forwards
//!   only its own userId's payloads.

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::state::{status_error, AppState};

/// The in-process notification hub: senders publish (userId, json);
/// every SSE connection subscribed filters to its own stream.
#[derive(Clone)]
pub struct Hub {
    tx: broadcast::Sender<(u32, String)>,
}

impl Default for Hub {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }
}

impl Hub {
    /// Try-publish (non-blocking; a slow/full buffer drops for that
    /// receiver only).
    pub fn publish(&self, user_id: u32, payload: String) {
        let _ = self.tx.send((user_id, payload));
    }

    fn subscribe(&self) -> broadcast::Receiver<(u32, String)> {
        self.tx.subscribe()
    }
}

fn extract_token(headers: &HeaderMap, query_token: Option<&String>) -> Option<String> {
    if let Some(t) = rushwind_http_binding::ctx::bearer_token(headers) {
        return Some(t);
    }
    if let Some(v) = headers.get("x-token").and_then(|v| v.to_str().ok()) {
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    query_token.filter(|t| !t.is_empty()).cloned()
}

/// The OPTIONS preflight answer: a fixed CORS header set with no
/// authorization gate — the transport answers before the authorize
/// check.
async fn events_preflight() -> Response {
    (
        StatusCode::NO_CONTENT,
        [
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS"),
            (
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Content-Type, Authorization, X-Token, Last-Event-ID",
            ),
            (header::ACCESS_CONTROL_MAX_AGE, "86400"),
        ],
    )
        .into_response()
}

/// The authorize-failure shape: an Unauthorized status with the
/// plain-text error line and the nosniff marker, no CORS headers.
fn sse_error(err: crate::state::StatusError) -> Response {
    let code = err.status;
    let reason = err.reason;
    let message = &err.message;
    (
        StatusCode::UNAUTHORIZED,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        format!(
            "error: code = {code} reason = {reason} message = {message} metadata = map[] cause = <nil>\n"
        ),
    )
        .into_response()
}

/// GET /events — authorize, then hold the SSE stream open, forwarding
/// this user's notifications.
pub async fn events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = extract_token(&headers, params.get("token")) else {
        return sse_error(status_error("UNAUTHORIZED", "invalid token"));
    };

    // Signature + expiry via the engine, whitelist + blacklist via the
    // store — the authorize ValidateTokenRequest(ACCESS) sequence.
    let Ok(claims) = state.authenticator.authenticate_token(&token) else {
        return sse_error(status_error("UNAUTHORIZED", "invalid token"));
    };
    let Some(uid) = claims
        .0
        .get("uid")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
    else {
        return sse_error(status_error("UNAUTHORIZED", "invalid token"));
    };
    let Ok(_jti) = claims.get_jwt_id() else {
        return sse_error(status_error("UNAUTHORIZED", "invalid token"));
    };
    // The session verdict rides the core service's ValidateToken RPC
    // (both whitelist and blacklist fold into the one verdict).
    let mut core = state.core.clone();
    let valid = match core
        .validate_token(tonic::Request::new(
            proto::proto::authentication::service::v1::ValidateTokenRequest {
                token: token.clone(),
                client_type: 0,
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
        return sse_error(status_error(
            "UNAUTHORIZED",
            "access token is revoked or expired",
        ));
    }

    // The stream must be the token's own userId.
    if !params
        .get("stream")
        .and_then(|s| s.parse::<u32>().ok())
        .is_some_and(|s| s == uid)
    {
        return sse_error(status_error("FORBIDDEN", "stream user mismatch"));
    }

    let rx = state.hub.subscribe();
    let stream = futures_util::stream::unfold((rx, uid), |(mut rx, uid)| async move {
        loop {
            match rx.recv().await {
                Ok((user_id, payload)) => {
                    // Only this connection's own userId's payloads are
                    // forwarded — the per-stream subscription filter.
                    if user_id != uid || payload.is_empty() {
                        continue;
                    }
                    let event = Event::default()
                        .id(uuid::Uuid::new_v4().to_string())
                        .data(payload)
                        .event("notification");
                    return Some((Ok::<_, Infallible>(event), (rx, uid)));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    let mut response = Sse::new(stream).into_response();
    // The transport's SSE header pass: CORS pair + keep-alive on top of
    // the content-type/cache-control the SSE body already carries.
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type"),
    );
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    response
}

/// The sse transport wire (the factory's settings node): the listener
/// address and the events route path. A missing node or field falls
/// back to the defaults below.
#[derive(Debug, Deserialize)]
struct SseWire {
    #[serde(default = "default_addr")]
    addr: rushwind_bootstrap::BindWire,
    #[serde(default = "default_path")]
    path: String,
}

impl Default for SseWire {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            path: default_path(),
        }
    }
}

fn default_addr() -> rushwind_bootstrap::BindWire {
    rushwind_bootstrap::BindWire(std::net::SocketAddr::from(([0, 0, 0, 0], 6601)))
}

fn default_path() -> String {
    "/events".to_string()
}

/// The sse transport factory: assembles the events router (the handler
/// and its preflight under the wire's address and path) into a
/// lifecycle server.
pub fn factory(
    state: std::sync::Arc<crate::state::AppState>,
) -> impl Fn(
    serde_json::Value,
    rushwind_bootstrap::RouteInput,
) -> rushwind_bootstrap::BoxFuture<
    'static,
    Result<std::sync::Arc<dyn rushwind_transport::Server>, rushwind_bootstrap::BootstrapError>,
> + Send
       + Sync
       + 'static {
    move |settings, _input| {
        let state = std::sync::Arc::clone(&state);
        Box::pin(async move {
            let wire = crate::server::settings_or_default::<SseWire>(settings, "sse wire")?;
            let path = if wire.path.is_empty() {
                "/"
            } else {
                wire.path.as_str()
            };
            let router = axum::Router::new().route(
                path,
                axum::routing::get(events)
                    .options(events_preflight)
                    .with_state(state),
            );
            let server = rushwind_transport_axum::AxumServer::new(wire.addr.0, router)?;
            Ok(std::sync::Arc::new(server) as std::sync::Arc<dyn rushwind_transport::Server>)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        map
    }

    fn some(token: &str) -> Option<String> {
        Some(token.to_string())
    }

    #[test]
    fn token_extraction_prefers_bearer_then_x_token_then_the_query() {
        // Authorization leads over both other carriers.
        let map = headers(&[("authorization", "Bearer a"), ("x-token", "b")]);
        assert_eq!(
            extract_token(&map, some("c").as_ref()).as_deref(),
            Some("a")
        );
        // The lower-case scheme spelling reads the same.
        let map = headers(&[("authorization", "bearer a")]);
        assert_eq!(extract_token(&map, None).as_deref(), Some("a"));
        // X-Token stands in when no bearer rides.
        let map = headers(&[("x-token", "b")]);
        assert_eq!(
            extract_token(&map, some("c").as_ref()).as_deref(),
            Some("b")
        );
        // The query is the last carrier.
        assert_eq!(
            extract_token(&HeaderMap::new(), some("c").as_ref()).as_deref(),
            Some("c")
        );
        // Another scheme on Authorization falls through to the others.
        let map = headers(&[("authorization", "Basic zzz"), ("x-token", "b")]);
        assert_eq!(extract_token(&map, None).as_deref(), Some("b"));
        // Empty carriers never count.
        let map = headers(&[("x-token", "")]);
        assert_eq!(extract_token(&map, some("").as_ref()), None);
        assert_eq!(extract_token(&map, None), None);
        assert_eq!(extract_token(&HeaderMap::new(), None), None);
    }

    #[tokio::test]
    async fn authorize_failure_renders_the_plain_text_error_line() {
        let response = sse_error(status_error("UNAUTHORIZED", "invalid token"));
        // Every authorize failure is 401 on the wire ...
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            response
                .headers()
                .get(header::X_CONTENT_TYPE_OPTIONS)
                .unwrap(),
            "nosniff"
        );
        // ... with no CORS headers ...
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
            .is_none());
        // ... and the envelope's code / reason / message inside the line.
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            body.as_ref(),
            b"error: code = 401 reason = UNAUTHORIZED message = invalid token metadata = map[] cause = <nil>\n"
                .as_slice()
        );

        // A forbidden verdict keeps the 401 status; only the line carries 403.
        let response = sse_error(status_error("FORBIDDEN", "stream user mismatch"));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            body.as_ref(),
            b"error: code = 403 reason = FORBIDDEN message = stream user mismatch metadata = map[] cause = <nil>\n"
                .as_slice()
        );
    }

    #[tokio::test]
    async fn preflight_answers_204_with_the_fixed_cors_set() {
        let response = events_preflight().await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let headers = response.headers();
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "*"
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_METHODS).unwrap(),
            "GET, OPTIONS"
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_HEADERS).unwrap(),
            "Content-Type, Authorization, X-Token, Last-Event-ID"
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_MAX_AGE).unwrap(),
            "86400"
        );
    }

    #[tokio::test]
    async fn hub_fans_out_to_every_subscriber() {
        let hub = Hub::default();
        let mut a = hub.subscribe();
        // A clone shares the one channel.
        let mut b = hub.clone().subscribe();
        hub.publish(7, "{\"n\":1}".to_string());
        assert_eq!(a.recv().await.unwrap(), (7, "{\"n\":1}".to_string()));
        assert_eq!(b.recv().await.unwrap(), (7, "{\"n\":1}".to_string()));
        // Publishing with nobody listening is a harmless drop.
        Hub::default().publish(1, "x".to_string());
    }
}
