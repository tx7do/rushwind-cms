//! Config loading — parses the embedded yaml defaults
//! (`assets/data.yaml`, `assets/auth.yaml`, compiled into the binary)
//! with env overrides (`RUSHWIND_DATABASE_SOURCE` /
//! `RUSHWIND_REDIS_ADDR` / `RUSHWIND_REDIS_PASSWORD`, plus the shared
//! secret through `jwt_signing_key` — the same env var name the Go
//! reference's `${jwt_signing_key:...}` placeholder resolves). The
//! server assembly document (`assets/server.yaml`) belongs to the
//! lifecycle assembler.

use rushwind_bootstrap::DurationWire;
use serde::Deserialize;

const DATA_YAML: &str = include_str!("../assets/data.yaml");
const AUTH_YAML: &str = include_str!("../assets/auth.yaml");

/// Which client section of auth.yaml this deployment eats — the admin
/// BFF reads `admin`, the app BFF reads `app` (mirroring the reference's
/// authenticator.yaml).
pub const AUTH_SECTION: &str = "app";

#[derive(Debug, Clone)]
pub struct Config {
    /// The core domain service's gRPC endpoint (static dial).
    pub core_addr: String,
    #[allow(dead_code)]
    pub database_source: String,
    pub redis_addr: String,
    pub redis_password: String,
    /// HS256 shared secret (verify + mint) — the client section's key or
    /// the `jwt_signing_key` env override.
    pub jwt_key: String,
    pub access_token_expires_secs: i64,
    pub refresh_token_expires_secs: i64,
    /// The login-password AES key (application-layer encryption).
    #[allow(dead_code)]
    pub aes_key: String,
}

#[derive(Debug, Deserialize)]
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
    #[allow(dead_code)]
    driver: String,
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
    #[serde(default)]
    aes_key: String,
}

impl Config {
    /// Loads the vendored yaml files with env overrides applied.
    pub fn load() -> Result<Self, String> {
        let data: DataFile =
            serde_yaml::from_str(DATA_YAML).map_err(|e| format!("parse data.yaml: {e}"))?;
        let auth: AuthFile =
            serde_yaml::from_str(AUTH_YAML).map_err(|e| format!("parse auth.yaml: {e}"))?;

        let data_section = data.data.unwrap_or_default();
        let database = data_section.database.unwrap_or_default();
        let redis_section = data_section.redis.unwrap_or_default();
        let client = auth
            .authenticator
            .and_then(|a| match AUTH_SECTION {
                "app" => a.app,
                _ => a.admin,
            })
            .unwrap_or_default();

        // The code defaults: access 15 min, refresh 7 days — the same
        // fallbacks the reference's jwt layer carries.
        let access_token_expires_secs = env_int(
            "RUSHWIND_ACCESS_TOKEN_EXPIRES_SECS",
            client
                .access_token_expires
                .as_ref()
                .map(|d| d.0.as_secs() as i64),
            900,
        );
        let refresh_token_expires_secs = env_int(
            "RUSHWIND_REFRESH_TOKEN_EXPIRES_SECS",
            client
                .refresh_token_expires
                .as_ref()
                .map(|d| d.0.as_secs() as i64),
            7 * 24 * 3600,
        );

        Ok(Config {
            core_addr: env_string("CORE_GRPC_ADDR", "http://127.0.0.1:6602".into()),
            database_source: env_string("RUSHWIND_DATABASE_SOURCE", database.source),
            redis_addr: env_string("RUSHWIND_REDIS_ADDR", redis_section.addr),
            redis_password: env_string("RUSHWIND_REDIS_PASSWORD", redis_section.password),
            jwt_key: env_string("jwt_signing_key", client.key),
            access_token_expires_secs,
            refresh_token_expires_secs,
            aes_key: client.aes_key,
        })
    }
}

/// The env-override forms the loader uses. Each returns the yaml (or
/// code) default unless the environment carries a value of the expected
/// shape.
fn env_int(name: &str, yaml: Option<i64>, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .or(yaml)
        .unwrap_or(default)
}

fn env_string(name: &str, yaml: String) -> String {
    std::env::var(name).unwrap_or(yaml)
}

#[cfg(test)]
mod tests {
    use super::{env_int, env_string};
    use std::env::{remove_var, set_var};

    // Unique names so parallel tests never collide on the shared
    // environment.
    const INT: &str = "RUSHWIND_TEST_ENV_HELPER_INT_APP";
    const STRING: &str = "RUSHWIND_TEST_ENV_HELPER_STRING_APP";

    #[test]
    fn env_int_takes_parseable_override_else_yaml_else_default() {
        set_var(INT, "1234");
        assert_eq!(env_int(INT, None, 42), 1234);
        set_var(INT, "not-a-number");
        assert_eq!(env_int(INT, Some(7), 42), 7);
        remove_var(INT);
        assert_eq!(env_int(INT, None, 42), 42);
    }

    #[test]
    fn env_string_takes_set_value_verbatim_else_yaml() {
        set_var(STRING, "");
        assert_eq!(env_string(STRING, "yaml".into()), "");
        remove_var(STRING);
        assert_eq!(env_string(STRING, "yaml".into()), "yaml");
    }
}
