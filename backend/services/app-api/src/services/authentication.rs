//! AuthenticationService — the app (C-side) BFF face of the login
//! chain. The domain logic lives in the core service; this BFF layer
//! carries the client-facing concerns: the app client-type pin (1), the
//! brute-force lockout + the progressive captcha gate (captcha only for
//! clients that declare `X-Captcha-Supported: 1` — the lockout spares
//! nobody), the `GenerateCaptcha` challenge endpoint, the refresh
//! binding's uid/jti fallback off the (possibly expired) bearer JWT,
//! and the body-borne refresh token (the reference's app BFF sets no
//! cookies — the refresh token stays in the JSON body).

use std::sync::Arc;

use pbjson_types::Empty;
use proto::proto::authentication::service::v1::{
    GenerateCaptchaResponse, LoginRequest, LoginResponse,
};

use crate::services::map_status;
use crate::state::{internal_error, operator_of, status_error, AppState, StatusError};

type Ctx = rushwind_http_binding::ctx::RequestContext;

/// The header pair the captcha gate reads, plus the client's own
/// declaration that it renders challenges at all.
const HEADER_CAPTCHA_ID: &str = "x-captcha-id";
const HEADER_CAPTCHA_VALUE: &str = "x-captcha-value";
const HEADER_CAPTCHA_SUPPORTED: &str = "x-captcha-supported";

/// The forwarded failure reasons that count as credential misses — the
/// rate limiter's increment trigger. Captcha errors (`CAPTCHA_INVALID` /
/// `CAPTCHA_REQUIRED`) are not brute-force evidence and infrastructure
/// failures must not lock anybody out.
const CREDENTIAL_FAIL_REASONS: [&str; 3] = ["INVALID_PASSWORD", "USER_NOT_FOUND", "INVALID_TENANT"];

pub struct AuthenticationService {
    pub state: Arc<AppState>,
}

impl AuthenticationService {
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

    /// The client's own declaration that it renders challenges
    /// (`X-Captcha-Supported: 1`); an absent declaration (the
    /// unadapted ends) reads as unsupported — the progressive gate
    /// would otherwise hard-block a one-failure login behind
    /// `CAPTCHA_REQUIRED` for the whole counter window.
    fn captcha_supported(&self, ctx: &Ctx) -> bool {
        ctx.headers
            .get(HEADER_CAPTCHA_SUPPORTED)
            .map(|v| v.trim() == "1")
            .unwrap_or(false)
    }

    /// The progressive captcha gate — identical to the admin face's,
    /// but only reached by clients that declared captcha support.
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

/// The bearer credential off the request headers (the refresh
/// binding's fallback key source when no verified operator rides).
fn bearer_of(ctx: &Ctx) -> Option<String> {
    let auth = ctx.headers.get("authorization")?;
    let token = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))?;
    (!token.is_empty()).then(|| token.to_string())
}

#[async_trait::async_trait]
impl proto::gen_app::services::AuthenticationServiceHandlers for AuthenticationService {
    async fn login(&self, ctx: Ctx, mut req: LoginRequest) -> Result<LoginResponse, StatusError> {
        use proto::proto::authentication::service::v1::GrantType;
        // Pin the client type (app) and forward to core. The refresh
        // token stays in the response body (the reference's app BFF
        // sets no cookies).
        req.client_type = Some(1);

        // Session metadata: the BFF injects the HTTP-layer facts core
        // cannot see (the online-session list's ip/ua read off these).
        if req.client_ip.as_deref().unwrap_or("").is_empty() && !ctx.ip.is_empty() {
            req.client_ip = Some(ctx.ip.clone());
        }
        if req.user_agent.as_deref().unwrap_or("").is_empty() && !ctx.user_agent.is_empty() {
            req.user_agent = Some(ctx.user_agent.clone());
        }

        let login_username = Self::username_of(&req);
        let grant_type = req.grant_type;

        if GrantType::try_from(grant_type) == Ok(GrantType::Password) {
            let client_ip = ctx.ip.as_str();
            // ── Gate 1: the brute-force lockout (ip + username, 5 failures
            //    → 15 min) — every end, captcha-capable or not: the
            //    lockout is the primary anti-brute-force defense and
            //    never grants an exemption for missing captcha UI.
            if crate::login_rate_limiter::is_locked(&self.state.redis, client_ip, &login_username)
                .await
            {
                return Err(status_error(
                    "BAD_REQUEST",
                    "too many login failures, please try again later",
                ));
            }
            // ── Gate 2: the progressive captcha, only for clients that
            //    declared captcha support.
            if self.captcha_supported(&ctx) {
                self.verify_login_captcha(client_ip, &login_username, &ctx)
                    .await?;
            }
        }

        let mut core = self.state.core.clone();
        let login_result = core.login(tonic::Request::new(req)).await;
        match login_result {
            Ok(r) => {
                // A successful password login clears the counters (back
                // to the captcha-free state).
                if GrantType::try_from(grant_type) == Ok(GrantType::Password) {
                    crate::login_rate_limiter::reset(&self.state.redis, &ctx.ip, &login_username)
                        .await;
                }
                Ok(r.into_inner())
            }
            Err(e) => {
                // A credential-family miss records against both failure
                // counters (captcha errors and infrastructure failures
                // do not count). The reason rides the status details.
                let mapped = map_status(e);
                if GrantType::try_from(grant_type) == Ok(GrantType::Password)
                    && CREDENTIAL_FAIL_REASONS.contains(&mapped.reason)
                {
                    crate::login_rate_limiter::check_and_incr(
                        &self.state.redis,
                        &ctx.ip,
                        &login_username,
                    )
                    .await;
                }
                Err(mapped)
            }
        }
    }

    /// `GenerateCaptcha` — the challenge mint (auth-free, the whitelist
    /// carries it): render + store, answer only in Redis.
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

    async fn logout(&self, ctx: Ctx, _req: Empty) -> Result<Empty, StatusError> {
        let payload = operator_of(&ctx)?;
        let mut core = self.state.core.clone();
        core.logout(tonic::Request::new(
            proto::proto::authentication::service::v1::LogoutRequest {
                client_type: 1,
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
        mut req: LoginRequest,
    ) -> Result<LoginResponse, StatusError> {
        // The regular path: the gate injected the operator while the
        // access token was still valid. The fallback path: the access
        // token is already expired (this route is whitelisted — no
        // operator injection happened), so the uid/jti binding key is
        // sniffed off the expired JWT's payload — the core validates
        // the refresh token value itself.
        let operator = ctx
            .claims
            .as_ref()
            .and_then(crate::token::UserTokenPayload::from_claims);
        if let Some(op) = &operator {
            req.user_id = Some(op.user_id);
            req.jti = Some(op.jti.clone());
        } else if let Some(token) = bearer_of(&ctx) {
            if let Some((uid, jti)) = auth::parse_unverified_bearer_jwt(&token) {
                req.user_id.get_or_insert(uid);
                req.jti.get_or_insert(jti);
            }
        }
        // The grant type stays caller-borne (the reference pins only
        // the client type).
        req.client_type = Some(1);

        let mut core = self.state.core.clone();
        Ok(core
            .refresh_token(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    // 2026-11 契约批次新增：OIDC 登录的前置步骤（登录页尚无 token），
    // 纯转发——state 防 CSRF 由 core 生成校验。
    async fn get_oidc_authorize_url(
        &self,
        _ctx: Ctx,
        req: proto::proto::authentication::service::v1::GetOidcAuthorizeUrlRequest,
    ) -> Result<proto::proto::authentication::service::v1::GetOidcAuthorizeUrlResponse, StatusError>
    {
        let mut core = self.state.core.clone();
        Ok(core
            .get_oidc_authorize_url(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}
