//! The app (C-side) BFF state: Redis (the captcha store), the HS256
//! verification engine (the gate's first stage), and the tonic channel
//! to the core domain service (the ONLY data path — the BFF never
//! touches the database).

use authenticationv1::authentication_service_client::AuthenticationServiceClient;
use proto::proto::authentication::service::v1 as authenticationv1;
use std::sync::Arc;

use redis::aio::ConnectionManager;

use crate::config::Config;

pub struct AppState {
    pub cfg: Config,
    pub redis: ConnectionManager,
    /// The verification engine the auth gate rides (shared secret).
    pub authenticator: Arc<dyn rushwind_authn::Authenticator>,
    /// The gRPC channel to the core domain service — the shared dial
    /// every per-service client clones.
    pub core_channel: tonic::transport::Channel,
    /// The authentication face (the login/logout/validate client).
    pub core: AuthenticationServiceClient<tonic::transport::Channel>,
    /// The SSE notification hub (`/events` subscribers).
    pub hub: crate::server::sse::Hub,
}

impl AppState {
    pub async fn connect(
        cfg: Config,
        authenticator: Arc<dyn rushwind_authn::Authenticator>,
    ) -> Result<Self, String> {
        let redis_source = if cfg.redis_password.is_empty() {
            format!("redis://{}/", cfg.redis_addr)
        } else {
            format!("redis://:{}@{}/", cfg.redis_password, cfg.redis_addr)
        };
        let client = redis::Client::open(redis_source).map_err(|e| format!("redis url: {e}"))?;
        let redis_conn = client
            .get_connection_manager()
            .await
            .map_err(|e| format!("redis connect: {e}"))?;

        let channel = tonic::transport::Endpoint::from_shared(cfg.core_addr.clone())
            .map_err(|e| format!("core endpoint: {e}"))?
            .connect()
            .await
            .map_err(|e| format!("core connect {}: {e}", cfg.core_addr))?;
        let core = AuthenticationServiceClient::new(channel.clone());

        Ok(Self {
            cfg,
            redis: redis_conn,
            authenticator,
            core_channel: channel,
            core,
            hub: crate::server::sse::Hub::default(),
        })
    }
}

/// The error envelope helper: reason → the admin error table's HTTP
/// status (`StatusError::new`).
pub fn status_error(
    reason: &'static str,
    message: impl Into<String>,
) -> rushwind_http_binding::envelope::StatusError {
    let status = proto::tables::error_status("app.service.v1", reason).unwrap_or(500);
    rushwind_http_binding::envelope::StatusError::new(status, reason, message)
}

/// The Unknown branch: 500 + empty reason.
pub fn internal_error(message: impl Into<String>) -> StatusError {
    rushwind_http_binding::envelope::internal_error(message)
}

pub type StatusError = rushwind_http_binding::envelope::StatusError;

/// The NOT_FOUND branch with the caller's resource label.
pub fn not_found(what: &str) -> StatusError {
    status_error("NOT_FOUND", format!("{what} not found"))
}

/// The verified operator extracted from the request context claims.
pub fn operator_of(
    ctx: &rushwind_http_binding::ctx::RequestContext,
) -> Result<store::auth::UserTokenPayload, StatusError> {
    ctx.claims
        .as_ref()
        .and_then(store::auth::UserTokenPayload::from_claims)
        .ok_or_else(|| status_error("UNAUTHORIZED", "missing identity"))
}

/// The operator's tenant, platform (0) when no identity rides.
pub fn tenant_of(ctx: &rushwind_http_binding::ctx::RequestContext) -> i64 {
    operator_of(ctx).map(|p| p.tenant_id).unwrap_or(0)
}

/// The required-payload extraction for create-style requests.
pub fn require_data<T>(data: Option<T>) -> Result<T, StatusError> {
    data.ok_or_else(|| status_error("BAD_REQUEST", "data required"))
}

/// timestamptz → protojson Timestamp.
pub fn ts_to_proto(
    value: chrono::DateTime<chrono::FixedOffset>,
) -> Option<pbjson_types::Timestamp> {
    Some(pbjson_types::Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    })
}

/// protojson Timestamp → timestamptz (UTC).
pub fn ts_from_proto(
    value: &pbjson_types::Timestamp,
) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    use chrono::TimeZone as _;
    chrono::Utc
        .timestamp_opt(value.seconds, value.nanos.max(0) as u32)
        .single()
        .map(|dt| dt.fixed_offset())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rushwind_http_binding::ctx::RequestContext;
    use serde_json::json;

    fn ctx_with_claims(claims: serde_json::Value) -> RequestContext {
        RequestContext {
            claims: claims.as_object().cloned(),
            ..Default::default()
        }
    }

    #[test]
    fn status_error_anchors_reasons_onto_the_owning_error_table() {
        let e = status_error("NOT_FOUND", "post not found");
        assert_eq!(
            (e.status, e.reason, e.message.as_str()),
            (404, "NOT_FOUND", "post not found")
        );
        // The progressive-captcha pair the login face answers with.
        for reason in ["CAPTCHA_REQUIRED", "CAPTCHA_INVALID"] {
            let e = status_error(reason, "m");
            assert_eq!(e.status, 400, "{reason}");
        }
        // The Unknown shape, and the caller's resource label.
        let e = internal_error("boom");
        assert_eq!((e.status, e.reason, e.message.as_str()), (500, "", "boom"));
        let e = not_found("post");
        assert_eq!(
            (e.status, e.reason, e.message.as_str()),
            (404, "NOT_FOUND", "post not found")
        );
        // A reason off the table collapses onto the 500 default — never a
        // guessed status.
        let e = status_error("MADE_UP", "x");
        assert_eq!((e.status, e.reason), (500, "MADE_UP"));
    }

    #[test]
    fn operator_of_reads_the_verified_identity_or_refuses() {
        let op = operator_of(&ctx_with_claims(
            json!({"uid": 7, "tid": 3, "sub": "alice", "jti": "jti-1"}),
        ))
        .unwrap();
        assert_eq!((op.user_id, op.tenant_id), (7, 3));
        assert_eq!(op.username, "alice");
        assert_eq!(op.jti, "jti-1");

        // No identity rides the context, or the bag carries none: the one
        // fixed refusal.
        for ctx in [
            RequestContext::default(),
            ctx_with_claims(json!({"sub": "alice"})),
            ctx_with_claims(json!({"uid": 0})),
            ctx_with_claims(json!({"uid": "7"})),
        ] {
            let err = operator_of(&ctx).unwrap_err();
            assert_eq!(
                (err.status, err.reason, err.message.as_str()),
                (401, "UNAUTHORIZED", "missing identity")
            );
        }
    }

    #[test]
    fn tenant_of_reads_the_operator_tenant_with_the_platform_default() {
        assert_eq!(tenant_of(&ctx_with_claims(json!({"uid": 7, "tid": 5}))), 5);
        // No tenant claim, or no identity at all: the platform scope.
        assert_eq!(tenant_of(&ctx_with_claims(json!({"uid": 7}))), 0);
        assert_eq!(tenant_of(&RequestContext::default()), 0);
    }

    #[test]
    fn require_data_unwraps_or_answers_bad_request() {
        assert_eq!(require_data(Some(3)).unwrap(), 3);
        let err = require_data::<i32>(None).unwrap_err();
        assert_eq!(
            (err.status, err.reason, err.message.as_str()),
            (400, "BAD_REQUEST", "data required")
        );
    }

    #[test]
    fn timestamps_round_trip_at_sub_second_precision() {
        use chrono::TimeZone as _;
        let value = chrono::Utc
            .timestamp_opt(1_792_430_205, 123_456_789)
            .single()
            .unwrap()
            .fixed_offset();
        let ts = ts_to_proto(value).unwrap();
        assert_eq!((ts.seconds, ts.nanos), (1_792_430_205, 123_456_789));
        assert_eq!(ts_from_proto(&ts).unwrap(), value);
        // Negative nanos (pre-epoch timestamps) clamp onto the whole second.
        let clamped = ts_from_proto(&pbjson_types::Timestamp {
            seconds: -3,
            nanos: -7,
        })
        .unwrap();
        assert_eq!(
            (clamped.timestamp(), clamped.timestamp_subsec_nanos()),
            (-3, 0)
        );
        // An out-of-range instant answers None instead of panicking.
        assert!(ts_from_proto(&pbjson_types::Timestamp {
            seconds: i64::MAX,
            nanos: 0
        })
        .is_none());
    }
}
