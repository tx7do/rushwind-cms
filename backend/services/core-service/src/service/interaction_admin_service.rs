//! The interaction-admin service — the platform-admin purge/reset
//! face over the interaction ledgers and counters, with the audit
//! row per purge (the reference's interaction admin face).

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::interaction_repo;
use crate::service::context::require_admin_operator;
use crate::service::interaction_service::metric_num;
use crate::service::interaction_service::InteractionService;
use crate::state::{bad, db_status, AppState};
use store::entities::{comment_likes, interaction_counters, post_likes, post_watches};

use proto::proto::audit::service::v1 as auditv1;
use proto::proto::interaction::service::v1 as interactionv1;

// ── Interaction admin (the清算 face) ─────────────────────────────────

/// The interaction_counters target_type column values (POST=1 /
/// COMMENT=2, the proto's enum numbers).
fn target_type_num(target_type: i32) -> Option<i16> {
    match target_type {
        1 => Some(1),
        2 => Some(2),
        _ => None,
    }
}

/// The admin清算 gate (the reference's requireAdminOperator): the
/// operator identity must ride the metadata (401 otherwise) and the
/// operator must sit in the platform/system context — tenant 0 in the
/// metadata bag (403 otherwise).
/// The purge/reset audit row (the reference's writeAudit: one
/// OperationAuditLog with the DELETE action; a failed audit write never
/// fails the RPC).
async fn write_purge_audit(
    db: &sea_orm::DatabaseConnection,
    operator_tenant_id: i64,
    operator_user_id: i64,
    resource_type: &str,
    resource_id: &str,
    success: bool,
) {
    let entry = auditv1::OperationAuditLog {
        tenant_id: Some(operator_tenant_id as u32),
        user_id: Some(operator_user_id as u32),
        resource_type: Some(resource_type.to_string()),
        resource_id: Some(resource_id.to_string()),
        action: Some(auditv1::operation_audit_log::ActionType::Delete as i32),
        success: Some(success),
        ..Default::default()
    };
    let _ = crate::service::operation_audit_log_service::insert_operation_audit(db, entry).await;
}

/// The target-wide ledger wipe + counter zero-out in one transaction
/// (the reference's PurgeTargetInteractions repo tail).
async fn purge_target_interactions_inner(
    db: &sea_orm::DatabaseConnection,
    target_type: i16,
    target_id: i64,
) -> Result<u32, Status> {
    let txn = db.begin().await.map_err(db_status)?;
    // The like ledgers follow the target family; the watches are
    // post-only but deleted unconditionally like the reference.
    let mut affected: u32 = match target_type {
        1 => interaction_repo::delete_post_likes_of_post(&txn, target_id).await?,
        _ => interaction_repo::delete_comment_likes_of_comment(&txn, target_id).await?,
    } as u32;
    affected += interaction_repo::delete_post_watches_of_post(&txn, target_id).await? as u32;
    // The counters zero out to row deletion: LIKE always, WATCH for
    // whatever row exists on this target.
    interaction_counters::Entity::delete_many()
        .filter(interaction_counters::Column::TargetType.eq(target_type))
        .filter(interaction_counters::Column::TargetId.eq(target_id))
        .filter(interaction_counters::Column::Metric.is_in([1i16, 2]))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    txn.commit().await.map_err(db_status)?;
    Ok(affected)
}

/// The user-wide ledger wipe (the reference's PurgeUserInteractions):
/// every ledger row of the user goes and each removed row rolls its
/// target's counter back one step, all inside one transaction (the
/// reference splits it into 200-row short transactions; the port keeps
/// the single-transaction semantics).
async fn purge_user_interactions_inner(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
) -> Result<u32, Status> {
    let txn = db.begin().await.map_err(db_status)?;
    let mut affected: u32 = 0;
    // post likes (LIKE metric, target=post)
    for post_id in interaction_repo::post_like_targets_of_user(&txn, user_id).await? {
        interaction_repo::adjust_counter(&txn, 0, 1, post_id, 1, -1).await?;
        affected += 1;
    }
    post_likes::Entity::delete_many()
        .filter(post_likes::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    // comment likes (LIKE metric, target=comment)
    for comment_id in interaction_repo::comment_like_targets_of_user(&txn, user_id).await? {
        interaction_repo::adjust_counter(&txn, 0, 2, comment_id, 1, -1).await?;
        affected += 1;
    }
    comment_likes::Entity::delete_many()
        .filter(comment_likes::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    // post watches (WATCH metric, target=post)
    for post_id in interaction_repo::post_watch_targets_of_user(&txn, user_id).await? {
        interaction_repo::adjust_counter(&txn, 0, 1, post_id, 2, -1).await?;
        affected += 1;
    }
    post_watches::Entity::delete_many()
        .filter(post_watches::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    txn.commit().await.map_err(db_status)?;
    Ok(affected)
}

/// The counter recompute (the reference's ResetCounter): count the
/// ledger truth for (target, metric), then push the counter row to it
/// inside one transaction.
async fn reset_counter_inner(
    db: &sea_orm::DatabaseConnection,
    target_type: i32,
    target_id: i64,
    metric: i32,
) -> Result<i64, Status> {
    // The ledger recount per metric family; WATCH counts the post
    // watches whatever the target type rides in on (like the reference).
    let recount: i64 = match (metric, target_type) {
        (1, 1) => interaction_repo::count_ledger(db, 1, 1, target_id).await? as i64,
        (1, 2) => interaction_repo::count_ledger(db, 1, 2, target_id).await? as i64,
        (1, _) => return Err(bad("invalid target type")),
        (2, _) => interaction_repo::count_ledger(db, 2, 1, target_id).await? as i64,
        _ => return Err(bad("invalid metric")),
    };
    let row_type = target_type_num(target_type).unwrap_or(1);
    let metric = metric_num(metric);
    let txn = db.begin().await.map_err(db_status)?;
    let delta = recount - interaction_repo::counter_of(&txn, row_type, target_id, metric).await?;
    if delta != 0 {
        interaction_repo::adjust_counter(&txn, 0, row_type, target_id, metric, delta).await?;
    }
    txn.commit().await.map_err(db_status)?;
    Ok(recount)
}

pub struct InteractionAdminService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl interactionv1::interaction_admin_service_server::InteractionAdminService
    for InteractionAdminService
{
    async fn purge_target_interactions(
        &self,
        request: Request<interactionv1::PurgeTargetInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeTargetInteractionsResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let target_type =
            target_type_num(req.target_type).ok_or_else(|| bad("invalid target type"))?;
        let target_id = req.target_id as i64;

        let outcome = purge_target_interactions_inner(&self.state.db, target_type, target_id).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_counter",
            &target_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(
            interactionv1::PurgeTargetInteractionsResponse {
                affected_rows: outcome?,
            },
        ))
    }

    async fn purge_user_interactions(
        &self,
        request: Request<interactionv1::PurgeUserInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeUserInteractionsResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let user_id = req.user_id as i64;

        let outcome = purge_user_interactions_inner(&self.state.db, user_id).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_user_ledger",
            &user_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(
            interactionv1::PurgeUserInteractionsResponse {
                affected_rows: outcome?,
            },
        ))
    }

    async fn reset_counter(
        &self,
        request: Request<interactionv1::ResetCounterRequest>,
    ) -> Result<Response<interactionv1::ResetCounterResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let target_id = req.target_id as i64;

        let outcome =
            reset_counter_inner(&self.state.db, req.target_type, target_id, req.metric).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_counter_reset",
            &target_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(interactionv1::ResetCounterResponse {
            recount: outcome?,
        }))
    }
}

impl InteractionAdminService {
    /// The admin counters read — shared with the public face.
    pub async fn get_counts(
        &self,
        request: Request<interactionv1::GetCountsRequest>,
    ) -> Result<Response<interactionv1::GetCountsResponse>, Status> {
        <InteractionService as interactionv1::interaction_service_server::InteractionService>::get_counts(
            &InteractionService {
                state: Arc::clone(&self.state),
            },
            request,
        )
        .await
    }
}
