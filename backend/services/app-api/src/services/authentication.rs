//! AuthenticationService — the app (C-side) BFF face of the login
//! chain. The domain logic lives in the core service; this BFF layer
//! carries the client-facing concerns: the app client-type pin (1), the
//! refresh binding's uid/jti fallback off the (possibly expired) bearer
//! JWT, and the body-borne refresh token (the reference's app BFF sets
//! no cookies — the refresh token stays in the JSON body).

use std::sync::Arc;

use pbjson_types::Empty;
use proto::proto::authentication::service::v1::{LoginRequest, LoginResponse};

use crate::services::map_status;
use crate::state::{operator_of, AppState, StatusError};

type Ctx = rushwind_http_binding::ctx::RequestContext;

pub struct AuthenticationService {
    pub state: Arc<AppState>,
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
        let _ = ctx;
        // Pin the client type (app) and forward to core. The app login
        // carries no captcha gate (the reference gates only the admin
        // form), and the refresh token stays in the response body.
        req.client_type = Some(1);
        let mut core = self.state.core.clone();
        Ok(core
            .login(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
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
}
