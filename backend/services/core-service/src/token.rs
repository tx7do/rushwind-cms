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

    /// The strict read — only the two declared enum values; anything
    /// else is an unknown family (the reference's getAuthenticator
    /// rejects them with "invalid client type").
    pub fn from_i32_strict(v: i32) -> Option<Self> {
        match v {
            0 => Some(ClientType::Admin),
            1 => Some(ClientType::App),
            _ => None,
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
        for prefix in ["at", "rt"] {
            let keys = self
                .scan_keys(&format!("gwc:{prefix}:{}:{}:*", self.client.as_u32(), uid))
                .await?;
            if keys.is_empty() {
                continue;
            }
            let mut conn = self.conn.clone();
            let _: () = conn.del(keys).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Every live access-token value of the user — the session family
    /// scan of the reference's GetAccessTokens (SCAN + MGET, expired
    /// rows dropped).
    pub async fn access_tokens(&self, uid: u32) -> Result<Vec<String>, String> {
        let keys = self
            .scan_keys(&format!("gwc:at:{}:{}:*", self.client.as_u32(), uid))
            .await?;
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.conn.clone();
        let values: Vec<Option<String>> = conn.mget(&keys).await.map_err(|e| e.to_string())?;
        Ok(values.into_iter().flatten().collect())
    }

    /// Resolves the jti whose stored access token equals `token` — the
    /// reference's scanFindValue (the by-token blacklist target).
    pub async fn jti_of_access_token(
        &self,
        uid: u32,
        token: &str,
    ) -> Result<Option<String>, String> {
        let keys = self
            .scan_keys(&format!("gwc:at:{}:{}:*", self.client.as_u32(), uid))
            .await?;
        if keys.is_empty() {
            return Ok(None);
        }
        let mut conn = self.conn.clone();
        let values: Vec<Option<String>> = conn.mget(&keys).await.map_err(|e| e.to_string())?;
        for (key, value) in keys.into_iter().zip(values) {
            if value.as_deref() == Some(token) {
                return Ok(key.rsplit(':').next().map(str::to_owned));
            }
        }
        Ok(None)
    }

    /// Whether an access-token row exists for the jti (the reference's
    /// IsExistAccessTokenByJti).
    pub async fn has_access_token(&self, uid: u32, jti: &str) -> Result<bool, String> {
        let mut conn = self.conn.clone();
        let n: i64 = conn
            .exists(self.at_key(uid, jti))
            .await
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }

    /// Blacklists the jti with the reason as the value; a positive ttl
    /// expires the row, an absent one persists it (the reference's
    /// SET-with-zero-expiration form).
    pub async fn block_token(
        &self,
        jti: &str,
        reason: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let _: () = match ttl_secs {
            Some(secs) => conn.set_ex(Self::bl_key(jti), reason, secs).await,
            None => conn.set(Self::bl_key(jti), reason).await,
        }
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Revokes one token pair (access + refresh) of the jti — the
    /// reference's RevokeTokenByJti (the unblock face removes the
    /// session rows, the blacklist row itself just ages out).
    pub async fn revoke_by_jti(&self, uid: u32, jti: &str) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .del(vec![self.at_key(uid, jti), self.rt_key(uid, jti)])
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// SCAN-walks a key pattern to completion (the reference's scanKeys
    /// — cursor loop, no KEYS).
    async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>, String> {
        let mut conn = self.conn.clone();
        let mut keys = Vec::new();
        let mut cursor: u64 = 0;
        loop {
            let (next, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| e.to_string())?;
            keys.extend(batch);
            cursor = next;
            if cursor == 0 {
                break;
            }
        }
        Ok(keys)
    }

    /// A fresh opaque refresh token (random hex).
    pub fn new_refresh_token() -> String {
        use rand::RngCore as _;
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
}
