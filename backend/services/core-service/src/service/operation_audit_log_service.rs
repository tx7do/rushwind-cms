//! The operation-audit-log service (operation_audit_log_service.go).


use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::{audit_action_name, enum_num};

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






pub struct OperationAuditLogService {
    pub state: std::sync::Arc<crate::state::AppState>,
}


#[async_trait::async_trait]
impl auditv1::operation_audit_log_service_server::OperationAuditLogService
    for OperationAuditLogService
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
