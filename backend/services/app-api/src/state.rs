//! The app (C-side) BFF state: Redis (the captcha store), the HS256
//! verification engine (the gate's first stage), and the tonic channel
//! to the core domain service (the ONLY data path — the BFF never
//! touches the database).

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
    pub core: proto::proto::authentication::service::v1::authentication_service_client::AuthenticationServiceClient<tonic::transport::Channel>,
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
        let core =
            proto::proto::authentication::service::v1::authentication_service_client::AuthenticationServiceClient::new(
                channel.clone(),
            );

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
    operator_of(ctx).map(|p| p.tenant_id as i64).unwrap_or(0)
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
