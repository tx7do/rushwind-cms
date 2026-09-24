//! The core application state: database, Redis, the HS256 engine, and
//! the per-client token stores. The core service is the ONLY process
//! touching the database.

use redis::aio::ConnectionManager;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::Config;
use crate::token::{ClientType, TokenStore};

pub struct AppState {
    pub cfg: Config,
    pub db: DatabaseConnection,
    #[allow(dead_code)]
    pub redis: ConnectionManager,
    /// The mint/verify engine — HS256, one secret.
    pub jwt: rushwind_authn_jwt::JwtAuthenticator,
    /// The per-client session stores (admin ct=0, app ct=1).
    pub admin_tokens: TokenStore,
    pub app_tokens: TokenStore,
}

impl AppState {
    pub async fn connect(cfg: Config) -> Result<Self, String> {
        let mut opts = ConnectOptions::new(cfg.database_source.clone());
        opts.max_connections(25)
            .min_connections(5)
            .connect_timeout(std::time::Duration::from_secs(10));
        let db = Database::connect(opts)
            .await
            .map_err(|e| format!("postgres connect: {e}"))?;

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

        let jwt = rushwind_authn_jwt::JwtAuthenticator::new(
            rushwind_authn_jwt::JwtOptions::new()
                .with_algorithm("HS256")
                .map_err(|e| format!("jwt algorithm: {e:?}"))?
                .with_key(cfg.jwt_key.as_bytes()),
        );

        let admin_tokens = TokenStore::new(
            redis_conn.clone(),
            ClientType::Admin,
            cfg.admin_access_secs,
            cfg.admin_refresh_secs,
        );
        let app_tokens = TokenStore::new(
            redis_conn.clone(),
            ClientType::App,
            cfg.app_access_secs,
            cfg.app_refresh_secs,
        );

        Ok(Self {
            cfg,
            db,
            redis: redis_conn,
            jwt,
            admin_tokens,
            app_tokens,
        })
    }

    /// The store of a client type.
    pub fn tokens(&self, client: ClientType) -> &TokenStore {
        match client {
            ClientType::Admin => &self.admin_tokens,
            ClientType::App => &self.app_tokens,
        }
    }
}

/// The tonic Status form of a DB failure.
pub fn db_status(e: sea_orm::DbErr) -> tonic::Status {
    tonic::Status::internal(format!("db: {e}"))
}

/// A bad-request Status.
pub fn bad(message: &str) -> tonic::Status {
    tonic::Status::invalid_argument(message.to_string())
}

/// A not-found Status.
pub fn not_found(message: &str) -> tonic::Status {
    tonic::Status::not_found(message.to_string())
}

/// A forbidden Status.
pub fn forbidden(message: &str) -> tonic::Status {
    tonic::Status::permission_denied(message.to_string())
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
