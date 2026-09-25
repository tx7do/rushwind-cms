//! The media-asset face: full CRUD over the media library (the
//! reference's media service, asset type / processing status as
//! varchar enum names).
use std::sync::Arc;

use sea_orm::{
    EntityTrait, Set,
};
use tonic::{Request, Response, Status};

use crate::state::{bad, ts_to_proto, AppState};
use store::entities::media_assets;
use store::paging::fetch_paged;

use proto::proto::media::service::v1 as mediav1;

// ── MediaAsset ───────────────────────────────────────────────────────

fn asset_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "ASSET_TYPE_IMAGE",
            2 => "ASSET_TYPE_VIDEO",
            3 => "ASSET_TYPE_DOCUMENT",
            4 => "ASSET_TYPE_AUDIO",
            5 => "ASSET_TYPE_ARCHIVE",
            100 => "ASSET_TYPE_OTHER",
            _ => return None,
        }
        .to_string(),
    )
}

fn processing_status_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "PROCESSING_STATUS_UPLOADING",
            2 => "PROCESSING_STATUS_PROCESSING",
            3 => "PROCESSING_STATUS_COMPLETED",
            4 => "PROCESSING_STATUS_FAILED",
            _ => return None,
        }
        .to_string(),
    )
}

fn media_asset_proto(r: media_assets::Model) -> mediav1::MediaAsset {
    mediav1::MediaAsset {
        id: Some(r.id as u32),
        filename: r.filename.clone(),
        r#type: r.r#type.clone().map(|_| 1),
        mime_type: r.mime_type.clone(),
        size: r.size.map(|v| v as u64),
        storage_path: r.storage_path.clone(),
        url: r.url.clone(),
        width: r.width.map(|v| v as u32),
        height: r.height.map(|v| v as u32),
        duration: r.duration.map(|v| v as u32),
        alt_text: r.alt_text.clone(),
        title: r.title.clone(),
        caption: r.caption.clone(),
        processing_status: r.processing_status.clone().map(|_| 1),
        processing_error: r.processing_error.clone(),
        file_hash: r.file_hash.clone(),
        file_id: r.file_id.map(|v| v as u32),
        reference_count: r.reference_count.map(|v| v as u32),
        is_private: r.is_private,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct MediaAssetService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl mediav1::media_asset_service_server::MediaAssetService for MediaAssetService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<mediav1::ListMediaAssetResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            media_assets::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(mediav1::ListMediaAssetResponse {
            items: rows.into_iter().map(media_asset_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<mediav1::GetMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let id = req.id;
        let row = media_asset_repo::media_assets_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn create(
        &self,
        request: Request<mediav1::CreateMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = media_asset_repo::insert_media_assets(
            &self.state.db,
            media_assets::ActiveModel {
                filename: Set(data.filename),
                r#type: Set(data.r#type.and_then(asset_type_name)),
                mime_type: Set(data.mime_type),
                size: Set(data.size.map(|v| v as i64)),
                storage_path: Set(data.storage_path),
                url: Set(data.url),
                width: Set(data.width.map(|v| v as i64)),
                height: Set(data.height.map(|v| v as i64)),
                duration: Set(data.duration.map(|v| v as i64)),
                alt_text: Set(data.alt_text),
                title: Set(data.title),
                caption: Set(data.caption),
                processing_status: Set(data.processing_status.and_then(processing_status_name)),
                processing_error: Set(data.processing_error),
                file_hash: Set(data.file_hash),
                file_id: Set(data.file_id.map(|v| v as i64)),
                folder_id: Set(data.folder_id.map(|v| v as i64)),
                is_private: Set(data.is_private),
                created_by: Set(data.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn update(
        &self,
        request: Request<mediav1::UpdateMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let row = media_asset_repo::media_assets_by_id(&self.state.db, req.id as i64).await?;
        let mut a: media_assets::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.r#type {
                a.r#type = Set(asset_type_name(v));
            }
            if let Some(v) = data.filename {
                a.filename = Set(Some(v));
            }
            if let Some(v) = data.mime_type {
                a.mime_type = Set(Some(v));
            }
            if let Some(v) = data.size {
                a.size = Set(Some(v as i64));
            }
            if let Some(v) = data.storage_path {
                a.storage_path = Set(Some(v));
            }
            if let Some(v) = data.url {
                a.url = Set(Some(v));
            }
            if let Some(v) = data.width {
                a.width = Set(Some(v as i64));
            }
            if let Some(v) = data.height {
                a.height = Set(Some(v as i64));
            }
            if let Some(v) = data.duration {
                a.duration = Set(Some(v as i64));
            }
            if let Some(v) = data.alt_text {
                a.alt_text = Set(Some(v));
            }
            if let Some(v) = data.title {
                a.title = Set(Some(v));
            }
            if let Some(v) = data.caption {
                a.caption = Set(Some(v));
            }
            if let Some(v) = data.processing_status {
                a.processing_status = Set(processing_status_name(v));
            }
            if let Some(v) = data.processing_error {
                a.processing_error = Set(Some(v));
            }
            if let Some(v) = data.file_hash {
                a.file_hash = Set(Some(v));
            }
            if let Some(v) = data.file_id {
                a.file_id = Set(Some(v as i64));
            }
            if let Some(v) = data.folder_id {
                a.folder_id = Set(Some(v as i64));
            }
            if let Some(v) = data.is_private {
                a.is_private = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = media_asset_repo::update_media_assets(&self.state.db, a).await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<mediav1::DeleteMediaAssetRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let id = match req.query_by {
            Some(mediav1::delete_media_asset_request::QueryBy::Id(id)) => id as i64,
            _ => return Err(bad("query_by required")),
        };
        media_asset_repo::delete_media_assets(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::{media_asset_repo};

