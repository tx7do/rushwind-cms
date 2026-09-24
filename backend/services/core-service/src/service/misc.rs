//! The remaining read faces: OrgUnit, Position, LoginPolicy,
//! ContentModel, MediaAsset, Task, Translator, InternalMessage×3 —
//! list/get over the golden schema (write paths land per-module;
//! MediaAsset carries its full CRUD — the reference's media library
//! face).

use std::sync::Arc;

use sea_orm::{EntityTrait, PaginatorTrait, Set};
use tonic::{Request, Response, Status};

use crate::data::misc_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    content_models, internal_message_categories, internal_message_recipients, internal_messages,
    media_assets, sys_login_policies, sys_org_units, sys_positions, sys_tasks,
};
use store::paging::fetch_paged;

use proto::proto::authentication::service::v1 as authv1;
use proto::proto::content::service::v1 as contentv1;
use proto::proto::identity::service::v1 as identityv1;
use proto::proto::internal_message::service::v1 as imv1;
use proto::proto::media::service::v1 as mediav1;
use proto::proto::task::service::v1 as taskv1;

// ── OrgUnit ──────────────────────────────────────────────────────────

fn org_unit_proto(r: sys_org_units::Model) -> identityv1::OrgUnit {
    identityv1::OrgUnit {
        id: Some(r.id as u32),
        name: Some(r.name),
        code: r.code,
        path: r.path,
        status: Some(1),
        sort_order: r.sort_order.map(|v| v as u32),
        leader_id: r.leader_id.map(|v| v as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        remark: r.remark,
        description: r.description,
        r#type: Some(1),
        ..Default::default()
    }
}

pub struct OrgUnitServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::org_unit_service_server::OrgUnitService for OrgUnitServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListOrgUnitResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_org_units::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListOrgUnitResponse {
            items: rows.into_iter().map(org_unit_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountOrgUnitResponse>, Status> {
        let total = sys_org_units::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountOrgUnitResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetOrgUnitRequest>,
    ) -> Result<Response<identityv1::OrgUnit>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_org_unit_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::org_units_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(org_unit_proto(row)))
    }
}

// ── Position ─────────────────────────────────────────────────────────

fn position_proto(r: sys_positions::Model) -> identityv1::Position {
    identityv1::Position {
        id: Some(r.id as u32),
        name: Some(r.name),
        code: Some(r.code),
        headcount: Some(r.headcount as u32),
        sort_order: r.sort_order.map(|v| v as u32),
        status: Some(1),
        remark: r.remark,
        description: r.description,
        job_family: r.job_family,
        job_grade: r.job_grade,
        level: r.level,
        is_key_position: Some(r.is_key_position),
        tenant_id: r.tenant_id.map(|v| v as u32),
        ..Default::default()
    }
}

pub struct PositionServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::position_service_server::PositionService for PositionServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListPositionResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_positions::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListPositionResponse {
            items: rows.into_iter().map(position_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountPositionResponse>, Status> {
        let total = sys_positions::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountPositionResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetPositionRequest>,
    ) -> Result<Response<identityv1::Position>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_position_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::positions_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(position_proto(row)))
    }
}

// ── LoginPolicy ──────────────────────────────────────────────────────

fn login_policy_proto(r: sys_login_policies::Model) -> authv1::LoginPolicy {
    authv1::LoginPolicy {
        id: Some(r.id as u32),
        target_id: r.target_id.map(|v| v as u32),
        method: r.method.map(|_| 1),
        value: r.value,
        reason: r.reason,
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct LoginPolicyServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl authv1::login_policy_service_server::LoginPolicyService for LoginPolicyServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<authv1::ListLoginPolicyResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_login_policies::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(authv1::ListLoginPolicyResponse {
            items: rows.into_iter().map(login_policy_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<authv1::GetLoginPolicyRequest>,
    ) -> Result<Response<authv1::LoginPolicy>, Status> {
        let req = request.into_inner();
        let Some(authv1::get_login_policy_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::login_policies_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(login_policy_proto(row)))
    }
}

// ── ContentModel ─────────────────────────────────────────────────────

fn content_model_proto(r: content_models::Model) -> contentv1::ContentModel {
    contentv1::ContentModel {
        id: Some(r.id as u32),
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct ContentModelServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::content_model_service_server::ContentModelService for ContentModelServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListContentModelResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            content_models::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(contentv1::ListContentModelResponse {
            items: rows.into_iter().map(content_model_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_content_model_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::content_models_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(content_model_proto(row)))
    }
}

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

pub struct MediaAssetServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl mediav1::media_asset_service_server::MediaAssetService for MediaAssetServiceImpl {
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
        let row = repo::media_assets_by_id(&self.state.db, id as i64).await?;
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
        let row = repo::insert_media_assets(
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
        let row = repo::media_assets_by_id(&self.state.db, req.id as i64).await?;
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
        let row = repo::update_media_assets(&self.state.db, a).await?;
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
        repo::delete_media_assets(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Task ─────────────────────────────────────────────────────────────

fn task_proto(r: sys_tasks::Model) -> taskv1::Task {
    taskv1::Task {
        id: Some(r.id as u32),
        r#type: r.r#type.clone().map(|_| 1),
        type_name: r.type_name.clone(),
        cron_spec: r.cron_spec.clone(),
        enable: r.enable,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct TaskServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl taskv1::task_service_server::TaskService for TaskServiceImpl {
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
        let total = sys_tasks::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(taskv1::CountTaskResponse { count: total }))
    }
}

// ── InternalMessage cluster (minimal reads) ──────────────────────────

fn internal_message_proto(r: internal_messages::Model) -> imv1::InternalMessage {
    imv1::InternalMessage {
        id: Some(r.id as u32),
        title: r.title.clone(),
        content: r.content.clone(),
        sender_id: Some(r.sender_id as u32),
        category_id: r.category_id.map(|v| v as u32),
        status: r.status.clone().map(|_| 1),
        r#type: r.r#type.clone().map(|_| 1),
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct InternalMessageServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_service_server::InternalMessageService for InternalMessageServiceImpl {
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
            items: rows.into_iter().map(internal_message_proto).collect(),
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
        let row = internal_messages::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("internal message"))?;
        Ok(Response::new(internal_message_proto(row)))
    }
}

fn internal_message_category_proto(
    r: internal_message_categories::Model,
) -> imv1::InternalMessageCategory {
    imv1::InternalMessageCategory {
        id: Some(r.id as u32),
        ..Default::default()
    }
}

pub struct InternalMessageCategoryServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_category_service_server::InternalMessageCategoryService
    for InternalMessageCategoryServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageCategoryResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_message_categories::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageCategoryResponse {
            items: rows
                .into_iter()
                .map(internal_message_category_proto)
                .collect(),
            total,
        }))
    }
}

fn internal_message_recipient_proto(
    r: internal_message_recipients::Model,
) -> imv1::InternalMessageRecipient {
    imv1::InternalMessageRecipient {
        id: Some(r.id as u32),
        ..Default::default()
    }
}

pub struct InternalMessageRecipientServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_recipient_service_server::InternalMessageRecipientService
    for InternalMessageRecipientServiceImpl
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
            items: rows
                .into_iter()
                .map(internal_message_recipient_proto)
                .collect(),
            total,
        }))
    }
}
