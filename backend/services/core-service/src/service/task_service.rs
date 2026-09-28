//! The task service — admin CRUD over sys_tasks plus the
//! start/stop/restart controls as the enable-column flips (the
//! asynq registration/execution history is intentionally not
//! ported), mirroring the reference's task_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Set};
use tonic::{Request, Response, Status};

use crate::data::task_repo;
use crate::service::context::{optional_operator_user_id, tenant_of as request_tenant_of};
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::sys_tasks;
use store::paging::fetch_paged;

use proto::proto::task::service::v1 as taskv1;

fn task_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "PERIODIC" => 0,
        "DELAY" => 1,
        "WAIT_RESULT" => 2,
        _ => return None,
    })
}

fn task_type_name(v: i32) -> Option<&'static str> {
    Some(match v {
        0 => "PERIODIC",
        1 => "DELAY",
        2 => "WAIT_RESULT",
        _ => return None,
    })
}
// ── Task control ─────────────────────────────────────────────────────

pub struct TaskService {
    pub state: Arc<AppState>,
}

/// The reference's registered asynq task types — the two
/// RegisterSubscriber calls of asynq_server.go: pkg/task/backup.go's
/// "backup" and pkg/task/search_reindex.go's "search.reindex"
/// ("search.reindex.all" is declared but never subscribed, so it never
/// registers).
const REGISTERED_TASK_TYPES: [&str; 2] = ["backup", "search.reindex"];

fn task_proto(r: sys_tasks::Model) -> taskv1::Task {
    taskv1::Task {
        id: Some(r.id as u32),
        r#type: r.r#type.as_deref().and_then(task_type_num),
        type_name: r.type_name,
        task_payload: r.task_payload.map(|v| v.to_string()),
        cron_spec: r.cron_spec,
        task_options: r.task_options.and_then(task_option_from_json),
        enable: r.enable,
        remark: r.remark,
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// The reference stores the payload string verbatim as the jsonb column
/// content; a non-JSON payload falls back to a JSON string so the insert
/// can't fail on the column cast.
fn task_payload_json(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw).unwrap_or(serde_json::Value::String(raw.to_string()))
}

// TaskOption ↔ jsonb. The reference marshals the proto message through
// Go's encoding/json: camelCase json tags, {seconds,nanos} objects for
// the well-known duration/timestamp types, absent fields omitted. These
// converters mirror that shape on both directions.

fn duration_json(d: &pbjson_types::Duration) -> serde_json::Value {
    serde_json::json!({ "seconds": d.seconds, "nanos": d.nanos })
}

fn timestamp_json(t: &pbjson_types::Timestamp) -> serde_json::Value {
    serde_json::json!({ "seconds": t.seconds, "nanos": t.nanos })
}

fn duration_from_json(v: &serde_json::Value) -> Option<pbjson_types::Duration> {
    let m = v.as_object()?;
    Some(pbjson_types::Duration {
        seconds: m.get("seconds").and_then(|v| v.as_i64())?,
        nanos: m.get("nanos").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
    })
}

fn timestamp_from_json(v: &serde_json::Value) -> Option<pbjson_types::Timestamp> {
    let m = v.as_object()?;
    Some(pbjson_types::Timestamp {
        seconds: m.get("seconds").and_then(|v| v.as_i64())?,
        nanos: m.get("nanos").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
    })
}

fn task_option_to_json(o: &taskv1::TaskOption) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    if let Some(v) = o.max_retry {
        m.insert("maxRetry".into(), serde_json::Value::from(v));
    }
    if let Some(d) = &o.timeout {
        m.insert("timeout".into(), duration_json(d));
    }
    if let Some(t) = &o.deadline {
        m.insert("deadline".into(), timestamp_json(t));
    }
    if let Some(d) = &o.process_in {
        m.insert("processIn".into(), duration_json(d));
    }
    if let Some(t) = &o.process_at {
        m.insert("processAt".into(), timestamp_json(t));
    }
    if let Some(d) = &o.unique_ttl {
        m.insert("uniqueTtl".into(), duration_json(d));
    }
    if let Some(d) = &o.retention {
        m.insert("retention".into(), duration_json(d));
    }
    if let Some(v) = &o.group {
        m.insert("group".into(), serde_json::Value::from(v.as_str()));
    }
    if let Some(v) = &o.task_id {
        m.insert("taskId".into(), serde_json::Value::from(v.as_str()));
    }
    serde_json::Value::Object(m)
}

fn task_option_from_json(v: serde_json::Value) -> Option<taskv1::TaskOption> {
    let m = v.as_object()?;
    Some(taskv1::TaskOption {
        max_retry: m.get("maxRetry").and_then(|v| v.as_u64()).map(|v| v as u32),
        timeout: m.get("timeout").and_then(duration_from_json),
        deadline: m.get("deadline").and_then(timestamp_from_json),
        process_in: m.get("processIn").and_then(duration_from_json),
        process_at: m.get("processAt").and_then(timestamp_from_json),
        unique_ttl: m.get("uniqueTtl").and_then(duration_from_json),
        retention: m.get("retention").and_then(duration_from_json),
        group: m.get("group").and_then(|v| v.as_str()).map(str::to_string),
        task_id: m.get("taskId").and_then(|v| v.as_str()).map(str::to_string),
    })
}

/// validateTaskFields — type_name is always required; a periodic task
/// also needs a cron_spec. An unset type reads as PERIODIC, like the
/// reference's proto3 getter (0).
fn validate_task_fields(t: &taskv1::Task) -> Result<(), Status> {
    if t.type_name.as_deref().unwrap_or("").is_empty() {
        return Err(bad("type_name is required"));
    }
    if t.r#type.unwrap_or(0) == taskv1::task::Type::Periodic as i32
        && t.cron_spec.as_deref().unwrap_or("").is_empty()
    {
        return Err(bad("cron_spec is required for periodic task"));
    }
    Ok(())
}

/// The Create row shape of the reference's taskRepo: nillable fields from
/// the submitted DTO, created_at always, an explicit id only when the
/// DTO carries one. `created_by` is passed separately (the
/// allow-missing update fallback swaps updated_by into it).
fn task_insert_model(
    data: taskv1::Task,
    created_by: Option<i64>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> sys_tasks::ActiveModel {
    let mut m = sys_tasks::ActiveModel {
        tenant_id: Set(data.tenant_id.map(|v| v as i64)),
        r#type: Set(data.r#type.and_then(task_type_name).map(str::to_string)),
        type_name: Set(data.type_name),
        task_payload: Set(data.task_payload.as_deref().map(task_payload_json)),
        cron_spec: Set(data.cron_spec),
        task_options: Set(data.task_options.as_ref().map(task_option_to_json)),
        enable: Set(data.enable),
        remark: Set(data.remark),
        created_by: Set(created_by),
        created_at: Set(Some(now)),
        ..Default::default()
    };
    if let Some(id) = data.id {
        m.id = Set(id as i64);
    }
    m
}

async fn set_all_tasks(db: &sea_orm::DatabaseConnection, enable: bool) -> Result<u64, Status> {
    use sea_orm::QueryFilter;
    let rows = sys_tasks::Entity::find()
        .filter(sys_tasks::Column::Enable.eq(!enable))
        .all(db)
        .await
        .map_err(db_status)?;
    let mut n = 0u64;
    for row in rows {
        let mut a: sys_tasks::ActiveModel = row.into();
        a.enable = Set(Some(enable));
        a.updated_at = Set(Some(store::now()));
        a.update(db).await.map_err(db_status)?;
        n += 1;
    }
    Ok(n)
}

#[async_trait::async_trait]
impl taskv1::task_service_server::TaskService for TaskService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<taskv1::ListTaskResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_tasks::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(taskv1::ListTaskResponse {
            items: rows.into_iter().map(task_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<taskv1::CountTaskResponse>, Status> {
        use sea_orm::PaginatorTrait as _;
        let total = sys_tasks::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(taskv1::CountTaskResponse { count: total }))
    }

    async fn get(
        &self,
        request: Request<taskv1::GetTaskRequest>,
    ) -> Result<Response<taskv1::Task>, Status> {
        // The type-name lookup is tenant-scoped: a named tenant sees its
        // own row, an authenticated platform caller sees the tenant-0
        // rows, an anonymous context is rejected (the reference's
        // Get-by-TypeName rule; the (tenant_id, type_name) pair is the
        // uniqueness key).
        let tenant_id = request_tenant_of(&request);
        let authed = request.metadata().get("x-user-id").is_some();
        let req = request.into_inner();
        let row = match req.query_by {
            Some(taskv1::get_task_request::QueryBy::Id(id)) => {
                task_repo::tasks_by_id(&self.state.db, id as i64).await?
            }
            Some(taskv1::get_task_request::QueryBy::TypeName(type_name)) => {
                if !authed {
                    return Err(bad("tenant scope required to query task by type name"));
                }
                task_repo::task_by_type_name(&self.state.db, &type_name, tenant_id)
                    .await?
                    .ok_or_else(|| not_found("task"))?
            }
            None => return Err(bad("query_by required")),
        };
        Ok(Response::new(task_proto(row)))
    }

    async fn create(
        &self,
        request: Request<taskv1::CreateTaskRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("invalid parameter"));
        };
        validate_task_fields(&data)?;
        let created_by = data.created_by.map(|v| v as i64);
        task_repo::insert_tasks(
            &self.state.db,
            task_insert_model(data, created_by, store::now()),
        )
        .await?;
        // 偏差：参照仓随后 startTask 把任务注册进 asynq（失败仅记日志）。
        // 本移植没有调度执行体，enable 列即注册状态，调度执行属后续阶段。
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<taskv1::UpdateTaskRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant_id = request_tenant_of(&request);
        let operator_id = optional_operator_user_id(&request);
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("invalid parameter"));
        };
        validate_task_fields(&data)?;
        let id = req.id as i64;

        let existing = sys_tasks::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?;
        if existing.is_none() {
            // allow_missing: the create fallback reuses the submitted
            // data with created_by taking updated_by's value (the
            // reference's swap).
            if req.allow_missing.unwrap_or(false) {
                let created_by = data.updated_by.map(|v| v as i64);
                task_repo::insert_tasks(
                    &self.state.db,
                    task_insert_model(data, created_by, store::now()),
                )
                .await?;
                return Ok(Response::new(pbjson_types::Empty {}));
            }
            // The reference updates through UpdateOneID, which errors on
            // a missing row.
            return Err(not_found("task"));
        }
        let row = existing.unwrap();
        // The reference's tenant predicate applies for named tenants
        // only; a cross-tenant UpdateOneID affects no rows → not found.
        if tenant_id > 0 && row.tenant_id.unwrap_or(0) != tenant_id {
            return Err(not_found("task"));
        }

        let mut a: sys_tasks::ActiveModel = row.into();
        if let Some(v) = data.r#type.and_then(task_type_name) {
            a.r#type = Set(Some(v.to_string()));
        }
        if let Some(v) = data.type_name {
            a.type_name = Set(Some(v));
        }
        if let Some(v) = data.task_payload.as_deref() {
            a.task_payload = Set(Some(task_payload_json(v)));
        }
        if let Some(v) = data.cron_spec {
            a.cron_spec = Set(Some(v));
        }
        if let Some(v) = data.enable {
            a.enable = Set(Some(v));
        }
        if let Some(v) = data.remark {
            a.remark = Set(Some(v));
        }
        if let Some(o) = data.task_options.as_ref() {
            a.task_options = Set(Some(task_option_to_json(o)));
        }
        a.updated_at = Set(Some(store::now()));
        // updated_by is forced from the caller identity, the client
        // value is ignored (the reference's viewer rule).
        if operator_id > 0 {
            a.updated_by = Set(Some(operator_id));
        }
        task_repo::update_tasks(&self.state.db, a).await?;
        // 偏差：参照仓更新后先移除旧的 asynq 注册项，再按新的 enable 状
        // 态重启。本移植没有调度执行体，enable 列即注册状态，调度执行属
        // 后续阶段。
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<taskv1::DeleteTaskRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant_id = request_tenant_of(&request);
        let req = request.into_inner();
        let Some(taskv1::delete_task_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let id = id as i64;
        // The reference gets the row first (a missing id 404s there),
        // then deletes, then stops the scheduler registration.
        task_repo::tasks_by_id(&self.state.db, id).await?;
        task_repo::delete_task_scoped(&self.state.db, id, (tenant_id > 0).then_some(tenant_id))
            .await?;
        // 偏差：参照仓随后 stopTask 移除 asynq 注册项；本移植无调度执行
        // 体，无需额外动作。
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn list_task_type_name(
        &self,
        _request: Request<pbjson_types::Empty>,
    ) -> Result<Response<taskv1::ListTaskTypeNameResponse>, Status> {
        // The reference reads the asynq scheduler's registered task
        // types; this port freezes the same set (see
        // REGISTERED_TASK_TYPES).
        Ok(Response::new(taskv1::ListTaskTypeNameResponse {
            type_names: REGISTERED_TASK_TYPES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }))
    }

    async fn start_all_task(
        &self,
        _request: Request<pbjson_types::Empty>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        set_all_tasks(&self.state.db, true).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn stop_all_task(
        &self,
        _request: Request<pbjson_types::Empty>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        set_all_tasks(&self.state.db, false).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn restart_all_task(
        &self,
        _request: Request<pbjson_types::Empty>,
    ) -> Result<Response<taskv1::RestartAllTaskResponse>, Status> {
        // Restart = stop then start (the enable flip both ways).
        set_all_tasks(&self.state.db, false).await?;
        let n = set_all_tasks(&self.state.db, true).await?;
        Ok(Response::new(taskv1::RestartAllTaskResponse {
            count: n as i32,
        }))
    }

    async fn control_task(
        &self,
        request: Request<taskv1::ControlTaskRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // The lookup follows the reference's Get-by-TypeName tenant
        // scoping; the enable flip stands in for the scheduler
        // registration (the execution body is a later phase).
        let tenant_id = request_tenant_of(&request);
        let authed = request.metadata().get("x-user-id").is_some();
        let req = request.into_inner();
        if !authed {
            return Err(bad("tenant scope required to query task by type name"));
        }
        let row = task_repo::task_by_type_name(&self.state.db, &req.type_name, tenant_id)
            .await?
            .ok_or_else(|| not_found("task"))?;
        let mut a: sys_tasks::ActiveModel = row.into();
        a.updated_at = Set(Some(store::now()));
        // The control switch follows the reference's ControlTask:
        // Start=0, Stop=1, Restart=2 (restart = stop then start, so the
        // end state is enabled). The reference's startTask/stopTask also
        // error on an already-disabled task — an asynq registration
        // guard this port doesn't reproduce.
        match taskv1::control_task_request::ControlType::try_from(req.control_type) {
            Ok(taskv1::control_task_request::ControlType::Start) => a.enable = Set(Some(true)),
            Ok(taskv1::control_task_request::ControlType::Stop) => a.enable = Set(Some(false)),
            Ok(taskv1::control_task_request::ControlType::Restart) => a.enable = Set(Some(true)),
            Err(_) => {
                let value = req.control_type;
                return Err(bad(&format!("unknown control type: {value}")));
            }
        }
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn list_task_executions(
        &self,
        _request: Request<taskv1::ListTaskExecutionsRequest>,
    ) -> Result<Response<taskv1::ListTaskExecutionsResponse>, Status> {
        // 偏差：参照仓经 asynq Inspector 聚合 default 队列的四态执行实
        // 例（completed/archived/active/pending）。本移植没有执行历史存
        // 储（调度执行属后续阶段），返回空列表。
        Ok(Response::new(taskv1::ListTaskExecutionsResponse {
            items: Vec::new(),
            total: 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_type_round_trips_and_rejects_unknowns() {
        for (name, num) in [("PERIODIC", 0), ("DELAY", 1), ("WAIT_RESULT", 2)] {
            assert_eq!(task_type_num(name), Some(num), "{name}");
            assert_eq!(task_type_name(num), Some(name), "{num}");
        }
        for num in [3, -1, 100] {
            assert_eq!(task_type_name(num), None, "{num}");
        }
        for name in ["CRON", "periodic", ""] {
            assert_eq!(task_type_num(name), None, "{name}");
        }
    }
}
