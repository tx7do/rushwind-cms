//! The auth gate for the protected subtrees of the two CMS deployments
//! (the admin BFF and the app BFF) — the deployment face. The session
//! stage (the checker contract, the authenticate-then-check head, the
//! fixed failure texts) lives in `rushwind-authn-gate`; this file is
//! the two-stage gate body that anchors the failure envelopes to each
//! owning BFF's error tables (`admin.service.v1` / `app.service.v1`)
//! and injects the claim bag into the request extensions — the
//! request-context injection consumed by the glue into the
//! per-request [`rushwind_http_binding::ctx::RequestContext`]. The
//! reference's later stages (the tenant gate and the authorization
//! evaluator) land with the storage phase.

use std::sync::Arc;

use rushwind_authn::Authenticator;
use rushwind_authn_gate::SessionError;

/// The claim bag the gate injects into the request extensions on
/// success — re-exported for the hand-mounted faces, which read it off
/// the extensions the way the binding glue does.
pub use rushwind_authn::AuthClaims;

/// The server-side session checks (Redis whitelist/blacklist) — the
/// framework contract, re-exported at the deployment's spelling.
pub use rushwind_authn_gate::AccessTokenChecker;

/// The refresh fallback's claim sniff — the framework utility,
/// re-exported at the deployment's spelling.
pub use rushwind_authn_gate::parse_unverified_bearer_jwt;

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
    let bearer = rushwind_http_binding::ctx::bearer_token(req.headers());
    let claims = match rushwind_authn_gate::authenticate_and_check_session(
        auth.as_ref(),
        checker.as_ref(),
        &headers,
        bearer.as_deref(),
    )
    .await
    {
        Ok(claims) => claims,
        Err(err) => {
            return rushwind_http_binding::envelope::error_response(unauthorized(package, err));
        }
    };
    req.extensions_mut().insert(claims);
    next.run(req).await
}

/// The middleware failure: the status the owning BFF's error tables anchor
/// to `UNAUTHORIZED` (401), and the stage failure's fixed message text.
fn unauthorized(package: &str, err: SessionError) -> rushwind_http_binding::envelope::StatusError {
    let status = proto::tables::error_status(package, "UNAUTHORIZED").unwrap_or(401);
    rushwind_http_binding::envelope::StatusError::new(status, "UNAUTHORIZED", err.message())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_resolves_per_package_status() {
        let e = unauthorized("admin.service.v1", SessionError::MissingBearer);
        assert_eq!(e.status, 401);
        assert_eq!(e.reason, "UNAUTHORIZED");
        assert_eq!(e.message, "missing bearer token");
        let e = unauthorized("app.service.v1", SessionError::InvalidOrExpired);
        assert_eq!(e.message, "access token expired");
    }
}
