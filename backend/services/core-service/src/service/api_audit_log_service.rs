//! The api-audit-log service (api_audit_log_service.go).


use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;


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


pub struct ApiAuditLogService {
    pub state: std::sync::Arc<crate::state::AppState>,
}

pub struct LoginAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}

pub struct OperationAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}


#[async_trait::async_trait]
impl auditv1::api_audit_log_service_server::ApiAuditLogService for ApiAuditLogService {
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

