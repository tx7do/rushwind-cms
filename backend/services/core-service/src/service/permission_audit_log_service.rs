//! The permission-audit-log service (permission_audit_log_service.go).


use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::{audit_impl, enum_num};

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


fn permission_action_name(v: i32) -> Option<String> {
    auditv1::permission_audit_log::ActionType::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

/// The old/new value text → the JSON column (valid JSON keeps its
/// shape; free text lands as a JSON string).
fn audit_json_text(v: Option<String>) -> Option<serde_json::Value> {
    v.map(|v| serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v.clone())))
}

/// Inserts one permission-audit row.
pub async fn insert_permission_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::PermissionAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_permission_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id.map(|v| v as i64)),
        operator_id: Set(d.operator_id.map(|v| v as i64)),
        target_type: Set(d.target_type),
        target_id: Set(d.target_id),
        action: Set(d.action.and_then(permission_action_name)),
        old_value: Set(audit_json_text(d.old_value)),
        new_value: Set(audit_json_text(d.new_value)),
        ip_address: Set(d.ip_address.unwrap_or_default()),
        request_id: Set(d.request_id.unwrap_or_default()),
        reason: Set(d.reason.unwrap_or_default()),
        log_hash: Set(d.log_hash),
        signature: Set(d.signature),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}


audit_impl!(
    PermissionAuditLogService,
    permission_audit_log_service_server,
    PermissionAuditLogService,
    sys_permission_audit_logs,
    PermissionAuditLog,
    ListPermissionAuditLogResponse,
    GetPermissionAuditLogRequest,
    get_permission_audit_log_request,
    permission_audit_proto,
    CreatePermissionAuditLogRequest,
    insert_permission_audit
);
