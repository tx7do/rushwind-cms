//! policyEvaluationLogRepo — the data layer of the policy-evaluation
//! log domain, mirroring the reference's `policy_evaluation_log_repo.go`:
//! the paged listing, the by-id fetch and the append-only insert (the
//! audit write face of the permission engine).

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tonic::Status;

use proto::proto::pagination::PagingRequest;
use proto::proto::permission::service::v1 as permv1;
use store::entities::sys_policy_evaluation_logs as entity;
use store::paging::fetch_paged;

use crate::state::{db_status, StatusResult};

/// The paged listing (the reference's List).
pub async fn list(
    db: &DatabaseConnection,
    req: &PagingRequest,
) -> StatusResult<(Vec<entity::Model>, u64)> {
    fetch_paged(db, entity::Entity::find(), req)
        .await
        .map_err(|e| Status::internal(e.message))
}

/// The by-id fetch (the reference's Get with the Id query).
pub async fn by_id(db: &DatabaseConnection, id: i64) -> StatusResult<entity::Model> {
    entity::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| crate::state::not_found_status("policy evaluation log"))
}

/// The append (the reference's Create) — the log row lands verbatim,
/// created_at stamped now.
pub async fn insert(
    db: &DatabaseConnection,
    data: permv1::PolicyEvaluationLog,
) -> StatusResult<()> {
    entity::ActiveModel {
        tenant_id: Set(data.tenant_id.map(|v| v as i64)),
        user_id: Set(data.user_id.unwrap_or(0) as i64),
        membership_id: Set(data.membership_id.unwrap_or(0) as i64),
        permission_id: Set(data.permission_id.unwrap_or(0) as i64),
        policy_id: Set(data.policy_id.map(|v| v as i64)),
        request_path: Set(data.request_path),
        request_method: Set(data.request_method),
        result: Set(data.result.unwrap_or(false)),
        effect_details: Set(data.effect_details),
        scope_sql: Set(data.scope_sql),
        ip_address: Set(data.ip_address),
        trace_id: Set(data.trace_id),
        evaluation_context: Set(data.evaluation_context),
        log_hash: Set(data.log_hash),
        signature: Set(data.signature),
        created_at: Set(Some(store::now())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_status)?;
    Ok(())
}
