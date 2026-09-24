//! The domain AuthenticationService — the reference's core login
//! chain, gRPC face:
//!
//! * Login (password grant): tenant resolve → credential verify (AES
//!   unwrap + bcrypt + dummy equalizer) → user status → permission gate
//!   (sys:access_backend / sys:access_app per client type) → HS256
//!   access JWT + opaque refresh, registered in the `gwc:` key family;
//!   refresh grant: verify-and-revoke rotation then re-mint.
//! * Logout: revokes every token of the user (SCAN prefix family).
//! * RegisterUser: the C-side register flow (user + credential + the
//!   tenant:user role binding, one transaction).
//! * ValidateToken: the BFF session check (whitelist + blacklist).
//! * WhoAmI: the operator echo off a presented token.

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

use proto::proto::authentication::service::v1::{
    authentication_service_server::AuthenticationService, BlockTokenRequest, BlockTokenResponse,
    GetAccessTokensRequest, GetAccessTokensResponse, LoginRequest, LoginResponse, LogoutRequest,
    RegisterUserRequest, RegisterUserResponse, RevokeTokenByIdRequest, UnblockTokenRequest,
    ValidateTokenRequest, ValidateTokenResponse, WhoAmIResponse,
};
use store::auth::UserTokenPayload;

use crate::state::{bad, db_status, forbidden, not_found, AppState};
use crate::token::{ClientType, TokenStore};

pub struct AuthenticationServiceImpl {
    pub state: Arc<AppState>,
}

impl AuthenticationServiceImpl {
    fn client_of(req: &LoginRequest) -> ClientType {
        ClientType::from_i32(req.client_type.unwrap_or(0))
    }

    fn client_of_i32(v: i32) -> ClientType {
        ClientType::from_i32(v)
    }

    /// The login permission gate per client profile.
    fn required_permission(client: ClientType) -> &'static str {
        match client {
            ClientType::Admin => "sys:access_backend",
            ClientType::App => "sys:access_app",
        }
    }

    /// Mints the pair and registers the Redis rows.
    async fn issue_pair(
        &self,
        client: ClientType,
        payload: &UserTokenPayload,
    ) -> Result<(String, Option<String>), Status> {
        use rushwind_authn::Authenticator as _;
        let store = self.state.tokens(client);
        let now = chrono::Utc::now().timestamp();
        let access = self
            .state
            .jwt
            .create_identity(&rushwind_authn::AuthClaims(
                payload.to_access_claims(now, now + store.access_secs()),
            ))
            .map_err(|_| Status::internal("create access token failed"))?;
        let refresh = (store.refresh_secs() > 0).then(TokenStore::new_refresh_token);
        store
            .add_token_pair(payload.user_id, &payload.jti, &access, refresh.as_deref())
            .await
            .map_err(Status::internal)?;
        Ok((access, refresh))
    }

    fn login_response(
        &self,
        client: ClientType,
        access: String,
        refresh: Option<String>,
    ) -> LoginResponse {
        let store = self.state.tokens(client);
        LoginResponse {
            token_type: proto::proto::authentication::service::v1::TokenType::Bearer as i32,
            access_token: access,
            refresh_token: refresh,
            expires_in: store.access_secs(),
            refresh_expires_in: Some(store.refresh_secs()),
            ..Default::default()
        }
    }

    /// Builds the payload for an authenticated user — fresh roles via
    /// the permission gate.
    async fn payload_for(
        &self,
        user: &store::entities::sys_users::Model,
        client: ClientType,
        req: &LoginRequest,
    ) -> Result<UserTokenPayload, Status> {
        let mut payload = UserTokenPayload {
            user_id: user.id as u32,
            tenant_id: user.tenant_id.unwrap_or(0) as u32,
            username: user.username.clone().unwrap_or_default(),
            jti: uuid::Uuid::now_v7().simple().to_string(),
            client_id: req.client_id.clone(),
            device_id: req.device_id.clone(),
            ..Default::default()
        };
        store::auth::resolve_authority(
            &self.state.db,
            user.id,
            Self::required_permission(client),
            &mut payload.roles,
        )
        .await
        .map_err(to_tonic)?;
        Ok(payload)
    }

    async fn user_row(&self, id: i64) -> Result<store::entities::sys_users::Model, Status> {
        store::entities::sys_users::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("user"))
    }

    /// The password grant (the reference's doGrantTypePassword).
    async fn do_password(&self, req: &LoginRequest) -> Result<LoginResponse, Status> {
        let client = Self::client_of(req);

        // Tenant resolve: blank code = platform.
        let tenant_id = match req.tenant_code.as_deref() {
            Some(code) if !code.trim().is_empty() => {
                match store::entities::sys_tenants::Entity::find()
                    .filter(store::entities::sys_tenants::Column::Code.eq(code.trim()))
                    .one(&self.state.db)
                    .await
                    .map_err(db_status)?
                {
                    Some(t) if t.status.as_deref() == Some("ON") => t.id,
                    _ => return Err(bad("invalid tenant")),
                }
            }
            _ => 0,
        };

        use proto::proto::authentication::service::v1::login_request::Identifier;
        let identifier = match &req.identifier {
            Some(Identifier::Email(email)) if !email.trim().is_empty() => {
                ("EMAIL", email.trim().to_string())
            }
            Some(Identifier::Mobile(mobile)) if !mobile.trim().is_empty() => {
                ("MOBILE", mobile.trim().to_string())
            }
            Some(Identifier::Username(username)) => ("USERNAME", username.clone()),
            _ => ("USERNAME", String::new()),
        };

        let user_id = store::auth::verify_credential(
            &self.state.db,
            tenant_id,
            identifier,
            req.password.as_deref().unwrap_or(""),
        )
        .await
        .map_err(to_tonic)?;

        let user = self.user_row(user_id).await?;
        if user.tenant_id.unwrap_or(0) != tenant_id {
            return Err(bad("invalid tenant"));
        }
        if user.status.as_deref() != Some("NORMAL") {
            return Err(forbidden("user is disabled"));
        }

        let payload = self.payload_for(&user, client, req).await?;
        let (access, refresh) = self.issue_pair(client, &payload).await?;
        Ok(self.login_response(client, access, refresh))
    }

    /// The refresh grant (the reference's doGrantTypeRefreshToken): the
    /// binding keys come from the verified token/request, the rotation
    /// runs verify-and-revoke, then a fresh pair mints.
    async fn do_refresh(&self, req: &LoginRequest) -> Result<LoginResponse, Status> {
        let client = Self::client_of(req);
        let user_id = req.user_id.unwrap_or(0);
        let jti = req.jti.clone().unwrap_or_default();
        let refresh_token = req.refresh_token.clone().unwrap_or_default();
        if user_id == 0 || jti.is_empty() || refresh_token.is_empty() {
            return Err(Status::unauthenticated("invalid refresh token"));
        }
        let valid = self
            .state
            .tokens(client)
            .verify_and_revoke_pair(user_id, &jti, &refresh_token)
            .await
            .map_err(Status::internal)?;
        if !valid {
            return Err(Status::unauthenticated("invalid refresh token"));
        }

        let user = self.user_row(user_id as i64).await?;
        if user.status.as_deref() != Some("NORMAL") {
            return Err(forbidden("user is disabled"));
        }
        let payload = self.payload_for(&user, client, req).await?;
        let (access, refresh) = self.issue_pair(client, &payload).await?;
        Ok(self.login_response(client, access, refresh))
    }
}

#[async_trait::async_trait]
impl AuthenticationService for AuthenticationServiceImpl {
    async fn login(&self, request: Request<LoginRequest>) -> ResponseResult<LoginResponse> {
        let req = request.into_inner();
        use proto::proto::authentication::service::v1::GrantType;
        let resp = match GrantType::try_from(req.grant_type) {
            Ok(GrantType::Password) => self.do_password(&req).await?,
            Ok(GrantType::RefreshToken) => self.do_refresh(&req).await?,
            _ => return Err(bad("invalid grant type")),
        };
        Ok(Response::new(resp))
    }

    async fn logout(&self, request: Request<LogoutRequest>) -> ResponseResult<pbjson_types::Empty> {
        let req = request.into_inner();
        let client = Self::client_of_i32(req.client_type);
        if req.user_id == 0 {
            return Err(bad("invalid user id"));
        }
        self.state
            .tokens(client)
            .revoke_user_tokens(req.user_id)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> ResponseResult<RegisterUserResponse> {
        let req = request.into_inner();
        let user_id = store::auth::register_user(
            &self.state.db,
            &req.tenant_code.clone(),
            &req.username,
            req.password.as_str(),
            req.email.as_deref(),
        )
        .await
        .map_err(to_tonic)?;
        Ok(Response::new(RegisterUserResponse {
            user_id: user_id as u32,
        }))
    }

    async fn refresh_token(&self, request: Request<LoginRequest>) -> ResponseResult<LoginResponse> {
        let req = request.into_inner();
        Ok(Response::new(self.do_refresh(&req).await?))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> ResponseResult<ValidateTokenResponse> {
        let req = request.into_inner();
        let client = Self::client_of_i32(req.client_type);
        // Signature + expiry first (local), then the session families.
        use rushwind_authn::Authenticator as _;
        let Ok(claims) = self.state.jwt.authenticate_token(&req.token) else {
            return Ok(Response::new(ValidateTokenResponse {
                is_valid: false,
                is_blocked: false,
                payload: None,
            }));
        };
        let payload = match UserTokenPayload::from_claims(&claims.0) {
            Some(p) => p,
            None => {
                return Ok(Response::new(ValidateTokenResponse {
                    is_valid: false,
                    is_blocked: false,
                    payload: None,
                }))
            }
        };
        let store = self.state.tokens(client);
        let mut valid = store
            .is_valid_access_token(payload.user_id, &payload.jti, &req.token)
            .await;
        if valid && req.skip_redis != Some(true) {
            valid = !store.is_blocked(&payload.jti).await;
        }
        let resp = if valid {
            let data_scope = payload.data_scope.as_deref().and_then(data_scope_of);
            ValidateTokenResponse {
                payload: Some(
                    proto::proto::authentication::service::v1::UserTokenPayload {
                        user_id: payload.user_id,
                        tenant_id: Some(payload.tenant_id),
                        username: Some(payload.username.clone()),
                        client_id: payload.client_id.clone(),
                        device_id: payload.device_id.clone(),
                        jti: Some(payload.jti.clone()),
                        roles: payload.roles.clone(),
                        org_unit_id: payload.org_unit_id,
                        data_scope,
                        ..Default::default()
                    },
                ),
                is_valid: true,
                is_blocked: false,
            }
        } else {
            ValidateTokenResponse {
                is_valid: false,
                ..Default::default()
            }
        };
        Ok(Response::new(resp))
    }

    async fn get_access_tokens(
        &self,
        _request: Request<GetAccessTokensRequest>,
    ) -> ResponseResult<GetAccessTokensResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn revoke_token_by_id(
        &self,
        _request: Request<RevokeTokenByIdRequest>,
    ) -> ResponseResult<pbjson_types::Empty> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn block_token(
        &self,
        _request: Request<BlockTokenRequest>,
    ) -> ResponseResult<BlockTokenResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn unblock_token(
        &self,
        _request: Request<UnblockTokenRequest>,
    ) -> ResponseResult<pbjson_types::Empty> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn who_am_i(
        &self,
        request: Request<pbjson_types::Empty>,
    ) -> ResponseResult<WhoAmIResponse> {
        // The BFF forwards the operator metadata; absent → anonymous.
        let _ = request;
        Err(Status::unimplemented(
            "route through the BFF operator context",
        ))
    }

    async fn generate_captcha(
        &self,
        _request: Request<pbjson_types::Empty>,
    ) -> ResponseResult<proto::proto::authentication::service::v1::GenerateCaptchaResponse> {
        // Captcha is a BFF concern (Redis-backed, client-facing) — the
        // reference keeps it out of core too.
        Err(Status::unimplemented("captcha is a BFF concern"))
    }
}

/// The boxed result alias the generated faces use.
pub type ResponseResult<T> = Result<Response<T>, Status>;

/// The envelope StatusError → the tonic Status (HTTP-ish status → gRPC
/// code: 400 → InvalidArgument, 401/403 → Unauthenticated/PermissionDenied,
/// 404 → NotFound, 409 → AlreadyExists, else Internal).
fn to_tonic(e: rushwind_http_binding::envelope::StatusError) -> Status {
    let code = match e.status {
        400 => tonic::Code::InvalidArgument,
        401 => tonic::Code::Unauthenticated,
        403 => tonic::Code::PermissionDenied,
        404 => tonic::Code::NotFound,
        409 => tonic::Code::AlreadyExists,
        _ => tonic::Code::Internal,
    };
    Status::new(code, e.message)
}

/// The DataScope enum name → value (`identity.service.v1.DataScope`).
fn data_scope_of(name: &str) -> Option<i32> {
    use proto::proto::identity::service::v1::DataScope;
    Some(match name {
        "ALL" => DataScope::All as i32,
        "SELF" => DataScope::Self_ as i32,
        "UNIT_ONLY" => DataScope::UnitOnly as i32,
        "UNIT_AND_CHILD" => DataScope::UnitAndChild as i32,
        "SELECTED_UNITS" => DataScope::SelectedUnits as i32,
        _ => return None,
    })
}
