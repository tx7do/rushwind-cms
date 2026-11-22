//! The login failure counter — the brute-force gate behind the
//! progressive captcha. Redis-backed, IP + username dual-dimension;
//! every authz failure (`INVALID_PASSWORD` / `USER_NOT_FOUND` /
//! `INVALID_TENANT`) increments both counters, a success resets them,
//! and either counter reaching [`LOCK_FAIL_THRESHOLD`] locks the
//! pair for [`LOCKOUT_SECS`]. The same keyspace serves the app BFF's
//! limiter (shared namespace, disjoint BFF faces — the reference keeps
//! one counter per deployment over the same key format), so a brute
//! force through either face locks both.
//!
//! Every read/write is fail-open: the limiter is a defensive
//! enhancement, a Redis outage must not block legitimate logins.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// The failure count that trips the lockout (5 failures → locked).
pub const LOCK_FAIL_THRESHOLD: i64 = 5;

/// The lockout window, also the counter TTL — past the window with no
/// further failures the count expires and the pair unlocks.
pub const LOCKOUT_SECS: u64 = 15 * 60;

/// The progressive-captcha trigger: from the first recorded failure on,
/// the login form must carry a captcha. Sits between 0 and the lockout
/// threshold so a locked pair stays answerable only with a captcha.
pub const CAPTCHA_REQUIRED_THRESHOLD: i64 = 1;

fn fail_keys(ip: &str, username: &str) -> Vec<String> {
    let mut keys = Vec::with_capacity(2);
    if !ip.is_empty() {
        keys.push(format!("cms:login:fail:ip:{ip}"));
    }
    if !username.is_empty() {
        keys.push(format!("cms:login:fail:user:{username}"));
    }
    keys
}

/// The atomic "locked? → incr → first-write sets TTL" step, one key at a
/// time: at or above the threshold the counter freezes (still locked, no
/// growth); below it the failure records. Returns `(locked, count)`.
const INCR_IF_NOT_LOCKED: &str = r#"
local key = KEYS[1]
local threshold = tonumber(ARGV[1])
local ttl = tonumber(ARGV[2])

local current = tonumber(redis.call('GET', key) or "0")
if current >= threshold then
    return {1, current}
end

current = redis.call('INCR', key)
if current == 1 then
    redis.call('EXPIRE', key, ttl)
end
return {0, current}
"#;

/// Records one credential failure and answers whether the pair is now
/// locked. Fail-open: a Redis error leaves the login path unblocked.
pub async fn check_and_incr(redis: &ConnectionManager, ip: &str, username: &str) -> bool {
    let mut conn = redis.clone();
    for key in fail_keys(ip, username) {
        let script = redis::Script::new(INCR_IF_NOT_LOCKED);
        let res: Result<(i64, i64), _> = script
            .key(&key)
            .arg(LOCK_FAIL_THRESHOLD)
            .arg(LOCKOUT_SECS)
            .invoke_async(&mut conn)
            .await;
        if let Ok((locked, _)) = res {
            if locked == 1 {
                return true;
            }
        }
    }
    false
}

/// The pre-login probe: is the pair locked right now (no increments)?
pub async fn is_locked(redis: &ConnectionManager, ip: &str, username: &str) -> bool {
    let mut conn = redis.clone();
    for key in fail_keys(ip, username) {
        let cnt: Option<i64> = conn.get(&key).await.unwrap_or(None);
        if cnt.unwrap_or(0) >= LOCK_FAIL_THRESHOLD {
            return true;
        }
    }
    false
}

/// The read-only failure snapshot (the max over both dimensions) — the
/// progressive-captcha gate reads it to decide whether the login must
/// carry a captcha. Redis unavailable reads as 0 (fail-open).
pub async fn failure_attempts(redis: &ConnectionManager, ip: &str, username: &str) -> i64 {
    let mut conn = redis.clone();
    let mut max = 0;
    for key in fail_keys(ip, username) {
        let cnt: Option<i64> = conn.get(&key).await.unwrap_or(None);
        max = max.max(cnt.unwrap_or(0));
    }
    max
}

/// Clears both counters on a successful login (back to captcha-free).
pub async fn reset(redis: &ConnectionManager, ip: &str, username: &str) {
    let mut conn = redis.clone();
    for key in fail_keys(ip, username) {
        let _: Result<i64, _> = conn.del(&key).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live-Redis pin for the counter arc (dual-dimension keys, the
    /// lockout threshold, the reset). Skips unless `RUSHWIND_TEST_REDIS_URL`
    /// carries a DSN — CI runs without one and passes vacuously.
    async fn redis_of() -> Option<ConnectionManager> {
        let url = std::env::var("RUSHWIND_TEST_REDIS_URL").ok()?;
        let client = redis::Client::open(url).ok()?;
        client.get_connection_manager().await.ok()
    }

    #[tokio::test]
    async fn counter_locks_at_threshold_and_resets() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        // A unique username dimension per run: the ip dimension stays
        // empty (skipped) so repeated runs never share counters.
        let user = format!("rl_test_{}", uuid::Uuid::now_v7().simple());
        assert!(!is_locked(&redis, "", &user).await);
        assert!(
            failure_attempts(&redis, "", &user).await < CAPTCHA_REQUIRED_THRESHOLD,
            "no failures yet: the progressive gate must stay off"
        );
        // Four failures: counted, not locked.
        for _ in 0..(LOCK_FAIL_THRESHOLD - 1) {
            assert!(!check_and_incr(&redis, "", &user).await);
        }
        assert!(
            !is_locked(&redis, "", &user).await,
            "below threshold: unlocked"
        );
        // The fifth failure trips the lock (the next pre-check sees it).
        let _ = check_and_incr(&redis, "", &user).await;
        assert!(is_locked(&redis, "", &user).await);
        assert_eq!(
            failure_attempts(&redis, "", &user).await,
            LOCK_FAIL_THRESHOLD
        );
        // Success resets both dims — the login face calls this after a
        // successful password grant.
        reset(&redis, "", &user).await;
        assert!(!is_locked(&redis, "", &user).await);
        assert_eq!(failure_attempts(&redis, "", &user).await, 0);
    }

    #[tokio::test]
    async fn empty_dimensions_skip_counters() {
        // No ip, no username → no keys → nothing locks (the reference
        // skips empty dimensions the same way).
        let redis = match redis_of().await {
            Some(r) => r,
            None => {
                eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
                return;
            }
        };
        assert!(!check_and_incr(&redis, "", "").await);
        assert!(!is_locked(&redis, "", "").await);
        assert_eq!(failure_attempts(&redis, "", "").await, 0);
    }
}
