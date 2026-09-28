//! The data-access-audit-log service (data_access_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::{audit_impl, enum_num};

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

fn data_access_type_name(v: i32) -> Option<String> {
    auditv1::data_access_audit_log::AccessType::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn sensitive_level_name(v: i32) -> Option<String> {
    auditv1::SensitiveLevel::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

/// The old/new value text → the JSON column (valid JSON keeps its
/// shape; free text lands as a JSON string).
pub async fn insert_data_access_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::DataAccessAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_data_access_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id.map(|v| v as i64)),
        user_id: Set(d.user_id.map(|v| v as i64)),
        username: Set(d.username),
        ip_address: Set(d.ip_address),
        request_id: Set(d.request_id),
        data_source: Set(d.data_source),
        table_name: Set(d.table_name),
        data_id: Set(d.data_id),
        access_type: Set(d.access_type.and_then(data_access_type_name)),
        sql_digest: Set(d.sql_digest),
        sql_text: Set(d.sql_text),
        affected_rows: Set(d.affected_rows.map(|v| v as i64)),
        latency_ms: Set(d.latency_ms.map(|v| v as i64)),
        success: Set(d.success),
        sensitive_level: Set(d.sensitive_level.and_then(sensitive_level_name)),
        data_masked: Set(d.data_masked),
        masking_rules: Set(d.masking_rules),
        business_purpose: Set(d.business_purpose),
        data_category: Set(d.data_category),
        db_user: Set(d.db_user),
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
    DataAccessAuditLogService,
    data_access_audit_log_service_server,
    DataAccessAuditLogService,
    sys_data_access_audit_logs,
    DataAccessAuditLog,
    ListDataAccessAuditLogResponse,
    GetDataAccessAuditLogRequest,
    get_data_access_audit_log_request,
    data_access_audit_proto,
    CreateDataAccessAuditLogRequest,
    insert_data_access_audit
);
