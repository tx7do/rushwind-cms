//! The data-access-audit-log service (data_access_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::audit_impl;

// The varchar enum columns ride the proto enum value names on both
// sides — the golden schema stores what the reference's ent enums
// define, and the reference's EnumTypeConverter pairs the two
// directions.

fn data_access_audit_proto(
    r: store::entities::sys_data_access_audit_logs::Model,
) -> auditv1::DataAccessAuditLog {
    auditv1::DataAccessAuditLog {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        user_id: r.user_id,
        username: r.username,
        ip_address: r.ip_address,
        request_id: r.request_id,
        data_source: r.data_source,
        table_name: r.table_name,
        data_id: r.data_id,
        access_type: access_type_num(r.access_type.as_deref()),
        sql_digest: r.sql_digest,
        affected_rows: r.affected_rows.map(|v| v as u32),
        latency_ms: r.latency_ms.map(|v| v as u32),
        success: r.success,
        sensitive_level: sensitive_level_num(r.sensitive_level.as_deref()),
        data_masked: r.data_masked,
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn data_access_type_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::data_access_audit_log::AccessType::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn sensitive_level_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::SensitiveLevel::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn access_type_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::data_access_audit_log::AccessType::from_str_name)
        .map(|e| e as i32)
}

fn sensitive_level_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::SensitiveLevel::from_str_name)
        .map(|e| e as i32)
}

/// The old/new value text → the JSON column (valid JSON keeps its
/// shape; free text lands as a JSON string).
pub async fn insert_data_access_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::DataAccessAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_data_access_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id),
        user_id: Set(d.user_id),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Both enum columns ride the proto names on write and read — every
    /// declared member round-trips its own ordinal.
    #[test]
    fn the_enum_columns_round_trip_through_the_proto_names() {
        for v in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 100] {
            let name = data_access_type_name(v).unwrap_or_else(|| panic!("access_type {v}"));
            assert_eq!(access_type_num(Some(&name)), Some(v), "{name}");
        }
        assert_eq!(data_access_type_name(1).as_deref(), Some("SELECT"));
        assert_eq!(data_access_type_name(6).as_deref(), Some("BULK_READ"));
        assert_eq!(data_access_type_name(9).as_deref(), Some("DDL_CREATE"));
        assert_eq!(data_access_type_name(100).as_deref(), Some("OTHER"));
        for v in 1..=4 {
            let name = sensitive_level_name(v).unwrap_or_else(|| panic!("level {v}"));
            assert_eq!(sensitive_level_num(Some(&name)), Some(v), "{name}");
        }
        // Unknown ordinals write NULL; foreign labels read absent.
        for v in [0, -1, 15, 99, 101] {
            assert_eq!(data_access_type_name(v), None, "{v}");
            assert_eq!(sensitive_level_name(v), None, "{v}");
        }
        for junk in ["", "select", "READ", "1"] {
            assert_eq!(access_type_num(Some(junk)), None, "{junk}");
            assert_eq!(sensitive_level_num(Some(junk)), None, "{junk}");
        }
        assert_eq!(access_type_num(None), None);
        assert_eq!(sensitive_level_num(None), None);
    }
}
