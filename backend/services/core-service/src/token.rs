//! The Redis-backed token store of the core service — the key families
//! of the reference's `user_token_cache.go` (`gwc:at/rt/bl:`), the
//! verify-and-revoke rotation script, and the scan-based user
//! revocation. One store per client type (admin 0 / app 1), each with
//! its own expiry profile.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// The client type discriminant — `authentication.service.v1.ClientType`.
#[derive(Clone, Copy, Debug)]
pub enum ClientType {
    Admin,
    App,
}

impl ClientType {
    pub fn from_i32(v: i32) -> Self {
        if v == 1 {
            ClientType::App
        } else {
            ClientType::Admin
        }
    }

    fn as_u32(self) -> u32 {
        match self {
            ClientType::Admin => 0,
            ClientType::App => 1,
        }
    }
}

/// The refresh rotation script — the reference's
/// verifyAndRevokeRefreshTokenScript: verify RT → delete RT → delete AT,
/// atomically.
const VERIFY_AND_REVOKE: &str = r#"
local rtKey = KEYS[1]
local atKey = KEYS[2]
local refreshToken = ARGV[1]

local stored = redis.call('GET', rtKey)
if not stored or stored ~= refreshToken then
    return 0
end

redis.call('DEL', rtKey)
redis.call('DEL', atKey)
return 1
"#;

#[derive(Clone)]
pub struct TokenStore {
    conn: ConnectionManager,
    client: ClientType,
    access_secs: i64,
    refresh_secs: i64,
}

impl TokenStore {
    pub fn new(
        conn: ConnectionManager,
        client: ClientType,
        access_secs: i64,
        refresh_secs: i64,
    ) -> Self {
        Self {
            conn,
            client,
            access_secs,
            refresh_secs,
        }
    }

    pub fn access_secs(&self) -> i64 {
        self.access_secs
    }

    pub fn refresh_secs(&self) -> i64 {
        self.refresh_secs
    }

    fn at_key(&self, uid: u32, jti: &str) -> String {
        format!("gwc:at:{}:{}:{}", self.client.as_u32(), uid, jti)
    }

    fn rt_key(&self, uid: u32, jti: &str) -> String {
        format!("gwc:rt:{}:{}:{}", self.client.as_u32(), uid, jti)
    }

    fn bl_key(jti: &str) -> String {
        format!("gwc:bl:{jti}")
    }

    /// Whitelist exact-match — false means revoked/expired.
    pub async fn is_valid_access_token(&self, uid: u32, jti: &str, token: &str) -> bool {
        let Ok(stored): Result<String, _> = self.conn.clone().get(self.at_key(uid, jti)).await
        else {
            return false;
        };
        stored == token
    }

    /// Blacklist existence.
    pub async fn is_blocked(&self, jti: &str) -> bool {
        let Ok(n): Result<i64, _> = self.conn.clone().exists(Self::bl_key(jti)).await else {
            return false;
        };
        n > 0
    }

    /// Registers a fresh token pair (the mint path). A disabled refresh
    /// expiry (0s — the app profile) stores no RT row.
    pub async fn add_token_pair(
        &self,
        uid: u32,
        jti: &str,
        access: &str,
        refresh: Option<&str>,
    ) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_ex(
                self.at_key(uid, jti),
                access,
                self.access_secs.max(1) as u64,
            )
            .await
            .map_err(|e| e.to_string())?;
        if let Some(refresh) = refresh.filter(|_| self.refresh_secs > 0) {
            let _: () = conn
                .set_ex(self.rt_key(uid, jti), refresh, self.refresh_secs as u64)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// The refresh rotation: verify-and-revoke atomically.
    pub async fn verify_and_revoke_pair(
        &self,
        uid: u32,
        jti: &str,
        refresh: &str,
    ) -> Result<bool, String> {
        let mut conn = self.conn.clone();
        let script = redis::Script::new(VERIFY_AND_REVOKE);
        let result: i64 = script
            .key(self.rt_key(uid, jti))
            .key(self.at_key(uid, jti))
            .arg(refresh)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result == 1)
    }

    /// Revokes every token of the user (the logout path).
    pub async fn revoke_user_tokens(&self, uid: u32) -> Result<(), String> {
        let mut conn = self.conn.clone();
        for prefix in ["at", "rt"] {
            let pattern = format!("gwc:{prefix}:{}:{}:*", self.client.as_u32(), uid);
            let mut cursor: u64 = 0;
            loop {
                let (next, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| e.to_string())?;
                if !keys.is_empty() {
                    let _: () = conn.del(keys).await.map_err(|e| e.to_string())?;
                }
                cursor = next;
                if cursor == 0 {
                    break;
                }
            }
        }
        Ok(())
    }

    /// A fresh opaque refresh token (random hex).
    pub fn new_refresh_token() -> String {
        use rand::RngCore as _;
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
}
