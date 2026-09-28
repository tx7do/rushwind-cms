//! The login-audit-log service (login_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::{audit_action_name, enum_num};

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

fn audit_status_name(v: i32) -> String {
    match v {
        1 => "SUCCESS",
        _ => "FAILED",
    }
    .to_string()
}

/// Inserts one login-audit row (the middleware assembles the fields).
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

pub struct LoginAuditLogService {
    pub state: std::sync::Arc<crate::state::AppState>,
}

pub struct OperationAuditLogServiceImpl {
    pub state: std::sync::Arc<crate::state::AppState>,
}

#[async_trait::async_trait]
impl auditv1::login_audit_log_service_server::LoginAuditLogService for LoginAuditLogService {
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
