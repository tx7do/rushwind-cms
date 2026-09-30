//! The PolicyEvaluationLogService — the permission engine's decision
//! audit face (the reference's policy_evaluation_log_service.go): the
//! paged listing, the by-id fetch and the append.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use permissionv1::policy_evaluation_log_service_server::PolicyEvaluationLogService;
use proto::proto::permission::service::v1 as permv1;
use proto::proto::permission::service::v1 as permissionv1;

use crate::data::policy_evaluation_log_repo as repo;
use crate::state::{bad, ts_to_proto, AppState};

pub struct PolicyEvaluationLogServiceImpl {
    pub state: Arc<AppState>,
}

/// The entity row → the proto message.
fn policy_evaluation_log_proto(
    r: store::entities::sys_policy_evaluation_logs::Model,
) -> permv1::PolicyEvaluationLog {
    permv1::PolicyEvaluationLog {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        user_id: Some(r.user_id as u32),
        membership_id: Some(r.membership_id as u32),
        permission_id: Some(r.permission_id as u32),
        policy_id: r.policy_id.map(|v| v as u32),
        request_path: r.request_path,
        request_method: r.request_method,
        result: Some(r.result),
        effect_details: r.effect_details,
        scope_sql: r.scope_sql,
        ip_address: r.ip_address,
        trace_id: r.trace_id,
        evaluation_context: r.evaluation_context,
        log_hash: r.log_hash,
        signature: r.signature,
        created_at: r.created_at.and_then(ts_to_proto),
    }
}

#[async_trait::async_trait]
impl PolicyEvaluationLogService for PolicyEvaluationLogServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permv1::ListPolicyEvaluationLogResponse>, Status> {
        let (rows, total) = repo::list(&self.state.db, &request.into_inner()).await?;
        Ok(Response::new(permv1::ListPolicyEvaluationLogResponse {
            items: rows.into_iter().map(policy_evaluation_log_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<permv1::GetPolicyEvaluationLogRequest>,
    ) -> Result<Response<permv1::PolicyEvaluationLog>, Status> {
        let req = request.into_inner();
        let Some(permv1::get_policy_evaluation_log_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(policy_evaluation_log_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permv1::CreatePolicyEvaluationLogRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let Some(data) = request.into_inner().data else {
            return Err(bad("invalid parameter"));
        };
        repo::insert(&self.state.db, data).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
