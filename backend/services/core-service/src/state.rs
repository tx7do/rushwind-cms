//! The core application state: database, Redis, the HS256 engine, the
//! per-client token stores, and the optional object store endpoint.
//! The core service is the ONLY process touching the database and the
//! object store.

use redis::aio::ConnectionManager;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::{Config, OssConfig};
use crate::token::{ClientType, TokenStore};

/// The MinIO object store endpoint (the config-selected backend;
/// `None` falls the storage faces to the local disk store).
pub struct OssStore {
    pub client: minio::s3::MinioClient,
    pub download_host: String,
}

impl OssStore {
    fn connect(cfg: &OssConfig) -> Result<Self, String> {
        let url = format!(
            "{}://{}",
            if cfg.use_ssl { "https" } else { "http" },
            cfg.endpoint
        );
        let base_url: minio::s3::http::BaseUrl =
            url.parse().map_err(|e| format!("minio base url: {e:?}"))?;
        let provider =
            minio::s3::creds::StaticProvider::new(&cfg.access_key, &cfg.secret_key, None);
        let client = minio::s3::MinioClient::new(base_url, Some(provider), None, None)
            .map_err(|e| format!("minio client: {e:?}"))?;
        Ok(Self {
            client,
            download_host: cfg.download_host.clone(),
        })
    }
}

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
    /// The object store endpoint, when the config selects one.
    pub oss: Option<OssStore>,
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

        let oss = match &cfg.oss {
            Some(oss_cfg) => Some(OssStore::connect(oss_cfg)?),
            None => None,
        };

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
            oss,
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

/// The repo-layer result alias (the tonic Status error channel).
pub type StatusResult<T> = Result<T, tonic::Status>;

/// NOT_FOUND with the resource label (the repo-layer form).
pub fn not_found_status(what: &str) -> tonic::Status {
    tonic::Status::not_found(format!("{what} not found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(nanos: u32) -> chrono::DateTime<chrono::FixedOffset> {
        chrono::NaiveDate::from_ymd_opt(2026, 11, 23)
            .unwrap()
            .and_hms_nano_opt(8, 30, 5, nanos)
            .unwrap()
            .and_utc()
            .fixed_offset()
    }

    // ── timestamp bridge ─────────────────────────────────────────────

    #[test]
    fn timestamps_round_trip_at_sub_second_precision() {
        for nanos in [0, 1, 123_456_789, 999_999_999] {
            let value = at(nanos);
            let ts = ts_to_proto(value).unwrap();
            assert_eq!((ts.seconds, ts.nanos), (value.timestamp(), nanos as i32));
            // The bridge answers UTC; equality is by instant.
            let back = ts_from_proto(&ts).unwrap();
            assert_eq!(back, value, "{nanos}");
            assert_eq!(back.offset().local_minus_utc(), 0);
        }
        // The offset of the source does not survive — only the instant.
        let cst = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let shifted = at(500_000_000).with_timezone(&cst);
        assert_eq!(shifted.offset().local_minus_utc(), 8 * 3600);
        assert_eq!(
            ts_from_proto(&ts_to_proto(shifted).unwrap()).unwrap(),
            at(500_000_000)
        );
    }

    #[test]
    fn timestamp_bridge_normalizes_the_edges() {
        // Timestamps before the epoch carry negative nanos in proto — the
        // bridge clamps them onto the whole second instead of panicking.
        let before = ts_from_proto(&pbjson_types::Timestamp {
            seconds: 1_700_000_000,
            nanos: -5,
        })
        .unwrap();
        assert_eq!(
            (before.timestamp(), before.timestamp_subsec_nanos()),
            (1_700_000_000, 0)
        );
        // The zero Timestamp is the epoch; the far ends answer None.
        assert_eq!(
            ts_from_proto(&pbjson_types::Timestamp::default())
                .unwrap()
                .timestamp(),
            0
        );
        assert!(ts_from_proto(&pbjson_types::Timestamp {
            seconds: i64::MAX,
            nanos: 0
        })
        .is_none());
        assert!(ts_from_proto(&pbjson_types::Timestamp {
            seconds: i64::MIN,
            nanos: 0
        })
        .is_none());
    }

    // ── status helpers ───────────────────────────────────────────────

    #[test]
    fn status_helpers_carry_their_codes_and_texts() {
        // The DB error keeps sea-orm's own label inside the message.
        let db = db_status(sea_orm::DbErr::Custom("boom".into()));
        assert_eq!(db.code(), tonic::Code::Internal);
        assert_eq!(db.message(), "db: Custom Error: boom");

        let e = bad("data required");
        assert_eq!(
            (e.code(), e.message()),
            (tonic::Code::InvalidArgument, "data required")
        );
        let e = not_found("role");
        assert_eq!((e.code(), e.message()), (tonic::Code::NotFound, "role"));
        let e = not_found_status("role");
        assert_eq!(
            (e.code(), e.message()),
            (tonic::Code::NotFound, "role not found")
        );
        let e = forbidden("platform admin only");
        assert_eq!(
            (e.code(), e.message()),
            (tonic::Code::PermissionDenied, "platform admin only")
        );
    }
}
