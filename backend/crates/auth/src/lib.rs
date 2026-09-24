//! The auth gate for the protected subtrees of the two CMS deployments
//! (the admin BFF and the app BFF).
//!
//! Two-stage validation, in wire order:
//! `Authenticator.Authenticate(ACCESS)` → `AccessTokenChecker`
//! (Redis whitelist `at:{uid}:{jti}` exact compare + blacklist
//! `bl:{jti}`). The reference's later stages (the tenant gate and the
//! authorization evaluator) land with the storage phase; until then the
//! gate is the protected subtree's only defense. Failures render the
//! four-field status envelope: reason `UNAUTHORIZED` for every credential
//! failure (message pinned to `missing bearer token` /
//! `access token expired`).
//!
//! Success inserts the claim bag into the request extensions — the
//! request-context injection — consumed by the glue into the per-request
//! [`rushwind_http_binding::ctx::RequestContext`].
//!
//! The error tables anchor per deployment package (`admin.service.v1` /
//! `app.service.v1`), so the envelope helpers take the owning package as
//! a parameter.

use std::sync::Arc;

use async_trait::async_trait;
use rushwind_authn::{Authenticator, AuthnError};

/// The claim bag the gate injects into the request extensions on
/// success — re-exported for the hand-mounted faces, which read it off
/// the extensions the way the binding glue does.
pub use rushwind_authn::AuthClaims;

/// The server-side session checks (Redis whitelist/blacklist), split out
/// of the engine so the storage stays service-side.
#[async_trait]
pub trait AccessTokenChecker: Send + Sync {
    /// `at:{uid}:{jti}` exact-match — false means revoked/expired.
    async fn is_valid_access_token(&self, uid: u32, jti: &str, token: &str) -> bool;
    /// `bl:{jti}` existence.
    async fn is_blocked_access_token(&self, jti: &str) -> bool;
}

/// The gate middleware body: verify the credential, run the session
/// stage, pass through with the claims injected, or render the envelope.
/// `package` is the owning BFF's proto package (its error tables anchor
/// the UNAUTHORIZED reason).
pub async fn auth_gate(
    auth: Arc<dyn Authenticator>,
    checker: Arc<dyn AccessTokenChecker + 'static>,
    package: &'static str,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    match auth.authenticate(&headers) {
        Ok(claims) => {
            // The session stage: whitelist + blacklist.
            let uid = claims
                .0
                .get("uid")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0);
            let jti = claims.get_jwt_id().unwrap_or_default();
            let token = rushwind_http_binding::ctx::bearer_token(req.headers()).unwrap_or_default();
            let valid = if jti.is_empty() {
                false
            } else {
                checker.is_valid_access_token(uid, &jti, &token).await
                    && !checker.is_blocked_access_token(&jti).await
            };
            if !valid {
                return rushwind_http_binding::envelope::error_response(unauthorized(
                    package,
                    AuthnError::TokenExpired,
                ));
            }

            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(err) => rushwind_http_binding::envelope::error_response(unauthorized(package, err)),
    }
}

/// The middleware failure: the status the owning BFF's error tables anchor
/// to `UNAUTHORIZED` (401), and the branch's fixed message text.
fn unauthorized(package: &str, err: AuthnError) -> rushwind_http_binding::envelope::StatusError {
    let message = match err {
        AuthnError::MissingBearerToken => "missing bearer token",
        _ => "access token expired",
    };
    let status = proto::tables::error_status(package, "UNAUTHORIZED").unwrap_or(401);
    rushwind_http_binding::envelope::StatusError::new(status, "UNAUTHORIZED", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_resolves_per_package_status() {
        let e = unauthorized("admin.service.v1", AuthnError::MissingBearerToken);
        assert_eq!(e.status, 401);
        assert_eq!(e.reason, "UNAUTHORIZED");
        assert_eq!(e.message, "missing bearer token");
        let e = unauthorized("app.service.v1", AuthnError::TokenExpired);
        assert_eq!(e.message, "access token expired");
    }
}
