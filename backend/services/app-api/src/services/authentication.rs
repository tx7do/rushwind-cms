//! AuthenticationService — the app (C-side) BFF face of the login
//! chain. The domain logic lives in the core service; this BFF layer
//! carries the client-facing concerns: the app client-type pin (1),
//! the short-lived app token profile (refresh disabled), and the
//! cookie pair (path-narrowed to the app refresh endpoint).

use std::sync::Arc;

use pbjson_types::Empty;
use proto::proto::authentication::service::v1::{LoginRequest, LoginResponse};

use crate::services::map_status;
use crate::state::{operator_of, AppState, StatusError};

type Ctx = rushwind_http_binding::ctx::RequestContext;

/// The app client's refresh-expiry stamp (the authenticator's app
/// profile disables refresh — the cookie mirror stays for shape
/// parity; the core store writes no RT row under a 0s expiry).
const REFRESH_TTL_SECS: i64 = 0;

pub struct AuthenticationService {
    pub state: Arc<AppState>,
}

impl AuthenticationService {
    fn set_refresh_cookies(&self, ctx: &Ctx, refresh: &str, secure: bool) {
        if refresh.is_empty() {
            return;
        }
        let (rt, exp) = crate::token::refresh_cookie_values(refresh, secure, REFRESH_TTL_SECS);
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
}

#[async_trait::async_trait]
impl proto::gen_app::services::AuthenticationServiceHandlers for AuthenticationService {
    async fn login(&self, ctx: Ctx, req: LoginRequest) -> Result<LoginResponse, StatusError> {
        // Pin the client type (app) and forward to core. The app login
        // carries no captcha gate (the reference gates only the admin
        // form).
        let mut req = req;
        req.client_type = Some(1);
        let mut core = self.state.core.clone();
        let mut resp = core
            .login(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner();

        let secure = Self::cookie_secure(&ctx);
        if let Some(rt) = resp.refresh_token.take() {
            self.set_refresh_cookies(&ctx, &rt, secure);
        }
        Ok(resp)
    }

    async fn logout(&self, ctx: Ctx, _req: Empty) -> Result<Empty, StatusError> {
        let payload = operator_of(&ctx)?;
        self.clear_refresh_cookies(&ctx, Self::cookie_secure(&ctx));
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
        req: LoginRequest,
    ) -> Result<LoginResponse, StatusError> {
        let mut req = req;
        if req.refresh_token.as_deref().unwrap_or("").is_empty() {
            if let Some(cv) = ctx.cookies.get("refresh_token") {
                req.refresh_token = Some(cv.clone());
            }
        }
        let operator = ctx
            .claims
            .as_ref()
            .and_then(crate::token::UserTokenPayload::from_claims);
        if req.user_id.is_none() {
            req.user_id = operator.as_ref().map(|o| o.user_id);
        }
        if req.jti.as_deref().unwrap_or("").is_empty() {
            req.jti = operator.as_ref().map(|o| o.jti.clone());
        }
        req.client_type = Some(1);
        req.grant_type = proto::proto::authentication::service::v1::GrantType::RefreshToken as i32;

        let mut core = self.state.core.clone();
        let mut resp = core
            .refresh_token(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner();

        let secure = Self::cookie_secure(&ctx);
        if let Some(rt) = resp.refresh_token.take() {
            self.set_refresh_cookies(&ctx, &rt, secure);
        }
        Ok(resp)
    }
}
