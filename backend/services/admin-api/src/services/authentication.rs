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
}

#[async_trait::async_trait]
impl proto::gen_admin::services::AuthenticationServiceHandlers for AuthenticationService {
    async fn login(&self, ctx: Ctx, req: LoginRequest) -> Result<LoginResponse, StatusError> {
        use proto::proto::authentication::service::v1::GrantType;
        // The captcha gate — password grant only.
        if GrantType::try_from(req.grant_type) == Ok(GrantType::Password) {
            let ok = crate::captcha::verify(
                &self.state.redis,
                &ctx.headers
                    .get(HEADER_CAPTCHA_ID)
                    .cloned()
                    .unwrap_or_default(),
                &ctx.headers
                    .get(HEADER_CAPTCHA_VALUE)
                    .cloned()
                    .unwrap_or_default(),
            )
            .await;
            if !ok {
                return Err(status_error("BAD_REQUEST", "invalid or missing captcha"));
            }
        }

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
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let tid = claims
                        .as_ref()
                        .and_then(|c| c.get("tid"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
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
                    login_method: grant,
                },
            )
            .await;
        }
        let mut resp = login_result.map_err(map_status)?.into_inner();

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
}
