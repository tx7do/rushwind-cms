//! The admin internal-message face: send_message rides the core RPC
//! then fans the notification payload out over the SSE hub (the core
//! returns the per-recipient events via response metadata — no
//! re-query); the inbox/mark-read/delete faces proxy through.

use std::sync::Arc;

use crate::services::map_status;
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
        _ctx: Ctx,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<imv1::ListInternalMessageResponse, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .list_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get_message(
        &self,
        _ctx: Ctx,
        req: imv1::GetInternalMessageRequest,
    ) -> Result<imv1::InternalMessage, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .get_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn update_message(
        &self,
        _ctx: Ctx,
        req: imv1::UpdateInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        core.update_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }

    async fn delete_message(
        &self,
        _ctx: Ctx,
        req: imv1::DeleteInternalMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        core.delete_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        Ok(pbjson_types::Empty {})
    }

    async fn send_message(
        &self,
        _ctx: Ctx,
        req: imv1::SendMessageRequest,
    ) -> Result<imv1::SendMessageResponse, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        let resp = core
            .send_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?;
        // The SSE fan-out: the payload metadata carries
        // [uid:json, uid:json, ...] — push each to its user's stream.
        if let Some(payload) = resp
            .metadata()
            .get("x-sse-payload")
            .and_then(|v| v.to_str().ok())
        {
            for entry in payload.split("},{") {
                // Re-split robustly: entries are uid:{json} pairs.
                let (uid, json) = match entry.split_once(':') {
                    Some((uid, json)) => (uid, json),
                    None => continue,
                };
                let uid: u32 = match uid
                    .trim_matches(|c: char| c == '[' || c == ']' || c == ',')
                    .parse()
                {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let mut body = json.to_string();
                if !body.starts_with('{') {
                    body = format!("{{{body}");
                }
                if !body.ends_with('}') {
                    body = format!("{body}}}");
                }
                self.state.hub.publish(uid, body);
            }
        }
        Ok(resp.into_inner())
    }

    async fn revoke_message(
        &self,
        _ctx: Ctx,
        req: imv1::RevokeMessageRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = imv1::internal_message_service_client::InternalMessageServiceClient::new(
            self.state.core_channel.clone(),
        );
        Ok(core
            .revoke_message(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}
