//! The internal-message service — the complete message face: the
//! send flow (message row → recipient rows), the CRUD over sys
//! messages and the revoke, mirroring the reference's
//! internal_message_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::data::internal_message_repo;
use crate::service::context::{
    operator_of, optional_operator_user_id, tenant_of as request_tenant_of,
};
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{internal_message_recipients, internal_messages};
use store::paging::fetch_paged;

use crate::service::internal_message_recipient_service::recipient_proto;
use proto::proto::internal_message::service::v1 as imv1;

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

pub struct InternalMessageService {
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
impl imv1::internal_message_service_server::InternalMessageService for InternalMessageService {
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
        let row = internal_message_repo::internal_messages_by_id(&self.state.db, id as i64).await?;
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
                    "receivedAt": proto.received_at.map(|ts| {
                        serde_json::json!({"seconds": ts.seconds, "nanos": ts.nanos})
                    }),
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
}
