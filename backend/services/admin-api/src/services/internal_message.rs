//! The admin internal-message face: send_message rides the core RPC
//! then fans the notification payload out over the SSE hub (the core
//! returns the per-recipient events via response metadata — no
//! re-query); the inbox/mark-read/delete faces proxy through.

use std::sync::Arc;

use crate::services::{map_status, with_operator};
use crate::state::{AppState, StatusError};

use proto::proto::internal_message::service::v1 as imv1;

type Ctx = rushwind_http_binding::ctx::RequestContext;

pub struct InternalMessageService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::gen_admin::services::InternalMessageServiceHandlers for InternalMessageService {
    async fn list_message(
        &self,
        ctx: Ctx,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<imv1::ListInternalMessageResponse, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_message(
        &self,
        ctx: Ctx,
        req: imv1::GetInternalMessageRequest,
    ) -> Result<imv1::InternalMessage, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update_message(
        &self,
        ctx: Ctx,
        req: imv1::UpdateInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        core.update_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }

    async fn delete_message(
        &self,
        ctx: Ctx,
        req: imv1::DeleteInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        core.delete_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }

    async fn send_message(
        &self,
        ctx: Ctx,
        req: imv1::SendMessageRequest,
    ) -> Result<imv1::SendMessageResponse, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        let resp = core
            .send_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?;
        // The SSE fan-out: the payload metadata carries the base64 of a
        // JSON event list [{uid, payload}, …] — decoded here, each event
        // pushed to its user's stream.
        if let Some(wire) = resp
            .metadata()
            .get("x-sse-payload")
            .and_then(|v| v.to_str().ok())
        {
            use base64::Engine as _;
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(wire)
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
            if let Some(serde_json::Value::Array(events)) = decoded {
                for ev in events {
                    let uid = ev.get("uid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let payload = ev.get("payload").cloned().unwrap_or_default();
                    if uid > 0 && payload.is_object() {
                        let body =
                            serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
                        self.state.hub.publish(uid, body);
                    }
                }
            }
        }
        Ok(resp.into_inner())
    }

    async fn revoke_message(
        &self,
        ctx: Ctx,
        req: imv1::RevokeMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .revoke_message(with_operator(&ctx, req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}
