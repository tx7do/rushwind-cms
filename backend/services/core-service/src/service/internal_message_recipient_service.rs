//! The internal-message recipient service — the inbox face: the
//! recipient listing/lookup, the read/status flips and the inbox
//! removal, mirroring the reference's
//! internal_message_recipient_service.go.


use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::{internal_message_recipient_repo};
use crate::service::context::{
    operator_of,
};
use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::{internal_message_recipients, internal_messages};
use store::paging::fetch_paged;

use proto::proto::internal_message::service::v1 as imv1;

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
pub(crate) fn recipient_proto(r: internal_message_recipients::Model) -> imv1::InternalMessageRecipient {
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

// ── Recipient inbox face ─────────────────────────────────────────────

pub struct InternalMessageRecipientService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_recipient_service_server::InternalMessageRecipientService
    for InternalMessageRecipientService
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
        let row = internal_message_recipient_repo::internal_message_recipients_by_id(&self.state.db, id as i64).await?;
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
        internal_message_recipient_repo::mark_notifications_status(
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

#[cfg(test)]
mod tests {
    use super::*;

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

}
