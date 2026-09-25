//! The tenant service — the complete tenant face: list/get/resolve plus
//! create/update/delete and the tenant-with-admin bootstrap, mirroring
//! the reference's tenant_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::{tenant_repo};
use crate::service::context::{
    optional_operator_user_id,
};
use crate::service::user_service::insert_credential;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_role_metadata, sys_role_permissions, sys_roles, sys_tenants, sys_user_roles,
    sys_users,
};
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;

const TENANT_ADMIN_TEMPLATE_ROLE_CODE: &str = "template:tenant:manager";
const TENANT_ADMIN_ROLE_CODE: &str = "tenant:manager";
const TENANT_ADMIN_ROLE_NAME: &str = "租户管理员";

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

// ── Tenant writes ────────────────────────────────────────────────────

pub struct TenantService {
    pub state: Arc<AppState>,
}


#[async_trait::async_trait]
impl identityv1::tenant_service_server::TenantService for TenantService {
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

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountTenantResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_tenants::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountTenantResponse {
            count: total,
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

    async fn create(
        &self,
        request: Request<identityv1::CreateTenantRequest>,
    ) -> Result<Response<identityv1::Tenant>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = sys_tenants::ActiveModel {
            name: Set(Some(data.name.clone().unwrap_or_default())),
            code: Set(Some(data.code.clone().unwrap_or_default())),
            domain: Set(data.domain),
            industry: Set(data.industry),
            status: Set(Some("ON".to_string())),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(tenant_proto(row)))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdateTenantRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_tenants::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tenant"))?;
        let mut a: sys_tenants::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.domain {
                a.domain = Set(Some(v));
            }
            if let Some(v) = data.status {
                a.status = Set(Some(if v == 1 { "ON" } else { "OFF" }.to_string()));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeleteTenantRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(identityv1::delete_tenant_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        tenant_repo::delete_tenants(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn tenant_exists(
        &self,
        request: Request<identityv1::TenantExistsRequest>,
    ) -> Result<Response<identityv1::TenantExistsResponse>, Status> {
        let req = request.into_inner();
        let row = sys_tenants::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(sys_tenants::Column::Code.eq(req.code.clone()))
                    .add(sys_tenants::Column::Name.eq(req.name.clone())),
            )
            .one(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::TenantExistsResponse {
            exist: row.is_some(),
        }))
    }

    /// CreateTenantWithAdminUser — one transaction: create the tenant,
    /// copy the tenant-admin role template into it, create the admin
    /// user with that role, write the bcrypt credential, and point the
    /// tenant's admin_user_id at the new user. Any failure rolls the
    /// whole chain back.
    async fn create_tenant_with_admin_user(
        &self,
        request: Request<identityv1::CreateTenantWithAdminUserRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // 操作人身份必须从元数据推导，忽略客户端传入的 operator_user_id，
        // 防止伪造审计归属（缺失时按 0 记，与参照一致）
        let operator_id = optional_operator_user_id(&request);
        let req = request.into_inner();
        let Some(tenant) = req.tenant else {
            return Err(bad("invalid parameter"));
        };
        let Some(user) = req.user else {
            return Err(bad("invalid parameter"));
        };

        // The reference's duplicate check discards the flag (dead guard):
        // it only surfaces DB failures — the code unique index remains the
        // real conflict guard. Kept query-for-query.
        let _ = sys_tenants::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(sys_tenants::Column::Code.eq(tenant.code.clone().unwrap_or_default()))
                    .add(sys_tenants::Column::Name.eq(tenant.name.clone().unwrap_or_default())),
            )
            .one(&self.state.db)
            .await
            .map_err(db_status)?;

        let txn = self.state.db.begin().await.map_err(db_status)?;
        let now = store::now();

        // 1. The tenant row.
        let tenant_row = sys_tenants::ActiveModel {
            name: Set(tenant.name.clone()),
            code: Set(tenant.code.clone()),
            domain: Set(tenant.domain.clone()),
            logo_url: Set(tenant.logo_url.clone()),
            industry: Set(tenant.industry.clone()),
            remark: Set(tenant.remark.clone()),
            subscription_plan: Set(tenant.subscription_plan.clone()),
            status: Set(Some("ON".to_string())),
            created_by: Set(tenant.created_by.map(|v| v as i64)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;
        let tenant_id = tenant_row.id;

        // 2. Copy the tenant-admin role template (protected, platform
        //    owned, ON) into the tenant.
        let template = sys_roles::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(sys_roles::Column::Code.eq(TENANT_ADMIN_TEMPLATE_ROLE_CODE))
                    .add(sys_roles::Column::Status.eq("ON"))
                    .add(sys_roles::Column::IsProtected.eq(true))
                    .add(
                        sea_orm::Condition::any()
                            .add(sys_roles::Column::TenantId.is_null())
                            .add(sys_roles::Column::TenantId.eq(0)),
                    ),
            )
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tenant admin role template"))?;
        let template_permission_ids: Vec<i64> = sys_role_permissions::Entity::find()
            .filter(sys_role_permissions::Column::RoleId.eq(template.id))
            .all(&txn)
            .await
            .map_err(db_status)?
            .into_iter()
            .map(|p| p.permission_id)
            .collect();

        let role_row = sys_roles::ActiveModel {
            name: Set(Some(TENANT_ADMIN_ROLE_NAME.to_string())),
            code: Set(Some(TENANT_ADMIN_ROLE_CODE.to_string())),
            r#type: Set("TENANT".to_string()),
            is_protected: Set(true),
            tenant_id: Set(Some(tenant_id)),
            created_by: Set(Some(operator_id)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        // Role metadata: not a template, points back at the role code it
        // was minted from; auto-sync, tenant scope.
        sys_role_metadata::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            role_id: Set(Some(role_row.id)),
            is_template: Set(Some(false)),
            template_for: Set(Some(TENANT_ADMIN_ROLE_CODE.to_string())),
            sync_policy: Set(Some("AUTO".to_string())),
            scope: Set(Some("TENANT".to_string())),
            created_by: Set(Some(operator_id)),
            created_at: Set(Some(now)),
            custom_overrides: Set(serde_json::json!({})),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        for pid in template_permission_ids {
            sys_role_permissions::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                role_id: Set(role_row.id),
                permission_id: Set(pid),
                status: Set("ON".to_string()),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }

        // 3. The admin user, bound to the copied role (the reference
        //    appends user.role_id to user.role_ids and dedupes).
        let admin_row = sys_users::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            username: Set(user.username.clone()),
            nickname: Set(user.nickname.clone()),
            realname: Set(user.realname.clone()),
            email: Set(user.email.clone()),
            mobile: Set(user.mobile.clone()),
            avatar: Set(user.avatar.clone()),
            status: Set(Some("NORMAL".to_string())),
            created_by: Set(user.created_by.map(|v| v as i64)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(db_status)?;

        let mut role_ids: Vec<i64> = Vec::new();
        if let Some(role_id) = user.role_id {
            role_ids.push(role_id as i64);
        }
        for role_id in user.role_ids.iter().map(|v| *v as i64) {
            if !role_ids.contains(&role_id) {
                role_ids.push(role_id);
            }
        }
        for role_id in role_ids {
            sys_user_roles::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                user_id: Set(admin_row.id),
                role_id: Set(role_id),
                is_primary: Set(true),
                status: Set("ACTIVE".to_string()),
                assigned_at: Set(Some(now)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(db_status)?;
        }

        // 4. The bcrypt credential (USERNAME / PASSWORD_HASH, primary).
        insert_credential(
            &txn,
            tenant_id,
            admin_row.id,
            &admin_row.username.clone().unwrap_or_default(),
            &req.password,
        )
        .await?;

        // 5. Point the tenant at its admin user.
        sys_tenants::ActiveModel {
            id: Set(tenant_id),
            admin_user_id: Set(Some(admin_row.id)),
            ..Default::default()
        }
        .update(&txn)
        .await
        .map_err(db_status)?;

        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    /// 按域名解析租户 id（app BFF 匿名链路的 Host 归属）：domain 精确
    /// 匹配，未中按去端口主机名回退一次（开发前端恒带端口）；未匹配
    /// 返回 0，调用方据此 fail-closed。
    async fn resolve_tenant_by_domain(
        &self,
        request: Request<identityv1::ResolveTenantByDomainRequest>,
    ) -> Result<Response<identityv1::ResolveTenantByDomainResponse>, Status> {
        let domain = request.into_inner().domain;
        let lookup = |d: String| {
            sys_tenants::Entity::find()
                .filter(sys_tenants::Column::Domain.eq(d))
                .one(&self.state.db)
        };
        let mut row = lookup(domain.clone()).await.map_err(db_status)?;
        if row.is_none() {
            if let Some((host, _port)) = domain.split_once(':') {
                if !host.is_empty() {
                    row = lookup(host.to_string()).await.map_err(db_status)?;
                }
            }
        }
        Ok(Response::new(identityv1::ResolveTenantByDomainResponse {
            tenant_id: row.map(|r| r.id as u32).unwrap_or(0),
        }))
    }
}
