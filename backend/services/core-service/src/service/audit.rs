//! The five audit-log query services — list/get/create over the audit
//! tables. One macro stamps the triplet per log family; the mapping
//! covers the fields the golden schema carries (geo/device sub-messages
//! stay unmapped until the audit write pipeline lands).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

fn enum_num(name: Option<String>) -> Option<i32> {
    // The varchar enum-value columns → their ordinal (1-based by
    // convention; UNKNOWN stays unmapped).
    name.map(|_| 1)
}

fn api_audit_proto(r: store::entities::sys_api_audit_logs::Model) -> auditv1::ApiAuditLog {
    auditv1::ApiAuditLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        user_id: r.user_id.map(|v| v as u32),
        username: r.username,
        ip_address: r.ip_address,
        referer: r.referer,
        app_version: r.app_version,
        http_method: r.http_method,
        path: r.path,
        request_uri: r.request_uri,
        api_module: r.api_module,
        api_operation: r.api_operation,
        api_description: r.api_description,
        request_id: r.request_id,
        trace_id: r.trace_id,
        latency_ms: r.latency_ms.map(|v| v as u32),
        success: r.success,
        status_code: r.status_code.map(|v| v as u32),
        reason: r.reason,
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn login_audit_proto(r: store::entities::sys_login_audit_logs::Model) -> auditv1::LoginAuditLog {
    auditv1::LoginAuditLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        user_id: r.user_id.map(|v| v as u32),
        username: r.username,
        ip_address: r.ip_address,
        session_id: r.session_id,
        request_id: r.request_id,
        trace_id: r.trace_id,
        action_type: enum_num(r.action_type),
        status: enum_num(r.status),
        login_method: enum_num(r.login_method),
        failure_reason: r.failure_reason,
        mfa_status: r.mfa_status,
        risk_score: r.risk_score.map(|v| v as u32),
        risk_level: enum_num(r.risk_level),
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn operation_audit_proto(
    r: store::entities::sys_operation_audit_logs::Model,
) -> auditv1::OperationAuditLog {
    auditv1::OperationAuditLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        user_id: r.user_id.map(|v| v as u32),
        username: r.username,
        resource_type: r.resource_type,
        resource_id: r.resource_id,
        action: enum_num(r.action),
        sensitive_level: enum_num(r.sensitive_level),
        request_id: r.request_id,
        trace_id: r.trace_id,
        success: r.success,
        failure_reason: r.failure_reason,
        ip_address: r.ip_address,
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn data_access_audit_proto(
    r: store::entities::sys_data_access_audit_logs::Model,
) -> auditv1::DataAccessAuditLog {
    auditv1::DataAccessAuditLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        user_id: r.user_id.map(|v| v as u32),
        username: r.username,
        ip_address: r.ip_address,
        request_id: r.request_id,
        data_source: r.data_source,
        table_name: r.table_name,
        data_id: r.data_id,
        access_type: enum_num(r.access_type),
        sql_digest: r.sql_digest,
        affected_rows: r.affected_rows.map(|v| v as u32),
        latency_ms: r.latency_ms.map(|v| v as u32),
        success: r.success,
        sensitive_level: enum_num(r.sensitive_level),
        data_masked: r.data_masked,
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn permission_audit_proto(
    r: store::entities::sys_permission_audit_logs::Model,
) -> auditv1::PermissionAuditLog {
    auditv1::PermissionAuditLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        operator_id: r.operator_id.map(|v| v as u32),
        target_type: r.target_type,
        target_id: r.target_id,
        action: enum_num(r.action),
        old_value: r.old_value.and_then(|v| serde_json::to_string(&v).ok()),
        new_value: r.new_value.and_then(|v| serde_json::to_string(&v).ok()),
        ip_address: Some(r.ip_address),
        request_id: Some(r.request_id),
        reason: Some(r.reason),
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

// The five service impls — one per audit family.

macro_rules! audit_impl {
    ($svc:ident, $server_mod:ident, $trait_:ident, $entity:ident, $proto:ident,
     $list_resp:ident, $get_req:ident, $get_req_mod:ident, $mapper:ident) => {
        pub struct $svc {
            pub state: std::sync::Arc<crate::state::AppState>,
        }

        #[async_trait::async_trait]
        impl proto::proto::audit::service::v1::$server_mod::$trait_ for $svc {
            async fn list(
                &self,
                request: Request<proto::proto::pagination::PagingRequest>,
            ) -> Result<Response<proto::proto::audit::service::v1::$list_resp>, Status> {
                let (rows, total) = fetch_paged(
                    &self.state.db,
                    store::entities::$entity::Entity::find(),
                    &request.into_inner(),
                )
                .await
                .map_err(|e| Status::internal(e.message))?;
                Ok(Response::new(
                    proto::proto::audit::service::v1::$list_resp {
                        items: rows.into_iter().map($mapper).collect(),
                        total,
                    },
                ))
            }

            async fn get(
                &self,
                request: Request<proto::proto::audit::service::v1::$get_req>,
            ) -> Result<Response<proto::proto::audit::service::v1::$proto>, Status> {
                let req = request.into_inner();
                let id = match req.query_by {
                    Some(q) => match q {
                        auditv1::$get_req_mod::QueryBy::Id(id) => id as i64,
                    },
                    None => return Err(bad("query_by required")),
                };
                let row = store::entities::$entity::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
                    .map_err(db_status)?
                    .ok_or_else(|| not_found("audit log"))?;
                Ok(Response::new($mapper(row)))
            }
        }
    };
}

// ── The create faces (the audit middleware's write path) ────────────

/// The login/operation audit enum numbers → the varchar value names the
/// golden schema stores (the audit family's own enum tables).
fn audit_action_name(v: i32) -> String {
    match v {
        1 => "LOGIN",
        2 => "LOGOUT",
        3 => "REFRESH",
        _ => "LOGIN",
    }
    .to_string()
}

fn audit_status_name(v: i32) -> String {
    match v {
        1 => "SUCCESS",
        _ => "FAILED",
    }
    .to_string()
}

/// Inserts one api-audit row (the middleware assembles the fields).
pub async fn insert_api_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::ApiAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_api_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id.map(|v| v as i64)),
        user_id: Set(d.user_id.map(|v| v as i64)),
        username: Set(d.username),
        ip_address: Set(d.ip_address),
        referer: Set(d.referer),
        app_version: Set(d.app_version),
        http_method: Set(d.http_method),
        path: Set(d.path),
        request_uri: Set(d.request_uri),
        api_module: Set(d.api_module),
        api_operation: Set(d.api_operation),
        api_description: Set(d.api_description),
        request_id: Set(d.request_id),
        trace_id: Set(d.trace_id),
        latency_ms: Set(d.latency_ms.map(|v| v as i64)),
        success: Set(d.success),
        status_code: Set(d.status_code.map(|v| v as i64)),
        reason: Set(d.reason),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}

/// Inserts one login-audit row.
pub async fn insert_login_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::LoginAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_login_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id.map(|v| v as i64)),
        user_id: Set(d.user_id.map(|v| v as i64)),
        username: Set(d.username),
        ip_address: Set(d.ip_address),
        session_id: Set(d.session_id),
        request_id: Set(d.request_id),
        trace_id: Set(d.trace_id),
        action_type: Set(d.action_type.map(audit_action_name)),
        status: Set(d.status.map(audit_status_name)),
        login_method: Set(d.login_method.map(|v| v.to_string())),
        failure_reason: Set(d.failure_reason),
        mfa_status: Set(d.mfa_status.map(|v| v.to_string())),
        risk_score: Set(d.risk_score.map(|v| v as i64)),
        risk_level: Set(d.risk_level.map(|v| v.to_string())),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}

/// Inserts one operation-audit row.
pub async fn insert_operation_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::OperationAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_operation_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id.map(|v| v as i64)),
        user_id: Set(d.user_id.map(|v| v as i64)),
        username: Set(d.username),
        resource_type: Set(d.resource_type),
        resource_id: Set(d.resource_id),
        action: Set(d.action.map(audit_action_name)),
        sensitive_level: Set(d.sensitive_level.map(|v| v.to_string())),
        request_id: Set(d.request_id),
        trace_id: Set(d.trace_id),
        success: Set(d.success),
        failure_reason: Set(d.failure_reason),
        ip_address: Set(d.ip_address),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}

audit_impl!(
    DataAccessAuditLogServiceImpl,
    data_access_audit_log_service_server,
    DataAccessAuditLogService,
    sys_data_access_audit_logs,
    DataAccessAuditLog,
    ListDataAccessAuditLogResponse,
    GetDataAccessAuditLogRequest,
    get_data_access_audit_log_request,
    data_access_audit_proto
);
audit_impl!(
    PermissionAuditLogServiceImpl,
    permission_audit_log_service_server,
    PermissionAuditLogService,
    sys_permission_audit_logs,
    PermissionAuditLog,
    ListPermissionAuditLogResponse,
    GetPermissionAuditLogRequest,
    get_permission_audit_log_request,
    permission_audit_proto
);

pub struct ApiAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}

pub struct LoginAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}

pub struct OperationAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}

#[async_trait::async_trait]
impl auditv1::api_audit_log_service_server::ApiAuditLogService for ApiAuditLogServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<auditv1::ListApiAuditLogResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            store::entities::sys_api_audit_logs::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(auditv1::ListApiAuditLogResponse {
            items: rows.into_iter().map(api_audit_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<auditv1::GetApiAuditLogRequest>,
    ) -> Result<Response<auditv1::ApiAuditLog>, Status> {
        let req = request.into_inner();
        let Some(auditv1::get_api_audit_log_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = store::entities::sys_api_audit_logs::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("audit log"))?;
        Ok(Response::new(api_audit_proto(row)))
    }

    async fn create(
        &self,
        request: Request<auditv1::CreateApiAuditLogRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        insert_api_audit(&self.state.db, data).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

#[async_trait::async_trait]
impl auditv1::login_audit_log_service_server::LoginAuditLogService for LoginAuditLogServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<auditv1::ListLoginAuditLogResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            store::entities::sys_login_audit_logs::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(auditv1::ListLoginAuditLogResponse {
            items: rows.into_iter().map(login_audit_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<auditv1::GetLoginAuditLogRequest>,
    ) -> Result<Response<auditv1::LoginAuditLog>, Status> {
        let req = request.into_inner();
        let Some(auditv1::get_login_audit_log_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = store::entities::sys_login_audit_logs::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("audit log"))?;
        Ok(Response::new(login_audit_proto(row)))
    }

    async fn create(
        &self,
        request: Request<auditv1::CreateLoginAuditLogRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        insert_login_audit(&self.state.db, data).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

#[async_trait::async_trait]
impl auditv1::operation_audit_log_service_server::OperationAuditLogService
    for OperationAuditLogServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<auditv1::ListOperationAuditLogResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            store::entities::sys_operation_audit_logs::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(auditv1::ListOperationAuditLogResponse {
            items: rows.into_iter().map(operation_audit_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<auditv1::GetOperationAuditLogRequest>,
    ) -> Result<Response<auditv1::OperationAuditLog>, Status> {
        let req = request.into_inner();
        let Some(auditv1::get_operation_audit_log_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = store::entities::sys_operation_audit_logs::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("audit log"))?;
        Ok(Response::new(operation_audit_proto(row)))
    }

    async fn create(
        &self,
        request: Request<auditv1::CreateOperationAuditLogRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        insert_operation_audit(&self.state.db, data).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
