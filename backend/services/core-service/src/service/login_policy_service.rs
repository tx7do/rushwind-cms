//! The login-policy face — per-tenant authentication policy CRUD
//! (type/method as the varchar enum names the reference stores).
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::sys_login_policies;
use store::paging::fetch_paged;

use proto::proto::authentication::service::v1 as authv1;

/// The proto enum ordinal → the varchar value name the golden schema
/// stores (the EnumTypeConverter name-map behavior; unknown ordinals
/// stay unset).
macro_rules! enum_name {
    ($ty:ty) => {
        |v: i32| -> Option<String> {
            <$ty as TryFrom<i32>>::try_from(v)
                .ok()
                .map(|e| e.as_str_name().to_string())
        }
    };
}

// ── LoginPolicy ──────────────────────────────────────────────────────

fn login_policy_proto(r: sys_login_policies::Model) -> authv1::LoginPolicy {
    authv1::LoginPolicy {
        id: Some(r.id as u32),
        target_id: r.target_id.map(|v| v as u32),
        method: r.method.map(|_| 1),
        value: r.value,
        reason: r.reason,
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct LoginPolicyService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl authv1::login_policy_service_server::LoginPolicyService for LoginPolicyService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<authv1::ListLoginPolicyResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_login_policies::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(authv1::ListLoginPolicyResponse {
            items: rows.into_iter().map(login_policy_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<authv1::GetLoginPolicyRequest>,
    ) -> Result<Response<authv1::LoginPolicy>, Status> {
        let req = request.into_inner();
        let Some(authv1::get_login_policy_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = login_policy_repo::login_policies_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(login_policy_proto(row)))
    }

    async fn create(
        &self,
        request: Request<authv1::CreateLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let mut a = sys_login_policies::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            target_id: Set(data.target_id.map(|v| v as i64)),
            value: Set(data.value),
            reason: Set(data.reason),
            created_by: Set(data.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(name) = data.r#type.and_then(enum_name!(authv1::login_policy::Type)) {
            a.r#type = Set(Some(name));
        }
        if let Some(name) = data
            .method
            .and_then(enum_name!(authv1::login_policy::Method))
        {
            a.method = Set(Some(name));
        }
        a.insert(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<authv1::UpdateLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 不存在则创建
        if req.allow_missing.unwrap_or(false)
            && login_policy_repo::login_policies_by_id(&self.state.db, req.id as i64)
                .await
                .is_err()
        {
            let mut create_data = data;
            create_data.created_by = create_data.updated_by;
            create_data.updated_by = None;
            Self {
                state: Arc::clone(&self.state),
            }
            .create(Request::new(authv1::CreateLoginPolicyRequest {
                data: Some(create_data),
            }))
            .await?;
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        let mut a = <sys_login_policies::ActiveModel as std::default::Default>::default();
        if let Some(v) = data.target_id {
            a.target_id = Set(Some(v as i64));
        }
        if let Some(name) = data.r#type.and_then(enum_name!(authv1::login_policy::Type)) {
            a.r#type = Set(Some(name));
        }
        if let Some(name) = data
            .method
            .and_then(enum_name!(authv1::login_policy::Method))
        {
            a.method = Set(Some(name));
        }
        if let Some(v) = data.value {
            a.value = Set(Some(v));
        }
        if let Some(v) = data.reason {
            a.reason = Set(Some(v));
        }
        a.updated_at = Set(Some(store::now()));
        sys_login_policies::Entity::update_many()
            .set(a)
            .filter(sys_login_policies::Column::Id.eq(req.id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<authv1::DeleteLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(authv1::delete_login_policy_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        login_policy_repo::delete_login_policies(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::login_policy_repo;
