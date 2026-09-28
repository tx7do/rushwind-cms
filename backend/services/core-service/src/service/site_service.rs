//! The site service (site_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::sites;
use store::paging::fetch_paged;

use proto::proto::site::service::v1 as sitev1;

// ── Site ─────────────────────────────────────────────────────────────

/// The enum-name ↔ number bridges the varchar columns carry.
pub(crate) fn link_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "URL" => 1,
        "PAGE" => 2,
        "POST" => 3,
        "CATEGORY" => 4,
        "TAG" => 5,
        _ => return None,
    })
}

pub(crate) fn link_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "URL",
            2 => "PAGE",
            3 => "POST",
            4 => "CATEGORY",
            5 => "TAG",
            _ => return None,
        }
        .to_string(),
    )
}

pub(crate) fn location_num(name: &str) -> Option<i32> {
    Some(match name {
        "HEADER" => 1,
        "FOOTER" => 2,
        "SIDEBAR" => 3,
        _ => return None,
    })
}

pub(crate) fn location_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "HEADER",
            2 => "FOOTER",
            3 => "SIDEBAR",
            _ => return None,
        }
        .to_string(),
    )
}

fn site_proto(r: sites::Model) -> sitev1::Site {
    sitev1::Site {
        id: Some(r.id as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        name: r.name,
        slug: r.slug,
        domain: r.domain,
        alternate_domains: r
            .alternate_domains
            .and_then(|j| serde_json::from_value::<Vec<String>>(j).ok())
            .unwrap_or_default(),
        is_default: r.is_default,
        status: r.status.as_deref().map(|_| 1),
        default_locale: r.default_locale,
        template: r.template,
        theme: r.theme,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct SiteService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::site_service_server::SiteService for SiteService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<sitev1::ListSiteResponse>, Status> {
        let (rows, total) =
            fetch_paged(&self.state.db, sites::Entity::find(), &request.into_inner())
                .await
                .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(sitev1::ListSiteResponse {
            items: rows.into_iter().map(site_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<sitev1::GetSiteRequest>,
    ) -> Result<Response<sitev1::Site>, Status> {
        let req = request.into_inner();
        let id = req.id;
        let row = site_repo::sites_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(site_proto(row)))
    }

    async fn get_site_by_domain(
        &self,
        request: Request<sitev1::GetSiteByDomainRequest>,
    ) -> Result<Response<sitev1::Site>, Status> {
        let req = request.into_inner();
        let row = sites::Entity::find()
            .filter(sites::Column::Domain.eq(req.domain))
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("site"))?;
        Ok(Response::new(site_proto(row)))
    }

    async fn create(
        &self,
        request: Request<sitev1::CreateSiteRequest>,
    ) -> Result<Response<sitev1::Site>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = sites::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            name: Set(data.name),
            slug: Set(data.slug),
            domain: Set(data.domain),
            alternate_domains: Set((!data.alternate_domains.is_empty())
                .then(|| serde_json::to_value(&data.alternate_domains).ok())
                .flatten()),
            is_default: Set(data.is_default),
            status: Set(Some("ON".to_string())),
            default_locale: Set(data.default_locale),
            template: Set(data.template),
            theme: Set(data.theme),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(site_proto(row)))
    }

    async fn update(
        &self,
        request: Request<sitev1::UpdateSiteRequest>,
    ) -> Result<Response<sitev1::Site>, Status> {
        let req = request.into_inner();
        let row = sites::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("site"))?;
        let mut a: sites::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.slug {
                a.slug = Set(Some(v));
            }
            if let Some(v) = data.domain {
                a.domain = Set(Some(v));
            }
            if let Some(v) = data.default_locale {
                a.default_locale = Set(Some(v));
            }
            if let Some(v) = data.template {
                a.template = Set(Some(v));
            }
            if let Some(v) = data.theme {
                a.theme = Set(Some(v));
            }
            if let Some(v) = data.is_default {
                a.is_default = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(site_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<sitev1::DeleteSiteRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(sitev1::delete_site_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        site_repo::delete_sites(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::site_repo;
