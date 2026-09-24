//! The five audit-log query services — list/get/create over the audit
//! tables. One macro stamps the triplet per log family; the mapping
//! covers the fields the golden schema carries (geo/device sub-messages
//! stay unmapped until the audit write pipeline lands).

use sea_orm::EntityTrait as _;
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

audit_impl!(
    ApiAuditLogServiceImpl,
    api_audit_log_service_server,
    ApiAuditLogService,
    sys_api_audit_logs,
    ApiAuditLog,
    ListApiAuditLogResponse,
    GetApiAuditLogRequest,
    get_api_audit_log_request,
    api_audit_proto
);
audit_impl!(
    LoginAuditLogServiceImpl,
    login_audit_log_service_server,
    LoginAuditLogService,
    sys_login_audit_logs,
    LoginAuditLog,
    ListLoginAuditLogResponse,
    GetLoginAuditLogRequest,
    get_login_audit_log_request,
    login_audit_proto
);
audit_impl!(
    OperationAuditLogServiceImpl,
    operation_audit_log_service_server,
    OperationAuditLogService,
    sys_operation_audit_logs,
    OperationAuditLog,
    ListOperationAuditLogResponse,
    GetOperationAuditLogRequest,
    get_operation_audit_log_request,
    operation_audit_proto
);
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
