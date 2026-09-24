//! The internal-message write flow (send → recipient rows → the
//! notification the SSE hub pushes), the message CRUD face, the recipient
//! inbox face, and the task face (admin CRUD over sys_tasks plus the
//! start/stop/restart controls as the enable-column flips).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::messaging_repo as repo;
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
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

/// The operator identity when present (the reference tolerates its
/// absence on some flows, defaulting to operator 0).
fn optional_operator_user_id<T>(request: &tonic::Request<T>) -> i64 {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(0)
}

/// The caller's tenant scope off the BFF-forwarded header (0 = platform,
/// which scopes to nothing — the reference's `maybeTenantFromViewer`
/// only predicate applies for a *named* tenant).
fn request_tenant_of<T>(request: &tonic::Request<T>) -> i64 {
    request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
}

// ── enum converters ──────────────────────────────────────────────────
// The enum columns hold the proto enum *names* (the reference's
// EnumTypeConverter round-trips by name); these are the contract's
// name ↔ number pairs.

fn message_status_num(name: &str) -> Option<i32> {
    Some(match name {
        "DRAFT" => 0,
        "PUBLISHED" => 1,
        "SCHEDULED" => 2,
        "REVOKED" => 3,
        "ARCHIVED" => 5,
        "DELETED" => 6,
        _ => return None,
    })
}

fn message_status_name(v: i32) -> Option<&'static str> {
    Some(match v {
        0 => "DRAFT",
        1 => "PUBLISHED",
        2 => "SCHEDULED",
        3 => "REVOKED",
        5 => "ARCHIVED",
        6 => "DELETED",
        _ => return None,
    })
}

fn message_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "NOTIFICATION" => 0,
        "PRIVATE" => 1,
        "GROUP" => 2,
        _ => return None,
    })
}

fn message_type_name(v: i32) -> Option<&'static str> {
    Some(match v {
        0 => "NOTIFICATION",
        1 => "PRIVATE",
        2 => "GROUP",
        _ => return None,
    })
}

fn recipient_status_num(name: &str) -> Option<i32> {
    Some(match name {
        "SENT" => 0,
        "RECEIVED" => 1,
        "READ" => 2,
        "REVOKED" => 3,
        "DELETED" => 4,
        _ => return None,
    })
}

fn recipient_status_name(v: i32) -> Option<&'static str> {
    Some(match v {
        0 => "SENT",
        1 => "RECEIVED",
        2 => "READ",
        3 => "REVOKED",
        4 => "DELETED",
        _ => return None,
    })
}

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

// ── InternalMessage write flow ───────────────────────────────────────

fn message_proto(r: internal_messages::Model) -> imv1::InternalMessage {
    imv1::InternalMessage {
        id: Some(r.id as u32),
        title: r.title,
        content: r.content,
        sender_id: Some(r.sender_id as u32),
        category_id: r.category_id.map(|v| v as u32),
        status: r.status.as_deref().and_then(message_status_num),
        r#type: r.r#type.as_deref().and_then(message_type_num),
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
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
        status: r.status.as_deref().and_then(recipient_status_num),
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

/// The Create row shape of the reference's internalMessageRepo: fields
/// are set nillable from the submitted DTO, `sender_id` always (0 when
/// the client omitted it, the column is NOT NULL), `created_at` always,
/// and an explicit id only when the DTO carries one.
fn internal_message_insert_model(
    data: imv1::InternalMessage,
    created_by: Option<i64>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> internal_messages::ActiveModel {
    let mut m = internal_messages::ActiveModel {
        tenant_id: Set(data.tenant_id.map(|v| v as i64)),
        title: Set(data.title),
        content: Set(data.content),
        sender_id: Set(data.sender_id.unwrap_or(0) as i64),
        category_id: Set(data.category_id.map(|v| v as i64)),
        status: Set(data
            .status
            .and_then(message_status_name)
            .map(str::to_string)),
        r#type: Set(data.r#type.and_then(message_type_name).map(str::to_string)),
        created_by: Set(created_by),
        created_at: Set(Some(now)),
        ..Default::default()
    };
    if let Some(id) = data.id {
        m.id = Set(id as i64);
    }
    m
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

    async fn create_message(
        &self,
        request: Request<imv1::CreateInternalMessageRequest>,
    ) -> Result<Response<imv1::InternalMessage>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("invalid parameter"));
        };
        let created_by = data.created_by.map(|v| v as i64);
        let row = internal_message_insert_model(data, created_by, store::now())
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(message_proto(row)))
    }

    async fn update_message(
        &self,
        request: Request<imv1::UpdateInternalMessageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant_id = request_tenant_of(&request);
        let operator_id = optional_operator_user_id(&request);
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("invalid parameter"));
        };
        let id = req.id as i64;

        let existing = internal_messages::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?;
        if existing.is_none() {
            // allow_missing: the create fallback reuses the submitted
            // data with created_by taking updated_by's value (the
            // reference's swap). Without allow_missing the reference's
            // bulk update simply affects no rows — a silent success.
            if req.allow_missing.unwrap_or(false) {
                let created_by = data.updated_by.map(|v| v as i64);
                internal_message_insert_model(data, created_by, store::now())
                    .insert(&self.state.db)
                    .await
                    .map_err(db_status)?;
            }
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        let row = existing.unwrap();
        // The reference's tenant predicate applies for named tenants
        // only; a cross-tenant bulk update affects no rows (silent
        // success there, matching the port below).
        if tenant_id > 0 && row.tenant_id.unwrap_or(0) != tenant_id {
            return Ok(Response::new(pbjson_types::Empty {}));
        }

        let mut a: internal_messages::ActiveModel = row.into();
        if let Some(v) = data.title {
            a.title = Set(Some(v));
        }
        if let Some(v) = data.content {
            a.content = Set(Some(v));
        }
        if let Some(v) = data.sender_id {
            a.sender_id = Set(v as i64);
        }
        if let Some(v) = data.category_id {
            a.category_id = Set(Some(v as i64));
        }
        if let Some(v) = data.status.and_then(message_status_name) {
            a.status = Set(Some(v.to_string()));
        }
        if let Some(v) = data.r#type.and_then(message_type_name) {
            a.r#type = Set(Some(v.to_string()));
        }
        a.updated_at = Set(Some(store::now()));
        // updated_by is forced from the caller identity, the client
        // value is ignored (the reference's viewer rule).
        if operator_id > 0 {
            a.updated_by = Set(Some(operator_id));
        }
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete_message(
        &self,
        request: Request<imv1::DeleteInternalMessageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let tenant_id = request_tenant_of(&request);
        let req = request.into_inner();
        let Some(imv1::delete_internal_message_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let id = id as i64;
        if id == 0 {
            return Err(bad("invalid parameter"));
        }

        // 消息体删除与收件人级联清理同一事务提交（参照仓的 Tx +
        // CleanByMessageID）；按 id（及具名租户域）删除，未命中时参照
        // 仓的批量删除是静默 no-op，这里同样不报 not found。
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let mut del =
            internal_messages::Entity::delete_many().filter(internal_messages::Column::Id.eq(id));
        if tenant_id > 0 {
            del = del.filter(internal_messages::Column::TenantId.eq(tenant_id));
        }
        del.exec(&txn).await.map_err(db_status)?;
        internal_message_recipients::Entity::delete_many()
            .filter(internal_message_recipients::Column::MessageId.eq(id))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
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
        let mut events: Vec<serde_json::Value> = Vec::new();
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
            events.push(serde_json::json!({
                "uid": uid,
                "payload": {
                    "id": proto.id,
                    "messageId": proto.message_id,
                    "recipientUserId": proto.recipient_user_id,
                    "status": "RECEIVED",
                    "title": req.title,
                    "content": req.content,
                    "receivedAt": proto.received_at.map(|ts| serde_json::json!({"seconds": ts.seconds, "nanos": ts.nanos})),
                },
            }));
        }

        // The event list rides a response metadata key — base64 of the
        // JSON array, because gRPC metadata is header-shaped: raw UTF-8
        // (Chinese titles/contents) would ride as obs-text and the BFF's
        // ASCII-only read would drop the whole fan-out.
        let wire =
            serde_json::to_string(&events).map_err(|_| Status::internal("sse payload encode"))?;
        use base64::Engine as _;
        let mut resp = Response::new(imv1::SendMessageResponse {
            message_id: msg.id as u32,
        });
        resp.metadata_mut().insert(
            "x-sse-payload",
            base64::engine::general_purpose::STANDARD
                .encode(wire)
                .parse()
                .map_err(|_| Status::internal("payload header"))?,
        );
        Ok(resp)
    }

    async fn revoke_message(
        &self,
        request: Request<imv1::RevokeMessageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // 仅消息发送者本人（或平台上下文）可撤销消息体，防止跨租户/
        // 越权删除他人消息；操作者身份只认元数据里的登录态。
        let operator_id = operator_of(&request)?;
        let is_platform = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0)
            == 0;
        let req = request.into_inner();

        let txn = self.state.db.begin().await.map_err(db_status)?;
        let msg = internal_messages::Entity::find_by_id(req.message_id as i64)
            .one(&txn)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("internal message"))?;
        if !is_platform && msg.sender_id != operator_id {
            return Err(forbidden("only the sender can revoke this message"));
        }

        // 删消息体 + 级联清理该消息的收件人收件箱行（同一事务，避免悬
        // 空收件行指向已删除的消息体）。
        internal_messages::Entity::delete_by_id(msg.id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        internal_message_recipients::Entity::delete_many()
            .filter(internal_message_recipients::Column::MessageId.eq(req.message_id as i64))
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
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
        use proto::proto::pagination::paging_request::FilteringType;
        let uid = operator_of(&request)?;
        let req = request.into_inner();
        // The status form filter the front-end sends alongside its
        // (operator-pinned) recipient scope. The front page may also echo
        // an unresolved recipient_user_id ("undefined") — the operator
        // metadata wins, so the query's uid form is ignored outright and
        // `field__contains`-style operator suffixes are tolerated.
        let mut cond = sea_orm::Condition::all()
            .add(internal_message_recipients::Column::RecipientUserId.eq(uid));
        if let Some(FilteringType::Query(q)) = &req.filtering_type {
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(q)
            {
                if let Some(v) = map
                    .iter()
                    .find(|(k, _)| k.split("__").next() == Some("status"))
                    .and_then(|(_, v)| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    // The front form posts the proto enum number; the row
                    // stores the value name — accept either form.
                    let name = v
                        .parse::<i32>()
                        .ok()
                        .and_then(recipient_status_name)
                        .unwrap_or(v);
                    cond = cond.add(internal_message_recipients::Column::Status.eq(name));
                }
            }
        }
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_message_recipients::Entity::find().filter(cond),
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        // The inbox shows the message's title/content beside the receipt
        // row — batch-join the referenced messages.
        let msg_ids: Vec<i64> = rows.iter().filter_map(|r| r.message_id).collect();
        let msgs: std::collections::HashMap<i64, (Option<String>, Option<String>)> =
            if msg_ids.is_empty() {
                std::collections::HashMap::new()
            } else {
                internal_messages::Entity::find()
                    .filter(internal_messages::Column::Id.is_in(msg_ids))
                    .all(&self.state.db)
                    .await
                    .map_err(db_status)?
                    .into_iter()
                    .map(|m| (m.id, (m.title, m.content)))
                    .collect()
            };
        let items = rows
            .into_iter()
            .map(|r| {
                let mut p = recipient_proto(r.clone());
                if let Some((title, content)) = msgs.get(&r.message_id.unwrap_or(0)) {
                    p.title = title.clone();
                    p.content = content.clone();
                }
                p
            })
            .collect();
        Ok(Response::new(imv1::ListUserInboxResponse { items, total }))
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

    async fn mark_notifications_status(
        &self,
        request: Request<imv1::MarkNotificationsStatusRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let uid = operator_of(&request)?;
        let req = request.into_inner();
        if req.recipient_ids.is_empty() {
            return Err(bad("invalid parameter"));
        }
        let Some(new_status) = recipient_status_name(req.new_status) else {
            // 参照仓的枚举转换器对未知枚举值落不出目标状态，这里直接
            // 拒绝而不是静默 no-op。
            return Err(bad("unknown notification status"));
        };
        let _ = req.user_id; // 强制使用调用者身份，忽略请求体里的 user_id
        repo::mark_notifications_status(
            &self.state.db,
            req.recipient_ids.into_iter().map(|v| v as i64).collect(),
            uid,
            new_status,
            store::now(),
        )
        .await?;
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
                repo::tasks_by_id(&self.state.db, id as i64).await?
            }
            Some(taskv1::get_task_request::QueryBy::TypeName(type_name)) => {
                if !authed {
                    return Err(bad("tenant scope required to query task by type name"));
                }
                repo::task_by_type_name(&self.state.db, &type_name, tenant_id)
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
        repo::insert_tasks(
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
                repo::insert_tasks(
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
        repo::update_tasks(&self.state.db, a).await?;
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
        repo::tasks_by_id(&self.state.db, id).await?;
        repo::delete_task_scoped(&self.state.db, id, (tenant_id > 0).then_some(tenant_id)).await?;
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
        let row = repo::task_by_type_name(&self.state.db, &req.type_name, tenant_id)
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
    fn message_status_round_trips_and_rejects_unknowns() {
        for (name, num) in [
            ("DRAFT", 0),
            ("PUBLISHED", 1),
            ("SCHEDULED", 2),
            ("REVOKED", 3),
            ("ARCHIVED", 5),
            ("DELETED", 6),
        ] {
            assert_eq!(message_status_num(name), Some(num), "{name}");
            assert_eq!(message_status_name(num), Some(name), "{num}");
        }
        // The gap ordinal (4) and everything off the table reject.
        for num in [4, 7, -1, 100] {
            assert_eq!(message_status_name(num), None, "{num}");
        }
        for name in ["REMOVED", "published", ""] {
            assert_eq!(message_status_num(name), None, "{name}");
        }
    }

    #[test]
    fn message_type_round_trips_and_rejects_unknowns() {
        for (name, num) in [("NOTIFICATION", 0), ("PRIVATE", 1), ("GROUP", 2)] {
            assert_eq!(message_type_num(name), Some(num), "{name}");
            assert_eq!(message_type_name(num), Some(name), "{num}");
        }
        for num in [3, -1, 100] {
            assert_eq!(message_type_name(num), None, "{num}");
        }
        for name in ["BROADCAST", "notification", ""] {
            assert_eq!(message_type_num(name), None, "{name}");
        }
    }

    #[test]
    fn recipient_status_round_trips_and_rejects_unknowns() {
        for (name, num) in [
            ("SENT", 0),
            ("RECEIVED", 1),
            ("READ", 2),
            ("REVOKED", 3),
            ("DELETED", 4),
        ] {
            assert_eq!(recipient_status_num(name), Some(num), "{name}");
            assert_eq!(recipient_status_name(num), Some(name), "{num}");
        }
        for num in [5, -1, 100] {
            assert_eq!(recipient_status_name(num), None, "{num}");
        }
        for name in ["UNREAD", "read", ""] {
            assert_eq!(recipient_status_num(name), None, "{name}");
        }
    }

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
