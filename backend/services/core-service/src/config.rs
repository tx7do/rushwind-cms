//! Config loading — parses the embedded yaml defaults
//! (`assets/data.yaml`, `assets/auth.yaml`) with env overrides, plus
//! the gRPC listener address (`CORE_GRPC_ADDR`, default :6602 — the
//! reference resolves core via registry; the port keeps a static
//! default for direct-dial deployments).

use rushwind_bootstrap::DurationWire;
use serde::Deserialize;

const DATA_YAML: &str = include_str!("../assets/data.yaml");
const AUTH_YAML: &str = include_str!("../assets/auth.yaml");

#[derive(Debug, Clone)]
pub struct Config {
    pub grpc_addr: String,
    pub database_source: String,
    pub redis_addr: String,
    pub redis_password: String,
    /// HS256 shared secret (verify + mint).
    pub jwt_key: String,
    /// Per-client expiries, read off the authenticator's admin / app
    /// sections (the reference's per-client profiles).
    pub admin_access_secs: i64,
    pub admin_refresh_secs: i64,
    pub app_access_secs: i64,
    pub app_refresh_secs: i64,
}

#[derive(Debug, Default, Deserialize)]
struct DataFile {
    data: Option<DataSection>,
}

#[derive(Debug, Default, Deserialize)]
struct DataSection {
    database: Option<DatabaseSection>,
    redis: Option<RedisSection>,
}

#[derive(Debug, Default, Deserialize)]
struct DatabaseSection {
    #[serde(default)]
    source: String,
}

#[derive(Debug, Default, Deserialize)]
struct RedisSection {
    #[serde(default)]
    addr: String,
    #[serde(default)]
    password: String,
}

#[derive(Debug, Deserialize)]
struct AuthFile {
    authenticator: Option<AuthenticatorSection>,
}

#[derive(Debug, Default, Deserialize)]
struct AuthenticatorSection {
    #[serde(default)]
    admin: Option<ClientSection>,
    #[serde(default)]
    app: Option<ClientSection>,
}

#[derive(Debug, Default, Deserialize)]
struct ClientSection {
    #[serde(default)]
    #[allow(dead_code)]
    method: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    access_token_expires: Option<DurationWire>,
    #[serde(default)]
    refresh_token_expires: Option<DurationWire>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let data: DataFile =
            serde_yaml::from_str(DATA_YAML).map_err(|e| format!("parse data.yaml: {e}"))?;
        let auth: AuthFile =
            serde_yaml::from_str(AUTH_YAML).map_err(|e| format!("parse auth.yaml: {e}"))?;
        let section = data.data.unwrap_or_default();
        let database = section.database.unwrap_or_default();
        let redis = section.redis.unwrap_or_default();
        let clients = auth.authenticator.unwrap_or_default();
        let admin = clients.admin.unwrap_or_default();
        let app = clients.app.unwrap_or_default();

        let secs = |d: &Option<DurationWire>, fallback: i64| {
            d.as_ref().map(|x| x.0.as_secs() as i64).unwrap_or(fallback)
        };

        Ok(Self {
            grpc_addr: std::env::var("CORE_GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:6602".into()),
            database_source: std::env::var("RUSHWIND_DATABASE_SOURCE").unwrap_or(database.source),
            redis_addr: std::env::var("RUSHWIND_REDIS_ADDR").unwrap_or(redis.addr),
            redis_password: std::env::var("RUSHWIND_REDIS_PASSWORD").unwrap_or(redis.password),
            jwt_key: std::env::var("jwt_signing_key").unwrap_or(admin.key.clone()),
            admin_access_secs: secs(&admin.access_token_expires, 5400),
            admin_refresh_secs: secs(&admin.refresh_token_expires, 43200),
            app_access_secs: secs(&app.access_token_expires, 900),
            app_refresh_secs: secs(&app.refresh_token_expires, 0),
        })
    }
}
