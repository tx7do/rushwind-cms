//! The login-audit-log service (login_audit_log_service.go).

use sea_orm::EntityTrait as _;
use sea_orm::Set;
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;

// The varchar enum columns ride the proto enum value names — the golden
// schema stores what the reference's ent enums define (the same names
// the proto carries), and the reference's EnumTypeConverter pairs the
// two directions. Write: the ordinal → its name. Read: the stored name
// → its ordinal; anything outside the table (a legacy numeric string, a
// foreign label) reads as absent.

fn action_type_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::login_audit_log::ActionType::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn status_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::login_audit_log::Status::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn login_method_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::login_audit_log::LoginMethod::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn risk_level_name(v: i32) -> Option<String> {
    // The zero ordinal is the UNSPECIFIED member — the reference's
    // ent enum never declares it, so it writes NULL like any
    // unknown value.
    if v == 0 {
        return None;
    }
    auditv1::login_audit_log::RiskLevel::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn action_type_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::login_audit_log::ActionType::from_str_name)
        .map(|e| e as i32)
}

fn status_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::login_audit_log::Status::from_str_name)
        .map(|e| e as i32)
}

fn login_method_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::login_audit_log::LoginMethod::from_str_name)
        .map(|e| e as i32)
}

fn risk_level_num(name: Option<&str>) -> Option<i32> {
    name.and_then(auditv1::login_audit_log::RiskLevel::from_str_name)
        .map(|e| e as i32)
}

fn login_audit_proto(r: store::entities::sys_login_audit_logs::Model) -> auditv1::LoginAuditLog {
    auditv1::LoginAuditLog {
        id: Some(r.id),
        tenant_id: r.tenant_id,
        user_id: r.user_id,
        username: r.username,
        ip_address: r.ip_address,
        session_id: r.session_id,
        request_id: r.request_id,
        trace_id: r.trace_id,
        action_type: action_type_num(r.action_type.as_deref()),
        status: status_num(r.status.as_deref()),
        login_method: login_method_num(r.login_method.as_deref()),
        failure_reason: r.failure_reason,
        mfa_status: r.mfa_status,
        risk_score: r.risk_score.map(|v| v as u32),
        risk_level: risk_level_num(r.risk_level.as_deref()),
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// Inserts one login-audit row (the middleware assembles the fields).
pub async fn insert_login_audit(
    db: &sea_orm::DatabaseConnection,
    d: auditv1::LoginAuditLog,
) -> Result<(), Status> {
    use sea_orm::ActiveModelTrait;
    store::entities::sys_login_audit_logs::ActiveModel {
        tenant_id: Set(d.tenant_id),
        user_id: Set(d.user_id),
        username: Set(d.username),
        ip_address: Set(d.ip_address),
        session_id: Set(d.session_id),
        request_id: Set(d.request_id),
        trace_id: Set(d.trace_id),
        action_type: Set(d.action_type.and_then(action_type_name)),
        status: Set(d.status.and_then(status_name)),
        login_method: Set(d.login_method.and_then(login_method_name)),
        failure_reason: Set(d.failure_reason),
        mfa_status: Set(d.mfa_status),
        risk_score: Set(d.risk_score.map(|v| v as i64)),
        risk_level: Set(d.risk_level.and_then(risk_level_name)),
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
            total: total as i64,
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
        let row = store::entities::sys_login_audit_logs::Entity::find_by_id(id)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every declared member writes its own proto enum value name and
    /// reads back its own ordinal — the golden schema stores the names
    /// the reference's ent enums define, and the reference's
    /// EnumTypeConverter pairs the two directions.
    #[test]
    fn the_varchar_enums_round_trip_through_the_proto_names() {
        for v in 1..=5 {
            let name = action_type_name(v).unwrap_or_else(|| panic!("action_type {v}"));
            assert_eq!(action_type_num(Some(&name)), Some(v), "{name}");
        }
        for v in 1..=4 {
            let name = status_name(v).unwrap_or_else(|| panic!("status {v}"));
            assert_eq!(status_num(Some(&name)), Some(v), "{name}");
        }
        for v in 1..=6 {
            let name = login_method_name(v).unwrap_or_else(|| panic!("login_method {v}"));
            assert_eq!(login_method_num(Some(&name)), Some(v), "{name}");
        }
        for v in 1..=3 {
            let name = risk_level_name(v).unwrap_or_else(|| panic!("risk_level {v}"));
            assert_eq!(risk_level_num(Some(&name)), Some(v), "{name}");
        }
        // The names are the proto spellings themselves.
        assert_eq!(action_type_name(3).as_deref(), Some("SESSION_EXPIRED"));
        assert_eq!(status_name(2).as_deref(), Some("FAILED"));
        assert_eq!(login_method_name(1).as_deref(), Some("PASSWORD"));
        assert_eq!(risk_level_name(3).as_deref(), Some("HIGH"));
    }

    #[test]
    fn write_side_unknowns_collapse_onto_null_not_a_guessed_name() {
        // The zero ordinal is UNSPECIFIED — absent, never a stored label.
        for v in [0, -1, 7, 99] {
            assert_eq!(action_type_name(v), None, "{v}");
            assert_eq!(status_name(v), None, "{v}");
            assert_eq!(login_method_name(v), None, "{v}");
            assert_eq!(risk_level_name(v), None, "{v}");
        }
        // The declared boundaries stay inside: 5 = PASSWORD_RESET,
        // 4 = LOCKED, 6 = FIDO2, 3 = HIGH.
        assert!(action_type_name(5).is_some());
        assert!(status_name(4).is_some());
        assert!(login_method_name(6).is_some());
        assert!(risk_level_name(3).is_some());
    }

    #[test]
    fn read_side_unknown_varchars_read_as_absent() {
        // A foreign label, a lowercase spelling, or the empty column.
        for junk in ["", "login", "LOG-OUT", "REFRESH", "weird"] {
            assert_eq!(action_type_num(Some(junk)), None, "{junk}");
        }
        for junk in ["", "success", "OK", "1"] {
            assert_eq!(status_num(Some(junk)), None, "{junk}");
        }
        // The legacy numeric strings the pre-fix writer stored ("0" for a
        // password grant) were never reference data — they read absent.
        assert_eq!(login_method_num(Some("0")), None);
        // An absent column stays absent.
        for read in [
            action_type_num,
            status_num,
            login_method_num,
            risk_level_num,
        ] {
            assert_eq!(read(None), None);
        }
    }
}
