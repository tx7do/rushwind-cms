//! The BFF-side cookie plumbing (the refresh-token cookie pair) and
//! the payload re-export. The Redis token families live in the core
//! service (`core-service/src/token.rs`); the BFF only shapes cookies.

pub use store::auth::UserTokenPayload;

/// The refresh-token cookie's narrowed path (the app BFF's refresh
/// endpoint).
pub const REFRESH_COOKIE_PATH: &str = "/app/v1/refresh-token";

/// The cookie-pair builder: the HttpOnly cookie path-narrowed to the
/// refresh endpoint, and the non-HttpOnly expiry stamp the front-end
/// timer reads (the reference's setRefreshCookies).
pub fn refresh_cookie_values(
    refresh_token: &str,
    secure: bool,
    refresh_ttl_secs: i64,
) -> (String, String) {
    let max_age = refresh_ttl_secs.max(1);
    let expires = chrono::Utc::now().timestamp() + refresh_ttl_secs;
    let flags = |http_only: bool, path: &str| {
        format!(
            "Path={path}; Max-Age={max_age}; SameSite=Lax{}{}",
            if secure { "; Secure" } else { "" },
            if http_only { "; HttpOnly" } else { "" }
        )
    };
    (
        format!(
            "refresh_token={refresh_token}; {}",
            flags(true, REFRESH_COOKIE_PATH)
        ),
        format!("refresh_exp={expires}; {}", flags(false, "/")),
    )
}

/// The Max-Age=0 clearing pair — each cookie cleared on its own
/// original Path.
pub fn clear_cookie_values(secure: bool) -> (String, String) {
    let flags = |http_only: bool, path: &str| {
        format!(
            "Path={path}; Max-Age=0; SameSite=Lax{}{}",
            if secure { "; Secure" } else { "" },
            if http_only { "; HttpOnly" } else { "" }
        )
    };
    (
        format!("refresh_token=; {}", flags(true, REFRESH_COOKIE_PATH)),
        format!("refresh_exp=; {}", flags(false, "/")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_values_carry_the_reference_attributes() {
        let (rt, exp) = refresh_cookie_values("tok", false, 43200);
        assert!(rt.starts_with("refresh_token=tok; Path=/app/v1/refresh-token; Max-Age=43200"));
        assert!(rt.contains("HttpOnly"));
        assert!(rt.contains("SameSite=Lax"));
        assert!(!rt.contains("Secure"));
        assert!(exp.starts_with("refresh_exp="));
        assert!(exp.contains("Path=/;"));
        assert!(!exp.contains("HttpOnly"));

        let (rt, _) = refresh_cookie_values("tok", true, 43200);
        assert!(rt.contains("Secure"));

        let (rt, exp) = clear_cookie_values(false);
        assert!(rt.starts_with("refresh_token=; Path=/app/v1/refresh-token; Max-Age=0"));
        assert!(exp.starts_with("refresh_exp=; Path=/; Max-Age=0"));
    }
}
