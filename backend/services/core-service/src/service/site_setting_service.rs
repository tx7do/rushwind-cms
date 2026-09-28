//! The site setting service (site_setting_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::site_settings;
use store::paging::fetch_paged;

use proto::proto::site::service::v1 as sitev1;

// ── SiteSetting ──────────────────────────────────────────────────────

fn site_setting_proto(r: site_settings::Model) -> sitev1::SiteSetting {
    sitev1::SiteSetting {
        id: Some(r.id as u32),
        site_id: r.site_id.map(|v| v as u32),
        locale: r.locale,
        group: r.r#group,
        key: r.key,
        value: r.value,
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        options: r
            .options
            .and_then(|j| serde_json::from_value(j).ok())
            .unwrap_or_default(),
        is_required: r.is_required,
        validation_regex: r.validation_regex,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct SiteSettingService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::site_setting_service_server::SiteSettingService for SiteSettingService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<sitev1::ListSiteSettingResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            site_settings::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(sitev1::ListSiteSettingResponse {
            items: rows.into_iter().map(site_setting_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<sitev1::GetSiteSettingRequest>,
    ) -> Result<Response<sitev1::SiteSetting>, Status> {
        let req = request.into_inner();
        let id = req.id;
        let row = site_setting_repo::site_settings_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(site_setting_proto(row)))
    }

    async fn create(
        &self,
        request: Request<sitev1::CreateSiteSettingRequest>,
    ) -> Result<Response<sitev1::SiteSetting>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = site_settings::ActiveModel {
            site_id: Set(data.site_id.map(|v| v as i64)),
            locale: Set(data.locale),
            group: Set(data.r#group),
            key: Set(Some(data.key.unwrap_or_default())),
            value: Set(data.value),
            label: Set(data.label),
            description: Set(data.description),
            placeholder: Set(data.placeholder),
            options: Set((!data.options.is_empty())
                .then(|| serde_json::to_value(&data.options).ok())
                .flatten()),
            is_required: Set(data.is_required),
            validation_regex: Set(data.validation_regex),
            created_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(site_setting_proto(row)))
    }

    async fn update(
        &self,
        request: Request<sitev1::UpdateSiteSettingRequest>,
    ) -> Result<Response<sitev1::SiteSetting>, Status> {
        let req = request.into_inner();
        let row = site_settings::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("site setting"))?;
        let mut a: site_settings::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.value {
                a.value = Set(Some(v));
            }
            if let Some(v) = data.label {
                a.label = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(site_setting_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<sitev1::DeleteSiteSettingRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(sitev1::delete_site_setting_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        site_setting_repo::delete_site_settings(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::site_setting_repo;
