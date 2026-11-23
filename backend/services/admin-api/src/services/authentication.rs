//! AuthenticationService — the admin BFF face of the login chain. The
//! domain logic lives in the core service; this BFF layer carries the
//! client-facing concerns the reference's BFF carries:
//!
//! * GenerateCaptcha / VerifyCaptcha — the Redis-backed PNG challenge
//!   (BFF-owned, client-facing);
//! * Login — the captcha gate (X-Captcha-Id / X-Captcha-Value headers,
//!   password grant only) → the core Login RPC → refresh token
//!   stripped into the HttpOnly cookie pair;
//! * RefreshToken — the credential from the HttpOnly cookie, the
//!   binding keys (uid/jti) back-filled from the operator claims →
//!   the core RefreshToken RPC → the rotated cookie pair;
//! * Logout — the cookie clear + the core Logout RPC (full revocation).

use std::sync::Arc;

use pbjson_types::Empty;
use proto::proto::authentication::service::v1::{
    GenerateCaptchaResponse, LoginRequest, LoginResponse, VerifyCaptchaRequest,
    VerifyCaptchaResponse,
};

use crate::services::map_status;
use crate::state::{internal_error, operator_of, status_error, AppState, StatusError};
use crate::token::UserTokenPayload;

type Ctx = rushwind_http_binding::ctx::RequestContext;

/// The header pair the captcha gate reads.
const HEADER_CAPTCHA_ID: &str = "x-captcha-id";
const HEADER_CAPTCHA_VALUE: &str = "x-captcha-value";

/// The forwarded failure reasons that count as credential misses — the
/// rate limiter's increment trigger. Captcha errors (`CAPTCHA_INVALID` /
/// `CAPTCHA_REQUIRED`) are not brute-force evidence and infrastructure
/// failures must not lock anybody out.
const CREDENTIAL_FAIL_REASONS: [&str; 3] = ["INVALID_PASSWORD", "USER_NOT_FOUND", "INVALID_TENANT"];

/// The refresh-expiry fallback when the core answer carries none (the
/// reference's defaultRefreshTTL — 30 days).
const REFRESH_TTL_FALLBACK_SECS: i64 = 30 * 24 * 3600;

pub struct AuthenticationService {
    pub state: Arc<AppState>,
}

impl AuthenticationService {
    fn set_refresh_cookies(&self, ctx: &Ctx, refresh: &str, secure: bool, ttl_secs: i64) {
        let (rt, exp) = crate::token::refresh_cookie_values(refresh, secure, ttl_secs);
        ctx.add_reply_header("Set-Cookie", &rt);
        ctx.add_reply_header("Set-Cookie", &exp);
    }

    fn clear_refresh_cookies(&self, ctx: &Ctx, secure: bool) {
        let (rt, exp) = crate::token::clear_cookie_values(secure);
        ctx.add_reply_header("Set-Cookie", &rt);
        ctx.add_reply_header("Set-Cookie", &exp);
    }

    fn cookie_secure(ctx: &Ctx) -> bool {
        ctx.headers
            .get("x-forwarded-proto")
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or(false)
    }

    /// The core's refresh-expiry stamp for the cookie mirror, with the
    /// reference's 30-day fallback when the answer carries none.
    fn refresh_ttl(resp: &LoginResponse) -> i64 {
        match resp.refresh_expires_in {
            Some(v) if v > 0 => v,
            _ => REFRESH_TTL_FALLBACK_SECS,
        }
    }

    /// The username dimension of the failure counters — the oneof's
    /// username case only (an email/mobile identifier counts on the IP
    /// dimension alone, matching the reference's `GetUsername`).
    fn username_of(req: &LoginRequest) -> String {
        match req.identifier.as_ref() {
            Some(
                proto::proto::authentication::service::v1::login_request::Identifier::Username(u),
            ) => u.clone(),
            _ => String::new(),
        }
    }

    /// The progressive captcha gate: the challenge becomes mandatory
    /// once the ip+username failure count reaches
    /// [`crate::login_rate_limiter::CAPTCHA_REQUIRED_THRESHOLD`]. Below
    /// it nothing is verified or consumed; at it a missing pair answers
    /// `CAPTCHA_REQUIRED` and a failed verify `CAPTCHA_INVALID` — the
    /// failing challenge is always destroyed (verify-and-delete) so a
    /// mismatched captcha cannot be retried within its TTL.
    async fn verify_login_captcha(
        &self,
        client_ip: &str,
        username: &str,
        ctx: &Ctx,
    ) -> Result<(), StatusError> {
        let attempts =
            crate::login_rate_limiter::failure_attempts(&self.state.redis, client_ip, username)
                .await;
        if attempts < crate::login_rate_limiter::CAPTCHA_REQUIRED_THRESHOLD {
            return Ok(());
        }
        let captcha_id = ctx
            .headers
            .get(HEADER_CAPTCHA_ID)
            .cloned()
            .unwrap_or_default();
        let captcha_value = ctx
            .headers
            .get(HEADER_CAPTCHA_VALUE)
            .cloned()
            .unwrap_or_default();
        if captcha_id.is_empty() || captcha_value.is_empty() {
            return Err(status_error("CAPTCHA_REQUIRED", "captcha required"));
        }
        if !crate::captcha::verify(&self.state.redis, &captcha_id, &captcha_value).await {
            // A mismatch consumed nothing (verify-and-delete only fires on
            // match) — destroy the challenge explicitly so the failed
            // captcha cannot be retried inside its TTL.
            crate::captcha::destroy(&self.state.redis, &captcha_id).await;
            return Err(status_error("CAPTCHA_INVALID", "invalid captcha"));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl proto::gen_admin::services::AuthenticationServiceHandlers for AuthenticationService {
    async fn login(&self, ctx: Ctx, req: LoginRequest) -> Result<LoginResponse, StatusError> {
        use proto::proto::authentication::service::v1::GrantType;

        // Pin the client type (admin) and forward to core. An in-flight
        // refresh answered through the login route binds to the
        // operator's session keys.
        let grant = req.grant_type;
        let mut req = req;
        req.client_type = Some(0);
        if GrantType::try_from(req.grant_type) == Ok(GrantType::RefreshToken) {
            let op = operator_of(&ctx)?;
            req.jti = Some(op.jti);
            req.user_id = Some(op.user_id);
        }

        // Session metadata: the BFF injects the HTTP-layer facts core
        // cannot see (the online-session list's ip/ua read off these).
        if req.client_ip.as_deref().unwrap_or("").is_empty() && !ctx.ip.is_empty() {
            req.client_ip = Some(ctx.ip.clone());
        }
        if req.user_agent.as_deref().unwrap_or("").is_empty() && !ctx.user_agent.is_empty() {
            req.user_agent = Some(ctx.user_agent.clone());
        }

        let login_username = req
            .identifier
            .as_ref()
            .map(|i| match i {
                proto::proto::authentication::service::v1::login_request::Identifier::Username(
                    u,
                ) => u.clone(),
                proto::proto::authentication::service::v1::login_request::Identifier::Email(e) => {
                    e.clone()
                }
                proto::proto::authentication::service::v1::login_request::Identifier::Mobile(m) => {
                    m.clone()
                }
            })
            .unwrap_or_default();

        if GrantType::try_from(req.grant_type) == Ok(GrantType::Password) {
            let client_ip = ctx.ip.as_str();
            // ── Gate 1: the brute-force lockout (ip + username, 5 failures
            //    → 15 min). Counters increment on the credential-failure
            //    branch below, reset on success.
            if crate::login_rate_limiter::is_locked(&self.state.redis, client_ip, &login_username)
                .await
            {
                return Err(status_error(
                    "BAD_REQUEST",
                    "too many login failures, please try again later",
                ));
            }
            // ── Gate 2: the progressive captcha — required only once the
            //    failure count reaches the threshold.
            self.verify_login_captcha(client_ip, &Self::username_of(&req), &ctx)
                .await?;
        }

        let audit_ip = ctx
            .headers
            .get("x-forwarded-for")
            .cloned()
            .or_else(|| ctx.headers.get("x-real-ip").cloned())
            .unwrap_or_default();
        let audit_rid = ctx.headers.get("x-request-id").cloned().unwrap_or_default();
        let audit_state = Arc::clone(&self.state);
        let mut core = self.state.core.clone();
        let login_result = core.login(tonic::Request::new(req)).await;
        // The login audit — verdict + reason, success and failure alike.
        {
            let (ok, reason, uid, tid) = match &login_result {
                Ok(resp) => {
                    use base64::Engine as _;
                    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD
                        .decode(
                            resp.get_ref()
                                .access_token
                                .split('.')
                                .nth(1)
                                .unwrap_or_default(),
                        )
                        .ok()
                        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
                    let uid = claims
                        .as_ref()
                        .and_then(|c| c.get("uid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let tid = claims
                        .as_ref()
                        .and_then(|c| c.get("tid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (true, String::new(), uid, tid)
                }
                Err(e) => (false, e.message().to_string(), 0, 0),
            };
            crate::audit::write_login_audit(
                &audit_state,
                crate::audit::LoginAudit {
                    username: &login_username,
                    user_id: uid,
                    tenant_id: tid,
                    success: ok,
                    failure_reason: &reason,
                    ip: &audit_ip,
                    request_id: &audit_rid,
                },
            )
            .await;
        }
        let mut resp = match login_result {
            Ok(r) => r,
            Err(e) => {
                // A credential-family miss records against both failure
                // counters (captcha errors and infrastructure failures
                // do not count). The reason rides the status details.
                let mapped = map_status(e);
                if GrantType::try_from(grant) == Ok(GrantType::Password)
                    && CREDENTIAL_FAIL_REASONS.contains(&mapped.reason)
                {
                    crate::login_rate_limiter::check_and_incr(
                        &self.state.redis,
                        &ctx.ip,
                        &login_username,
                    )
                    .await;
                }
                return Err(mapped);
            }
        }
        .into_inner();

        // A successful password login clears the counters (back to the
        // captcha-free state).
        if GrantType::try_from(grant) == Ok(GrantType::Password) {
            crate::login_rate_limiter::reset(&self.state.redis, &ctx.ip, &login_username).await;
        }

        // The refresh token rides the HttpOnly cookie pair instead of
        // the body, with the core's own expiry stamp.
        let secure = Self::cookie_secure(&ctx);
        if let Some(rt) = resp.refresh_token.take() {
            if !rt.is_empty() {
                let ttl = Self::refresh_ttl(&resp);
                self.set_refresh_cookies(&ctx, &rt, secure, ttl);
            }
        }
        Ok(resp)
    }

    async fn logout(&self, ctx: Ctx, _req: Empty) -> Result<Empty, StatusError> {
        let payload = operator_of(&ctx)?;
        self.clear_refresh_cookies(&ctx, Self::cookie_secure(&ctx));
        let mut core = self.state.core.clone();
        core.logout(tonic::Request::new(
            proto::proto::authentication::service::v1::LogoutRequest {
                client_type: 0,
                user_id: payload.user_id,
            },
        ))
        .await
        .map_err(map_status)?;
        Ok(Empty {})
    }

    async fn refresh_token(
        &self,
        ctx: Ctx,
        req: LoginRequest,
    ) -> Result<LoginResponse, StatusError> {
        // The credential: body first, else the HttpOnly cookie.
        let mut req = req;
        if req.refresh_token.as_deref().unwrap_or("").is_empty() {
            if let Some(cv) = ctx.cookies.get("refresh_token") {
                req.refresh_token = Some(cv.clone());
            }
        }
        // The binding keys, priority high→low: the request body (the
        // front-end decodes them off its persisted access token —
        // non-secret; the credential is the refresh token value), the
        // operator claims (an in-flight refresh while the access token
        // still verifies), and the unverified payload of the expired
        // bearer JWT (same source, tolerating the expired case).
        if req.refresh_token.as_deref().unwrap_or("").is_empty() {
            if let Some(cv) = ctx.cookies.get("refresh_token") {
                req.refresh_token = Some(cv.clone());
            }
        }
        let operator = ctx.claims.as_ref().and_then(UserTokenPayload::from_claims);
        if req.user_id.is_none() {
            req.user_id = operator.as_ref().map(|o| o.user_id);
        }
        if req.jti.as_deref().unwrap_or("").is_empty() {
            req.jti = operator.as_ref().map(|o| o.jti.clone());
        }
        if req.user_id.unwrap_or(0) == 0 || req.jti.as_deref().unwrap_or("").is_empty() {
            let jwt = ctx
                .headers
                .get("authorization")
                .and_then(|v| {
                    v.strip_prefix("Bearer ")
                        .or_else(|| v.strip_prefix("bearer "))
                })
                .and_then(auth::parse_unverified_bearer_jwt);
            if let Some((uid, jti)) = jwt {
                if req.user_id.unwrap_or(0) == 0 {
                    req.user_id = Some(uid);
                }
                if req.jti.as_deref().unwrap_or("").is_empty() {
                    req.jti = Some(jti);
                }
            }
        }
        req.client_type = Some(0);

        let mut core = self.state.core.clone();
        let mut resp = core
            .refresh_token(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner();

        let secure = Self::cookie_secure(&ctx);
        if let Some(rt) = resp.refresh_token.take() {
            if !rt.is_empty() {
                let ttl = Self::refresh_ttl(&resp);
                self.set_refresh_cookies(&ctx, &rt, secure, ttl);
            }
        }
        Ok(resp)
    }

    async fn generate_captcha(
        &self,
        _ctx: Ctx,
        _req: Empty,
    ) -> Result<GenerateCaptchaResponse, StatusError> {
        let (id, image, _answer) = crate::captcha::generate(&self.state.redis)
            .await
            .map_err(internal_error)?;
        Ok(GenerateCaptchaResponse {
            captcha_id: id,
            image_base64: image,
        })
    }

    async fn verify_captcha(
        &self,
        _ctx: Ctx,
        req: VerifyCaptchaRequest,
    ) -> Result<VerifyCaptchaResponse, StatusError> {
        let ok = crate::captcha::verify(&self.state.redis, &req.captcha_id, &req.user_input).await;
        Ok(VerifyCaptchaResponse { valid: ok })
    }

    // ── 2026-11 契约批次新增——纯转发面（无 BFF 侧关注点：验证码/cookie
    //    都不沾边，登录态/token 语义全部在 core）。
    async fn get_oidc_authorize_url(
        &self,
        _ctx: Ctx,
        req: proto::proto::authentication::service::v1::GetOidcAuthorizeUrlRequest,
    ) -> Result<proto::proto::authentication::service::v1::GetOidcAuthorizeUrlResponse, StatusError>
    {
        let mut core = self.state.core.clone();
        core.get_oidc_authorize_url(tonic::Request::new(req))
            .await
            .map_err(map_status)
            .map(|r| r.into_inner())
    }

    async fn forgot_password(
        &self,
        _ctx: Ctx,
        req: proto::proto::authentication::service::v1::ForgotPasswordRequest,
    ) -> Result<Empty, StatusError> {
        let mut core = self.state.core.clone();
        core.forgot_password(tonic::Request::new(req))
            .await
            .map_err(map_status)
            .map(|r| r.into_inner())
    }

    async fn reset_password_by_code(
        &self,
        _ctx: Ctx,
        req: proto::proto::authentication::service::v1::ResetPasswordByCodeRequest,
    ) -> Result<Empty, StatusError> {
        let mut core = self.state.core.clone();
        core.reset_password_by_code(tonic::Request::new(req))
            .await
            .map_err(map_status)
            .map(|r| r.into_inner())
    }

    async fn get_tenant_branding(
        &self,
        _ctx: Ctx,
        req: proto::proto::authentication::service::v1::GetTenantBrandingRequest,
    ) -> Result<proto::proto::authentication::service::v1::GetTenantBrandingResponse, StatusError>
    {
        let mut core = self.state.core.clone();
        core.get_tenant_branding(tonic::Request::new(req))
            .await
            .map_err(map_status)
            .map(|r| r.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(headers: &[(&'static str, &'static str)]) -> Ctx {
        let mut ctx = Ctx::default();
        for (name, value) in headers {
            ctx.headers.insert((*name).into(), (*value).into());
        }
        ctx
    }

    #[test]
    fn cookie_secure_follows_the_forwarded_proto() {
        let secure = AuthenticationService::cookie_secure;
        assert!(secure(&ctx_with(&[("x-forwarded-proto", "https")])));
        // The scheme is matched case-insensitively.
        assert!(secure(&ctx_with(&[("x-forwarded-proto", "HTTPS")])));
        // Plain http, a proxy list, or no header at all stays unsecured.
        assert!(!secure(&ctx_with(&[("x-forwarded-proto", "http")])));
        assert!(!secure(&ctx_with(&[("x-forwarded-proto", "https, http")])));
        assert!(!secure(&Ctx::default()));
    }

    #[test]
    fn refresh_ttl_reads_the_core_stamp_with_the_30_day_fallback() {
        let resp = |refresh_expires_in: Option<i64>| LoginResponse {
            refresh_expires_in,
            ..Default::default()
        };
        assert_eq!(
            AuthenticationService::refresh_ttl(&resp(Some(43_200))),
            43_200
        );
        // Absent, zero, or negative: the reference's defaultRefreshTTL.
        for v in [None, Some(0), Some(-5)] {
            assert_eq!(
                AuthenticationService::refresh_ttl(&resp(v)),
                REFRESH_TTL_FALLBACK_SECS,
                "{v:?}"
            );
        }
        // The fallback itself: the reference's 30 days.
        let fallback = REFRESH_TTL_FALLBACK_SECS;
        assert_eq!(fallback, 30 * 24 * 3600);
    }

    #[test]
    fn failure_counter_username_reads_the_username_case_only() {
        use proto::proto::authentication::service::v1::login_request::Identifier;
        let req = |identifier: Option<Identifier>| LoginRequest {
            identifier,
            ..Default::default()
        };
        let username_of = AuthenticationService::username_of;
        assert_eq!(
            username_of(&req(Some(Identifier::Username("alice".into())))),
            "alice"
        );
        // An email or mobile identifier counts on the IP dimension alone.
        assert_eq!(
            username_of(&req(Some(Identifier::Email("a@b.c".into())))),
            ""
        );
        assert_eq!(
            username_of(&req(Some(Identifier::Mobile("13800000000".into())))),
            ""
        );
        assert_eq!(username_of(&req(None)), "");
    }

    #[test]
    fn credential_failure_reasons_anchor_onto_the_error_tables() {
        // The limiter trigger compares the mapped reason — a table literal
        // — so every counted reason must resolve against the tables.
        let anchored = |reason: &str| {
            proto::gen_admin::error_tables::ERROR_STATUS
                .iter()
                .chain(proto::gen_app::error_tables::ERROR_STATUS.iter())
                .any(|(_, r, _)| *r == reason)
        };
        for reason in CREDENTIAL_FAIL_REASONS {
            assert!(anchored(reason), "{reason} must be an error-table reason");
        }
        // The progressive-captcha pair and the lockout are not brute-force
        // evidence — they resolve too, but stay out of the trigger.
        assert_eq!(CREDENTIAL_FAIL_REASONS.len(), 3);
        for reason in ["CAPTCHA_INVALID", "CAPTCHA_REQUIRED", "BAD_REQUEST"] {
            assert!(anchored(reason), "{reason}");
            assert!(!CREDENTIAL_FAIL_REASONS.contains(&reason), "{reason}");
        }
    }
}
