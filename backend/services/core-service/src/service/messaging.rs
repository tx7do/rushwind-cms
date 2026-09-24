//! The internal-message write flow (send → recipient rows → the
//! notification the SSE hub pushes), the recipient inbox face, and the
//! task control face (start/stop/restart as the enable-column flips).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::messaging_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{internal_message_recipients, internal_messages, sys_tasks};
use store::paging::fetch_paged;

use proto::proto::internal_message::service::v1 as imv1;
use proto::proto::task::service::v1 as taskv1;

fn operator_of<T>(request: &tonic::Request<T>) -> Result<i64, Status> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| Status::unauthenticated("user identity required"))
}

// ── InternalMessage write flow ───────────────────────────────────────

fn message_proto(r: internal_messages::Model) -> imv1::InternalMessage {
    imv1::InternalMessage {
        id: Some(r.id as u32),
        title: r.title,
        content: r.content,
        sender_id: Some(r.sender_id as u32),
        category_id: r.category_id.map(|v| v as u32),
        status: r.status.map(|_| 1),
        r#type: r.r#type.map(|_| 1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn recipient_proto(r: internal_message_recipients::Model) -> imv1::InternalMessageRecipient {
    imv1::InternalMessageRecipient {
        id: Some(r.id as u32),
        message_id: r.message_id.map(|v| v as u32),
        recipient_user_id: r.recipient_user_id.map(|v| v as u32),
        status: r.status.as_deref().map(|s| if s == "READ" { 2 } else { 1 }),
        received_at: r.received_at.and_then(ts_to_proto),
        read_at: r.read_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// The send result the BFF SSE fan-out consumes: (recipient, payload)
/// pairs — payload = the recipient protojson body of the notification
/// event.
pub struct SentNotification {
    pub recipient_user_id: u32,
    pub payload_json: String,
}

pub struct InternalMessageWriteServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_service_server::InternalMessageService
    for InternalMessageWriteServiceImpl
{
    async fn list_message(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_messages::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageResponse {
            items: rows.into_iter().map(message_proto).collect(),
            total,
        }))
    }

    async fn get_message(
        &self,
        request: Request<imv1::GetInternalMessageRequest>,
    ) -> Result<Response<imv1::InternalMessage>, Status> {
        let req = request.into_inner();
        let Some(imv1::get_internal_message_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::internal_messages_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(message_proto(row)))
    }

    async fn send_message(
        &self,
        request: Request<imv1::SendMessageRequest>,
    ) -> Result<Response<imv1::SendMessageResponse>, Status> {
        let sender_id = operator_of(&request)?;
        let req = request.into_inner();
        let target_all = req.target_all.unwrap_or(false);
        if req.target_user_ids.is_empty() && !target_all {
            return Err(bad("recipients required"));
        }
        let now = store::now();
        // The message row.
        let msg = internal_messages::ActiveModel {
            title: Set(Some(req.title.clone().unwrap_or_default())),
            content: Set(Some(req.content.clone())),
            sender_id: Set(sender_id),
            category_id: Set(req.category_id.map(|v| v as i64)),
            status: Set(Some("SENT".to_string())),
            r#type: Set(Some("NOTIFICATION".to_string())),
            tenant_id: Set(Some(0)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;

        // Recipient set: explicit targets, or every user (broadcast).
        let targets: Vec<i64> = if target_all {
            use sea_orm::QueryFilter;
            store::entities::sys_users::Entity::find()
                .filter(store::entities::sys_users::Column::Status.eq("NORMAL"))
                .all(&self.state.db)
                .await
                .map_err(db_status)?
                .into_iter()
                .map(|u| u.id)
                .collect()
        } else {
            req.target_user_ids.iter().map(|v| *v as i64).collect()
        };
        // The recipient rows + the SSE payload list (returned via the
        // response metadata so the BFF hub can fan out without a
        // re-query).
        let mut payload = Vec::new();
        for uid in targets {
            let row = internal_message_recipients::ActiveModel {
                message_id: Set(Some(msg.id)),
                recipient_user_id: Set(Some(uid)),
                status: Set(Some("RECEIVED".to_string())),
                tenant_id: Set(Some(0)),
                received_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
            let proto = recipient_proto(row);
            payload.push(format!("{}:{}", uid, serde_json::to_string(&serde_json::json!({
                "id": proto.id,
                "messageId": proto.message_id,
                "recipientUserId": proto.recipient_user_id,
                "status": "RECEIVED",
                "title": req.title,
                "content": req.content,
                "receivedAt": proto.received_at.map(|ts| serde_json::json!({"seconds": ts.seconds, "nanos": ts.nanos})),
            })).unwrap_or_default()));
        }

        // The payload rides a response metadata key (JSON array) for the
        // BFF SSE fan-out.
        let mut resp = Response::new(imv1::SendMessageResponse {
            message_id: msg.id as u32,
        });
        resp.metadata_mut().insert(
            "x-sse-payload",
            format!("[{}]", payload.join(","))
                .parse()
                .map_err(|_| Status::internal("payload header"))?,
        );
        Ok(resp)
    }

    async fn revoke_message(
        &self,
        request: Request<imv1::RevokeMessageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = internal_messages::Entity::find_by_id(req.message_id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("message"))?;
        let mut a: internal_messages::ActiveModel = row.into();
        a.status = Set(Some("REVOKED".to_string()));
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Recipient inbox face ─────────────────────────────────────────────

pub struct RecipientWriteServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_recipient_service_server::InternalMessageRecipientService
    for RecipientWriteServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageRecipientResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_message_recipients::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageRecipientResponse {
            items: rows.into_iter().map(recipient_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<imv1::GetInternalMessageRecipientRequest>,
    ) -> Result<Response<imv1::InternalMessageRecipient>, Status> {
        let req = request.into_inner();
        let Some(imv1::get_internal_message_recipient_request::QueryBy::Id(id)) = req.query_by
        else {
            return Err(bad("query_by required"));
        };
        let row = repo::internal_message_recipients_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(recipient_proto(row)))
    }

    async fn list_user_inbox(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListUserInboxResponse>, Status> {
        let uid = operator_of(&request)?;
        let req = request.into_inner();
        let _ = req;
        let rows = internal_message_recipients::Entity::find()
            .filter(internal_message_recipients::Column::RecipientUserId.eq(uid))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        let total = rows.len() as u64;
        Ok(Response::new(imv1::ListUserInboxResponse {
            items: rows.into_iter().map(recipient_proto).collect(),
            total,
        }))
    }

    async fn mark_notification_as_read(
        &self,
        request: Request<imv1::MarkNotificationAsReadRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let uid = operator_of(&request)?;
        let req = request.into_inner();
        let _ = req.user_id; // operator wins over the echoed field
        for rid in req.recipient_ids {
            let row = internal_message_recipients::Entity::find_by_id(rid as i64)
                .one(&self.state.db)
                .await
                .map_err(db_status)?;
            if let Some(row) = row {
                if row.recipient_user_id.unwrap_or(0) != uid {
                    continue;
                }
                let mut a: internal_message_recipients::ActiveModel = row.into();
                a.status = Set(Some("READ".to_string()));
                a.read_at = Set(Some(store::now()));
                a.update(&self.state.db).await.map_err(db_status)?;
            }
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete_notification_from_inbox(
        &self,
        request: Request<imv1::DeleteNotificationFromInboxRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let uid = operator_of(&request)?;
        let req = request.into_inner();
        internal_message_recipients::Entity::delete_many()
            .filter(
                sea_orm::Condition::all()
                    .add(
                        internal_message_recipients::Column::Id
                            .is_in(req.recipient_ids.into_iter().map(|v| v as i64)),
                    )
                    .add(internal_message_recipients::Column::RecipientUserId.eq(uid)),
            )
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Task control ─────────────────────────────────────────────────────

pub struct TaskControlServiceImpl {
    pub state: Arc<AppState>,
}

fn task_proto(r: sys_tasks::Model) -> taskv1::Task {
    taskv1::Task {
        id: Some(r.id as u32),
        r#type: r.r#type.map(|_| 1),
        type_name: r.type_name,
        cron_spec: r.cron_spec,
        enable: r.enable,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
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
impl taskv1::task_service_server::TaskService for TaskControlServiceImpl {
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
        let req = request.into_inner();
        // Control by type name (the reference dispatches tasks by type);
        // 1 = START, others = STOP.
        let enable = req.control_type == 1;
        let row = sys_tasks::Entity::find()
            .filter(sys_tasks::Column::TypeName.eq(req.type_name.clone()))
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("task"))?;
        let mut a: sys_tasks::ActiveModel = row.into();
        a.enable = Set(Some(enable));
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
