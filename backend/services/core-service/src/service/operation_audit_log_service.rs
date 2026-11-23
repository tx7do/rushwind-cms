//! The operation-audit-log service (operation_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

// The varchar enum columns ride the proto enum value names — the golden
// schema stores what the reference's ent enums define, and the
// reference's EnumTypeConverter pairs the two directions. The action
// column is the OPERATION action family (CREATE / UPDATE / DELETE /
// …), not the login family the pre-fix writer stored.

fn action_type_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::operation_audit_log::ActionType::try_from(v)
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

fn action_type_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::operation_audit_log::ActionType::from_str_name)
        .map(|e| e as i32)
}

fn sensitive_level_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::SensitiveLevel::from_str_name)
        .map(|e| e as i32)
}

fn operation_audit_proto(
    r: store::entities::sys_operation_audit_logs::Model,
) -> auditv1::OperationAuditLog {
    auditv1::OperationAuditLog {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        user_id: r.user_id,
        username: r.username,
        resource_type: r.resource_type,
        resource_id: r.resource_id,
        action: action_type_num(r.action.as_deref()),
        sensitive_level: sensitive_level_num(r.sensitive_level.as_deref()),
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
        tenant_id: Set(d.tenant_id),
        user_id: Set(d.user_id),
        username: Set(d.username),
        resource_type: Set(d.resource_type),
        resource_id: Set(d.resource_id),
        action: Set(d.action.and_then(action_type_name)),
        sensitive_level: Set(d.sensitive_level.and_then(sensitive_level_name)),
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
            total: total as i64,
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
        let row = store::entities::sys_operation_audit_logs::Entity::find_by_id(id)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The action column is the operation family: every declared member
    /// (including the OTHER=100 outlier) writes its own proto name and
    /// reads back its own ordinal.
    #[test]
    fn the_action_column_round_trips_through_the_operation_names() {
        for v in [1, 2, 3, 4, 5, 6, 7, 8, 100] {
            let name = action_type_name(v).unwrap_or_else(|| panic!("action {v}"));
            assert_eq!(action_type_num(Some(&name)), Some(v), "{name}");
        }
        // The names are the proto spellings — a CREATE row never stores
        // the login family's "LOGIN" the pre-fix writer produced.
        assert_eq!(action_type_name(1).as_deref(), Some("CREATE"));
        assert_eq!(action_type_name(2).as_deref(), Some("UPDATE"));
        assert_eq!(action_type_name(3).as_deref(), Some("DELETE"));
        assert_eq!(action_type_name(100).as_deref(), Some("OTHER"));
        // Unknown ordinals write NULL instead of a guessed family's name.
        for v in [0, -1, 9, 99, 101] {
            assert_eq!(action_type_name(v), None, "{v}");
        }
        // Foreign labels read as absent.
        for junk in ["", "LOGIN", "LOGOUT", "REFRESH", "login"] {
            assert_eq!(action_type_num(Some(junk)), None, "{junk}");
        }
        assert_eq!(action_type_num(None), None);
    }

    #[test]
    fn the_sensitive_level_column_round_trips_through_the_shared_names() {
        for v in 1..=4 {
            let name = sensitive_level_name(v).unwrap_or_else(|| panic!("level {v}"));
            assert_eq!(sensitive_level_num(Some(&name)), Some(v), "{name}");
        }
        assert_eq!(sensitive_level_name(1).as_deref(), Some("PUBLIC"));
        assert_eq!(sensitive_level_name(4).as_deref(), Some("SECRET"));
        // The pre-fix writer stored ordinal strings — they were never
        // reference data and read absent now.
        for junk in ["", "1", "public", "TOP_SECRET"] {
            assert_eq!(sensitive_level_num(Some(junk)), None, "{junk}");
        }
        for v in [0, -1, 5] {
            assert_eq!(sensitive_level_name(v), None, "{v}");
        }
        assert_eq!(sensitive_level_num(None), None);
    }
}
