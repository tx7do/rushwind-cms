//! The app-side user profile face (the me view, the password change,
//! the profile edit) — one service, mirroring the contract's
//! user-profile face.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::data::{user_repo};
use crate::service::context::{
    operator_of,
};
use crate::service::user_service::{user_proto, UserService};
use crate::state::AppState;

use proto::proto::identity::service::v1 as identityv1;

// ── UserProfile (the app face) ───────────────────────────────────────

pub struct UserProfileService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::user_profile_service_server::UserProfileService for UserProfileService {
    async fn get_user(
        &self,
        request: Request<pbjson_types::Empty>,
    ) -> Result<Response<identityv1::User>, Status> {
        let uid = operator_of(&request)?;
        let row = user_repo::users_by_id(&self.state.db, uid).await?;
        Ok(Response::new(user_proto(row)))
    }

    async fn update_user(
        &self,
        request: Request<identityv1::UpdateUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let uid = operator_of(&request)?;
        let mut req = request.into_inner();
        req.id = uid as u32;
        <UserService as identityv1::user_service_server::UserService>::update(
            &UserService {
                state: Arc::clone(&self.state),
            },
            Request::new(req),
        )
        .await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
