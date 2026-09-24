//! Config loading — parses the embedded yaml defaults
//! (`assets/data.yaml`, `assets/auth.yaml`, `assets/oss.yaml`) with env
//! overrides, plus the gRPC listener address (`CORE_GRPC_ADDR`,
//! default :6602 — the reference resolves core via registry; the port
//! keeps a static default for direct-dial deployments).

use rushwind_bootstrap::DurationWire;
use serde::Deserialize;

const DATA_YAML: &str = include_str!("../assets/data.yaml");
const AUTH_YAML: &str = include_str!("../assets/auth.yaml");
const OSS_YAML: &str = include_str!("../assets/oss.yaml");

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
    /// The object store (the reference's oss.yaml minio section);
    /// `None` falls the storage faces back to the local disk store.
    pub oss: Option<OssConfig>,
}

/// The MinIO object store endpoint profile.
#[derive(Debug, Clone)]
pub struct OssConfig {
    pub endpoint: String,
    pub download_host: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
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
struct OssFile {
    oss: Option<OssSection>,
}

#[derive(Debug, Deserialize)]
struct OssSection {
    #[serde(default)]
    minio: Option<MinioSection>,
}

#[derive(Debug, Default, Deserialize)]
struct MinioSection {
    #[serde(default)]
    endpoint: String,
    #[serde(default)]
    download_host: String,
    #[serde(default)]
    access_key: String,
    #[serde(default)]
    secret_key: String,
    #[serde(default)]
    use_ssl: bool,
}

#[derive(Debug, Deserialize)]
struct AuthFile {
    authenticator: Option<AuthenticatorSection>,
}

/// The object-store profile of a parsed oss.yaml — the minio section,
/// dropped when its endpoint is blank (the section's presence alone
/// selects the provider; an empty endpoint means "not configured").
fn minio_section_of(file: OssFile) -> Option<MinioSection> {
    file.oss
        .and_then(|o| o.minio)
        .filter(|m| !m.endpoint.is_empty())
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
        let oss: OssFile =
            serde_yaml::from_str(OSS_YAML).map_err(|e| format!("parse oss.yaml: {e}"))?;
        let section = data.data.unwrap_or_default();
        let database = section.database.unwrap_or_default();
        let redis = section.redis.unwrap_or_default();
        let clients = auth.authenticator.unwrap_or_default();
        let admin = clients.admin.unwrap_or_default();
        let app = clients.app.unwrap_or_default();
        let minio = minio_section_of(oss);

        let secs = |d: &Option<DurationWire>, fallback: i64| {
            d.as_ref().map(|x| x.0.as_secs() as i64).unwrap_or(fallback)
        };

        // The object store profile: the embedded dev section carries the
        // reference's local MinIO; the env vars override its values (the
        // section's presence alone selects the provider).
        let oss = minio.map(|m| OssConfig {
            endpoint: std::env::var("RUSHWIND_MINIO_ENDPOINT").unwrap_or(m.endpoint),
            download_host: std::env::var("RUSHWIND_MINIO_DOWNLOAD_HOST").unwrap_or(m.download_host),
            access_key: std::env::var("RUSHWIND_MINIO_ACCESS_KEY").unwrap_or(m.access_key),
            secret_key: std::env::var("RUSHWIND_MINIO_SECRET_KEY").unwrap_or(m.secret_key),
            use_ssl: m.use_ssl,
        });

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
            oss,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_yaml_full_section_parses_every_field() {
        let file: OssFile = serde_yaml::from_str(
            "oss:\n\
             \x20 minio:\n\
             \x20   endpoint: \"127.0.0.1:9000\"\n\
             \x20   download_host: \"http://localhost:9000\"\n\
             \x20   access_key: \"root\"\n\
             \x20   secret_key: \"s3cret\"\n\
             \x20   use_ssl: true\n",
        )
        .unwrap();
        let m = minio_section_of(file).expect("endpoint present");
        assert_eq!(m.endpoint, "127.0.0.1:9000");
        assert_eq!(m.download_host, "http://localhost:9000");
        assert_eq!(m.access_key, "root");
        assert_eq!(m.secret_key, "s3cret");
        assert!(m.use_ssl);
    }

    #[test]
    fn oss_yaml_missing_fields_take_defaults_and_unknown_keys_are_ignored() {
        // The embedded asset carries extra keys (upload_host, token)
        // the section struct does not model — serde drops them.
        let file: OssFile = serde_yaml::from_str(
            "oss:\n  minio:\n    endpoint: e\n    upload_host: \"x\"\n    token: \"\"\n",
        )
        .unwrap();
        let m = minio_section_of(file).expect("endpoint present");
        assert_eq!(m.endpoint, "e");
        assert_eq!(m.download_host, "");
        assert_eq!(m.access_key, "");
        assert_eq!(m.secret_key, "");
        assert!(!m.use_ssl);
    }

    #[test]
    fn oss_yaml_blank_endpoint_and_missing_sections_filter_out() {
        // An empty endpoint means "not configured" — the section is
        // dropped and the storage faces fall back to the local disk.
        let file: OssFile = serde_yaml::from_str("oss:\n  minio:\n    endpoint: \"\"\n").unwrap();
        assert!(minio_section_of(file).is_none());

        let file: OssFile = serde_yaml::from_str("{}").unwrap();
        assert!(minio_section_of(file).is_none());

        let file: OssFile = serde_yaml::from_str("oss: {}").unwrap();
        assert!(minio_section_of(file).is_none());
    }

    #[test]
    fn embedded_oss_yaml_selects_the_local_minio_profile() {
        let file: OssFile = serde_yaml::from_str(OSS_YAML).unwrap();
        let m = minio_section_of(file).expect("the embedded dev section is configured");
        assert_eq!(m.endpoint, "127.0.0.1:9000");
        assert_eq!(m.download_host, "http://127.0.0.1:9000");
        assert_eq!(m.access_key, "root");
        assert_eq!(m.secret_key, "*Abcd123456");
        assert!(!m.use_ssl);
    }
}
