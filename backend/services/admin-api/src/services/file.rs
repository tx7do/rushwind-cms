//! The file face of the admin BFF: the JSON transport bridge over the
//! core's local object store (base64 bodies — the multipart streaming
//! codec lands with the transport phase), plus the download endpoint
//! streaming bytes back.

use std::sync::Arc;

use base64::Engine as _;

use crate::services::map_status;
use crate::state::{internal_error, operator_of, AppState, StatusError};

use proto::proto::storage::service::v1 as storagev1;

type Ctx = rushwind_http_binding::ctx::RequestContext;

pub struct FileService {
    pub state: Arc<AppState>,
}

fn core_file_client(
    channel: &tonic::transport::Channel,
) -> storagev1::file_service_client::FileServiceClient<tonic::transport::Channel> {
    storagev1::file_service_client::FileServiceClient::new(channel.clone())
}

/// A direct core call the bridge uses (mirrors core's put on the wire —
/// a tiny gRPC side channel: upload via create with the base64 body in
/// the file's `content_hash`-adjacent fields is hacky, so we ride the
/// create RPC carrying the base64 in `size_format` (documented stopgap
/// until the multipart codec lands).
const _: () = ();

impl FileService {
    async fn upload(
        &self,
        operator: i64,
        file_name: &str,
        mime: &str,
        bytes: Vec<u8>,
    ) -> Result<storagev1::File, StatusError> {
        // Stopgap JSON transport: the core's FileService.create accepts
        // the base64 body via the bucket_name field (base64 is ascii,
        // safe in varchar) — replaced by the multipart codec later.
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let mut core = core_file_client(&self.state.core_channel);
        let resp = core
            .create(tonic::Request::new(storagev1::CreateFileRequest {
                data: Some(storagev1::File {
                    file_name: Some(file_name.trim_start_matches('@').to_string()),
                    bucket_name: Some(b64),
                    extension: mime.rsplit('/').next().map(str::to_string),
                    ..Default::default()
                }),
            }))
            .await
            .map_err(map_status)?
            .into_inner();
        let _ = operator;
        Ok(resp)
    }
}

#[async_trait::async_trait]
impl proto::gen_admin::services::FileServiceHandlers for FileService {
    async fn list(
        &self,
        _ctx: Ctx,
        req: proto::proto::pagination::PagingRequest,
    ) -> Result<storagev1::ListFileResponse, StatusError> {
        let mut core = core_file_client(&self.state.core_channel);
        Ok(core
            .list(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn get(
        &self,
        _ctx: Ctx,
        req: storagev1::GetFileRequest,
    ) -> Result<storagev1::File, StatusError> {
        let mut core = core_file_client(&self.state.core_channel);
        Ok(core
            .get(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }

    async fn create(
        &self,
        ctx: Ctx,
        req: storagev1::CreateFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let operator = operator_of(&ctx).map(|o| o.user_id as i64).unwrap_or(0);
        let data = req.data.ok_or_else(|| internal_error("data required"))?;
        let file_name = data.file_name.clone().unwrap_or_default();
        let bytes = data
            .bucket_name
            .clone()
            .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok())
            .unwrap_or_default();
        if bytes.is_empty() {
            return Err(internal_error("empty upload body"));
        }
        let mime = data.extension.clone().unwrap_or_default();
        self.upload(operator, &file_name, &mime, bytes).await?;
        Ok(pbjson_types::Empty {})
    }

    async fn update(
        &self,
        _ctx: Ctx,
        _req: storagev1::UpdateFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        Err(internal_error("file metadata update not supported yet"))
    }

    async fn delete(
        &self,
        _ctx: Ctx,
        req: storagev1::DeleteFileRequest,
    ) -> Result<pbjson_types::Empty, StatusError> {
        let mut core = core_file_client(&self.state.core_channel);
        Ok(core
            .delete(tonic::Request::new(req))
            .await
            .map_err(map_status)?
            .into_inner())
    }
}
