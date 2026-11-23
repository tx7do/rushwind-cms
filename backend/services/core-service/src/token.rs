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

    fn at_key(&self, uid: i64, jti: &str) -> String {
        format!("gwc:at:{}:{}:{}", self.client.as_u32(), uid, jti)
    }

    fn rt_key(&self, uid: i64, jti: &str) -> String {
        format!("gwc:rt:{}:{}:{}", self.client.as_u32(), uid, jti)
    }

    fn bl_key(jti: &str) -> String {
        format!("gwc:bl:{jti}")
    }

    /// Whitelist exact-match — false means revoked/expired.
    pub async fn is_valid_access_token(&self, uid: i64, jti: &str, token: &str) -> bool {
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
        uid: i64,
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
        uid: i64,
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
    pub async fn revoke_user_tokens(&self, uid: i64) -> Result<(), String> {
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
    pub async fn access_tokens(&self, uid: i64) -> Result<Vec<String>, String> {
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
        uid: i64,
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
    pub async fn has_access_token(&self, uid: i64, jti: &str) -> Result<bool, String> {
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
    pub async fn revoke_by_jti(&self, uid: i64, jti: &str) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Live-Redis pin for the store faces (the key families, the
    /// whitelist, the verify-and-revoke rotation, the blacklist, the
    /// scans). Skips unless `RUSHWIND_TEST_REDIS_URL` carries a DSN — CI
    /// runs without one and passes vacuously.
    async fn redis_of() -> Option<ConnectionManager> {
        let url = std::env::var("RUSHWIND_TEST_REDIS_URL").ok()?;
        let client = redis::Client::open(url).ok()?;
        client.get_connection_manager().await.ok()
    }

    /// A per-run user id. The scan-based faces walk
    /// `gwc:<family>:<ct>:<uid>:*`, so every run draws its own id — a
    /// live session's family is never touched.
    fn test_uid() -> i64 {
        (uuid::Uuid::now_v7().as_u128() as u64 & 0x3FFF_FFFF_FFFF_FFFF) as i64 | 1
    }

    fn test_jti() -> String {
        format!("ut_{}", uuid::Uuid::now_v7().simple())
    }

    /// Sweeps whatever a test wrote — both families plus the blacklist row
    /// (the one row that never expires on its own).
    async fn sweep(admin: &TokenStore, app: &TokenStore, uid: i64, jti: &str) {
        let _ = admin.revoke_user_tokens(uid).await;
        let _ = app.revoke_user_tokens(uid).await;
        let mut conn = admin.conn.clone();
        let _: Result<i64, _> = conn.del(TokenStore::bl_key(jti)).await;
    }

    // ── the pure discriminants ───────────────────────────────────────

    #[test]
    fn client_type_lenient_read_folds_unknowns_onto_admin() {
        assert!(matches!(ClientType::from_i32(0), ClientType::Admin));
        assert!(matches!(ClientType::from_i32(1), ClientType::App));
        for v in [-1, 2, 99] {
            assert!(matches!(ClientType::from_i32(v), ClientType::Admin), "{v}");
        }
    }

    #[test]
    fn client_type_strict_read_accepts_only_the_declared_values() {
        assert!(matches!(
            ClientType::from_i32_strict(0),
            Some(ClientType::Admin)
        ));
        assert!(matches!(
            ClientType::from_i32_strict(1),
            Some(ClientType::App)
        ));
        for v in [-1, 2, 99] {
            assert!(ClientType::from_i32_strict(v).is_none(), "{v}");
        }
    }

    #[test]
    fn refresh_tokens_are_256_bit_hex_and_unique() {
        let a = TokenStore::new_refresh_token();
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|b| b.is_ascii_hexdigit()), "{a}");
        assert_ne!(a, TokenStore::new_refresh_token());
    }

    // ── the live store faces ─────────────────────────────────────────

    #[tokio::test]
    async fn pair_registration_whitelist_and_rotation() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        let uid = test_uid();
        let jti = test_jti();
        let admin = TokenStore::new(redis.clone(), ClientType::Admin, 60, 120);
        let app = TokenStore::new(redis.clone(), ClientType::App, 60, 0);
        let mut conn = redis.clone();

        // ── the key families: the pair lands under the client prefix 0.
        admin
            .add_token_pair(uid, &jti, "at-admin", Some("rt-admin"))
            .await
            .unwrap();
        let stored: String = conn.get(admin.at_key(uid, &jti)).await.unwrap();
        assert_eq!(stored, "at-admin");
        let stored: String = conn.get(admin.rt_key(uid, &jti)).await.unwrap();
        assert_eq!(stored, "rt-admin");
        // Each row rides its store's expiry profile.
        for (key, secs) in [
            (admin.at_key(uid, &jti), 60),
            (admin.rt_key(uid, &jti), 120),
        ] {
            let ttl: i64 = conn.ttl(key).await.unwrap();
            assert!(
                (secs - 30..=secs).contains(&ttl),
                "ttl {ttl} for a {secs}s row"
            );
        }
        // The app family is a sibling keyspace: nothing answers there.
        assert!(!app.has_access_token(uid, &jti).await.unwrap());
        assert!(app
            .jti_of_access_token(uid, "at-admin")
            .await
            .unwrap()
            .is_none());

        // ── the whitelist: exact match on the stored value, keyed by
        // (uid, jti).
        assert!(admin.is_valid_access_token(uid, &jti, "at-admin").await);
        assert!(!admin.is_valid_access_token(uid, &jti, "AT-ADMIN").await);
        assert!(!admin.is_valid_access_token(uid, &jti, "at-admin ").await);
        assert!(!admin.is_valid_access_token(uid, &jti, "").await);
        let other = test_jti();
        assert!(!admin.is_valid_access_token(uid, &other, "at-admin").await);
        assert!(
            !admin
                .is_valid_access_token(test_uid(), &jti, "at-admin")
                .await
        );

        // ── the rotation: a wrong refresh leaves the whole pair intact.
        assert!(!admin
            .verify_and_revoke_pair(uid, &jti, "rt-wrong")
            .await
            .unwrap());
        assert!(admin.is_valid_access_token(uid, &jti, "at-admin").await);
        // The right one revokes both rows atomically.
        assert!(admin
            .verify_and_revoke_pair(uid, &jti, "rt-admin")
            .await
            .unwrap());
        assert!(!admin.is_valid_access_token(uid, &jti, "at-admin").await);
        assert!(!admin.has_access_token(uid, &jti).await.unwrap());
        // Replay: the refresh row is gone too — nothing rotates twice.
        assert!(!admin
            .verify_and_revoke_pair(uid, &jti, "rt-admin")
            .await
            .unwrap());
        // The sibling app family never had rows.
        assert!(app.access_tokens(uid).await.unwrap().is_empty());

        sweep(&admin, &app, uid, &jti).await;
    }

    #[tokio::test]
    async fn a_disabled_refresh_expiry_stores_no_refresh_row() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        let uid = test_uid();
        let jti = test_jti();
        // The app profile: refresh expiry 0s.
        let app = TokenStore::new(redis.clone(), ClientType::App, 60, 0);
        let admin = TokenStore::new(redis.clone(), ClientType::Admin, 60, 0);
        let mut conn = redis.clone();
        assert_eq!((app.access_secs(), app.refresh_secs()), (60, 0));

        app.add_token_pair(uid, &jti, "at-app", Some("rt-app"))
            .await
            .unwrap();
        assert!(app.has_access_token(uid, &jti).await.unwrap());
        let stored: Option<String> = conn.get(app.rt_key(uid, &jti)).await.unwrap();
        assert_eq!(stored, None);
        // With no refresh row the rotation refuses — and, verifying
        // first, leaves the access row alone.
        assert!(!app
            .verify_and_revoke_pair(uid, &jti, "rt-app")
            .await
            .unwrap());
        assert!(app.is_valid_access_token(uid, &jti, "at-app").await);
        // The by-jti face still revokes the session cleanly.
        app.revoke_by_jti(uid, &jti).await.unwrap();
        assert!(!app.has_access_token(uid, &jti).await.unwrap());

        // A zero access expiry floors at a 1s row — never a persistent
        // session key.
        let floor = TokenStore::new(redis.clone(), ClientType::Admin, 0, 0);
        let jti_floor = test_jti();
        floor
            .add_token_pair(uid, &jti_floor, "at-0", None)
            .await
            .unwrap();
        let ttl: i64 = conn.ttl(floor.at_key(uid, &jti_floor)).await.unwrap();
        assert_eq!(ttl, 1);

        sweep(&admin, &app, uid, &jti).await;
    }

    #[tokio::test]
    async fn blacklist_and_the_by_jti_revoke() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        let uid = test_uid();
        let jti = test_jti();
        let admin = TokenStore::new(redis.clone(), ClientType::Admin, 60, 120);
        let app = TokenStore::new(redis.clone(), ClientType::App, 60, 0);
        let mut conn = redis.clone();

        // ── the blacklist is keyed by the jti alone — one row blocks
        // every client family.
        assert!(!admin.is_blocked(&jti).await);
        assert!(!app.is_blocked(&jti).await);
        admin.block_token(&jti, "logout", Some(120)).await.unwrap();
        assert!(admin.is_blocked(&jti).await);
        assert!(app.is_blocked(&jti).await);
        let reason: String = conn.get(TokenStore::bl_key(&jti)).await.unwrap();
        assert_eq!(reason, "logout");
        let ttl: i64 = conn.ttl(TokenStore::bl_key(&jti)).await.unwrap();
        assert!((90..=120).contains(&ttl), "ttl {ttl}");
        // An absent ttl persists the row instead (Redis answers -1).
        admin.block_token(&jti, "banned", None).await.unwrap();
        let ttl: i64 = conn.ttl(TokenStore::bl_key(&jti)).await.unwrap();
        assert_eq!(ttl, -1);

        // ── the by-jti revoke removes the session rows (the unblock
        // face) ...
        admin
            .add_token_pair(uid, &jti, "at", Some("rt"))
            .await
            .unwrap();
        admin.revoke_by_jti(uid, &jti).await.unwrap();
        assert!(!admin.has_access_token(uid, &jti).await.unwrap());
        // ... refresh row included: nothing is left to rotate.
        assert!(!admin.verify_and_revoke_pair(uid, &jti, "rt").await.unwrap());
        // The blacklist row itself just ages out.
        assert!(admin.is_blocked(&jti).await);
        // Revoking an unknown jti is a no-op, not an error.
        admin.revoke_by_jti(uid, &test_jti()).await.unwrap();

        sweep(&admin, &app, uid, &jti).await;
    }

    #[tokio::test]
    async fn session_scans_walk_the_user_family_only() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        let uid = test_uid();
        let (a, b, c) = (test_jti(), test_jti(), test_jti());
        let admin = TokenStore::new(redis.clone(), ClientType::Admin, 60, 120);
        let app = TokenStore::new(redis.clone(), ClientType::App, 60, 0);
        admin
            .add_token_pair(uid, &a, "at-a", Some("rt-a"))
            .await
            .unwrap();
        admin.add_token_pair(uid, &b, "at-b", None).await.unwrap();
        // The sibling family, under the same user.
        app.add_token_pair(uid, &c, "at-c", None).await.unwrap();

        // ── the session list: every live access token of the user,
        // family-scoped.
        let mut listed = admin.access_tokens(uid).await.unwrap();
        listed.sort();
        assert_eq!(listed, ["at-a", "at-b"]);
        assert_eq!(app.access_tokens(uid).await.unwrap(), ["at-c"]);
        // A user without sessions reads empty.
        assert!(admin.access_tokens(test_uid()).await.unwrap().is_empty());

        // ── the by-token blacklist target: the jti whose stored value
        // equals the token.
        assert_eq!(
            admin.jti_of_access_token(uid, "at-b").await.unwrap(),
            Some(b.clone())
        );
        assert_eq!(
            admin.jti_of_access_token(uid, "at-a").await.unwrap(),
            Some(a.clone())
        );
        // Unknown values, and the sibling family's tokens, answer None.
        assert_eq!(admin.jti_of_access_token(uid, "at-c").await.unwrap(), None);
        assert_eq!(
            admin.jti_of_access_token(uid, "at-zzz").await.unwrap(),
            None
        );
        assert_eq!(
            app.jti_of_access_token(uid, "at-c").await.unwrap(),
            Some(c.clone())
        );

        // ── logout: the user's whole admin family goes, the app family
        // survives.
        admin.revoke_user_tokens(uid).await.unwrap();
        assert!(admin.access_tokens(uid).await.unwrap().is_empty());
        assert!(!admin.has_access_token(uid, &a).await.unwrap());
        assert!(!admin.verify_and_revoke_pair(uid, &a, "rt-a").await.unwrap());
        assert_eq!(app.access_tokens(uid).await.unwrap(), ["at-c"]);
        // Revoking again is a clean no-op.
        admin.revoke_user_tokens(uid).await.unwrap();

        sweep(&admin, &app, uid, &c).await;
    }
}
