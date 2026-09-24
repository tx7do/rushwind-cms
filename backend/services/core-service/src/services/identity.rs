//! The identity read faces the BFFs surface most: User, Role, Tenant
//! listings and lookups (write paths land with the RBAC phase).

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{sys_roles, sys_tenants, sys_users};
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;
use proto::proto::permission::service::v1 as permissionv1;

pub(crate) fn user_proto(r: sys_users::Model) -> identityv1::User {
    identityv1::User {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        username: r.username,
        nickname: r.nickname,
        realname: r.realname,
        email: r.email,
        mobile: r.mobile,
        telephone: r.telephone,
        avatar: r.avatar,
        address: r.address,
        region: r.region,
        description: r.description,
        gender: r.gender.as_deref().map(|_| 1),
        last_login_at: r.last_login_at.and_then(ts_to_proto),
        last_login_ip: r.last_login_ip,
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct UserServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::user_service_server::UserService for UserServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListUserResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_users::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListUserResponse {
            items: rows.into_iter().map(user_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountUserResponse>, Status> {
        let total = sys_users::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountUserResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetUserRequest>,
    ) -> Result<Response<identityv1::User>, Status> {
        let req = request.into_inner();
        use identityv1::get_user_request::QueryBy;
        let row = match req.query_by {
            Some(QueryBy::Id(id)) => {
                sys_users::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Username(username)) => {
                sys_users::Entity::find()
                    .filter(sys_users::Column::Username.eq(username))
                    .one(&self.state.db)
                    .await
            }
            None => return Err(bad("query_by required")),
        }
        .map_err(db_status)?
        .ok_or_else(|| not_found("user"))?;
        Ok(Response::new(user_proto(row)))
    }
}

pub(crate) fn role_proto(r: sys_roles::Model) -> permissionv1::Role {
    permissionv1::Role {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        is_protected: Some(r.is_protected),
        r#type: Some(1),
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct RoleServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::role_service_server::RoleService for RoleServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListRoleResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_roles::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListRoleResponse {
            items: rows.into_iter().map(role_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetRoleRequest>,
    ) -> Result<Response<permissionv1::Role>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_role_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = sys_roles::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("role"))?;
        Ok(Response::new(role_proto(row)))
    }
}

pub(crate) fn tenant_proto(r: sys_tenants::Model) -> identityv1::Tenant {
    identityv1::Tenant {
        id: Some(r.id as u32),
        name: r.name,
        code: r.code,
        logo_url: r.logo_url,
        domain: r.domain,
        industry: r.industry,
        admin_user_id: r.admin_user_id.map(|v| v as u32),
        status: Some(1),
        r#type: Some(1),
        audit_status: r.audit_status.as_deref().map(|_| 1),
        subscription_plan: r.subscription_plan,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct TenantServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::tenant_service_server::TenantService for TenantServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListTenantResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_tenants::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListTenantResponse {
            items: rows.into_iter().map(tenant_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetTenantRequest>,
    ) -> Result<Response<identityv1::Tenant>, Status> {
        let req = request.into_inner();
        use identityv1::get_tenant_request::QueryBy;
        let row = match req.query_by {
            Some(QueryBy::Id(id)) => {
                sys_tenants::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Code(code)) => {
                sys_tenants::Entity::find()
                    .filter(sys_tenants::Column::Code.eq(code))
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Name(name)) => {
                sys_tenants::Entity::find()
                    .filter(sys_tenants::Column::Name.eq(name))
                    .one(&self.state.db)
                    .await
            }
            None => return Err(bad("query_by required")),
        }
        .map_err(db_status)?
        .ok_or_else(|| not_found("tenant"))?;
        Ok(Response::new(tenant_proto(row)))
    }

    async fn resolve_tenant_by_domain(
        &self,
        request: Request<identityv1::ResolveTenantByDomainRequest>,
    ) -> Result<Response<identityv1::ResolveTenantByDomainResponse>, Status> {
        let req = request.into_inner();
        let row = sys_tenants::Entity::find()
            .filter(sys_tenants::Column::Domain.eq(req.domain))
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tenant"))?;
        Ok(Response::new(identityv1::ResolveTenantByDomainResponse {
            tenant_id: row.id as u32,
        }))
    }
}
