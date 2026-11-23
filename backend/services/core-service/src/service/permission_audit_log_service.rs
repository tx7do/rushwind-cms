//! The permission-audit-log service (permission_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

use crate::service::audit_support::audit_impl;

// The action column rides the proto enum value names on both sides —
// the golden schema stores what the reference's ent enums define, and
// the reference's EnumTypeConverter pairs the two directions.

fn permission_audit_proto(
    r: store::entities::sys_permission_audit_logs::Model,
) -> auditv1::PermissionAuditLog {
    auditv1::PermissionAuditLog {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        operator_id: r.operator_id,
        target_type: r.target_type,
        target_id: r.target_id,
        action: permission_action_num(r.action.as_deref()),
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
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::permission_audit_log::ActionType::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn permission_action_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::permission_audit_log::ActionType::from_str_name)
        .map(|e| e as i32)
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
        tenant_id: Set(d.tenant_id),
        operator_id: Set(d.operator_id),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The write side already stored the proto names; the read side
    /// parses them back — every declared member round-trips its own
    /// ordinal.
    #[test]
    fn the_action_column_round_trips_through_the_proto_names() {
        for v in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 100] {
            let name = permission_action_name(v).unwrap_or_else(|| panic!("action {v}"));
            assert_eq!(permission_action_num(Some(&name)), Some(v), "{name}");
        }
        assert_eq!(permission_action_name(1).as_deref(), Some("GRANT"));
        assert_eq!(permission_action_name(5).as_deref(), Some("CREATE"));
        assert_eq!(permission_action_name(9).as_deref(), Some("BULK_GRANT"));
        assert_eq!(permission_action_name(100).as_deref(), Some("OTHER"));
        // Unknown ordinals and foreign labels stay out of the table.
        for v in [0, -1, 15, 99, 101] {
            assert_eq!(permission_action_name(v), None, "{v}");
        }
        for junk in ["", "LOGIN", "grant", "1"] {
            assert_eq!(permission_action_num(Some(junk)), None, "{junk}");
        }
        assert_eq!(permission_action_num(None), None);
    }
}
