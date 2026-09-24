//! The site domain services: Site, SiteSetting, Navigation,
//! NavigationItem — CRUD over the golden schema.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{navigation_items, navigations, site_settings, sites};
use store::paging::fetch_paged;

use proto::proto::site::service::v1 as sitev1;

// ── Site ─────────────────────────────────────────────────────────────

/// The enum-name ↔ number bridges the varchar columns carry.
fn link_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "URL" => 1,
        "PAGE" => 2,
        "POST" => 3,
        "CATEGORY" => 4,
        "TAG" => 5,
        _ => return None,
    })
}

fn link_type_name(v: i32) -> Option<String> {
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

fn location_num(name: &str) -> Option<i32> {
    Some(match name {
        "HEADER" => 1,
        "FOOTER" => 2,
        "SIDEBAR" => 3,
        _ => return None,
    })
}

fn location_name(v: i32) -> Option<String> {
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

pub struct SiteServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::site_service_server::SiteService for SiteServiceImpl {
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
        let row = sites::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("site"))?;
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
        sites::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

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

pub struct SiteSettingServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::site_setting_service_server::SiteSettingService for SiteSettingServiceImpl {
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
        let row = site_settings::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("site setting"))?;
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
        site_settings::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Navigation ───────────────────────────────────────────────────────

fn navigation_item_proto(r: navigation_items::Model) -> sitev1::NavigationItem {
    sitev1::NavigationItem {
        id: Some(r.id as u32),
        navigation_id: r.navigation_id.map(|v| v as u32),
        title: r.title,
        description: r.description,
        icon: r.icon,
        url: r.url,
        link_type: r.link_type.as_deref().and_then(link_type_num),
        object_id: r.object_id.map(|v| v as u32),
        sort_order: r.sort_order.map(|v| v as u32),
        is_open_new_tab: r.is_open_new_tab,
        is_invalid: r.is_invalid,
        parent_id: r.parent_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

fn navigation_proto(
    r: navigations::Model,
    items: Vec<navigation_items::Model>,
) -> sitev1::Navigation {
    sitev1::Navigation {
        id: Some(r.id as u32),
        name: r.name,
        location: r.location.as_deref().and_then(location_num),
        locale: r.locale,
        is_active: r.is_active,
        items: items.into_iter().map(navigation_item_proto).collect(),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

async fn navigation_items_of(
    db: &sea_orm::DatabaseConnection,
    navigation_id: i64,
) -> Result<Vec<navigation_items::Model>, Status> {
    navigation_items::Entity::find()
        .filter(navigation_items::Column::NavigationId.eq(navigation_id))
        .all(db)
        .await
        .map_err(db_status)
}

pub struct NavigationServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::navigation_service_server::NavigationService for NavigationServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<sitev1::ListNavigationResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            navigations::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let children = navigation_items_of(&self.state.db, r.id).await?;
            items.push(navigation_proto(r, children));
        }
        Ok(Response::new(sitev1::ListNavigationResponse {
            items,
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<sitev1::GetNavigationRequest>,
    ) -> Result<Response<sitev1::Navigation>, Status> {
        let req = request.into_inner();
        let Some(sitev1::get_navigation_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = navigations::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("navigation"))?;
        let children = navigation_items_of(&self.state.db, row.id).await?;
        Ok(Response::new(navigation_proto(row, children)))
    }

    async fn create(
        &self,
        request: Request<sitev1::CreateNavigationRequest>,
    ) -> Result<Response<sitev1::Navigation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = navigations::ActiveModel {
            name: Set(Some(data.name.clone().unwrap_or_default())),
            location: Set(data.location.and_then(location_name)),
            locale: Set(data.locale),
            is_active: Set(data.is_active),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        for item in data.items {
            navigation_items::ActiveModel {
                navigation_id: Set(Some(row.id)),
                title: Set(item.title),
                description: Set(item.description),
                icon: Set(item.icon),
                url: Set(item.url),
                link_type: Set(item.link_type.and_then(link_type_name)),
                object_id: Set(item.object_id.map(|v| v as i64)),
                sort_order: Set(item.sort_order.map(|v| v as i64)),
                is_open_new_tab: Set(item.is_open_new_tab),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        let children = navigation_items_of(&self.state.db, row.id).await?;
        Ok(Response::new(navigation_proto(row, children)))
    }

    async fn update(
        &self,
        request: Request<sitev1::UpdateNavigationRequest>,
    ) -> Result<Response<sitev1::Navigation>, Status> {
        let req = request.into_inner();
        let row = navigations::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("navigation"))?;
        let mut a: navigations::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(Some(v));
            }
            if let Some(v) = data.is_active {
                a.is_active = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        let children = navigation_items_of(&self.state.db, row.id).await?;
        Ok(Response::new(navigation_proto(row, children)))
    }

    async fn delete(
        &self,
        request: Request<sitev1::DeleteNavigationRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(sitev1::delete_navigation_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        navigations::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── NavigationItem ───────────────────────────────────────────────────

pub struct NavigationItemServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::navigation_item_service_server::NavigationItemService for NavigationItemServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<sitev1::ListNavigationItemResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            navigation_items::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(sitev1::ListNavigationItemResponse {
            items: rows.into_iter().map(navigation_item_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<sitev1::GetNavigationItemRequest>,
    ) -> Result<Response<sitev1::NavigationItem>, Status> {
        let req = request.into_inner();
        let Some(sitev1::get_navigation_item_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = navigation_items::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("navigation item"))?;
        Ok(Response::new(navigation_item_proto(row)))
    }

    async fn create(
        &self,
        request: Request<sitev1::CreateNavigationItemRequest>,
    ) -> Result<Response<sitev1::NavigationItem>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = navigation_items::ActiveModel {
            navigation_id: Set(data.navigation_id.map(|v| v as i64)),
            title: Set(data.title),
            description: Set(data.description),
            icon: Set(data.icon),
            url: Set(data.url),
            link_type: Set(data.link_type.and_then(link_type_name)),
            object_id: Set(data.object_id.map(|v| v as i64)),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_open_new_tab: Set(data.is_open_new_tab),
            created_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(navigation_item_proto(row)))
    }

    async fn update(
        &self,
        request: Request<sitev1::UpdateNavigationItemRequest>,
    ) -> Result<Response<sitev1::NavigationItem>, Status> {
        let req = request.into_inner();
        let row = navigation_items::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("navigation item"))?;
        let mut a: navigation_items::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.title {
                a.title = Set(Some(v));
            }
            if let Some(v) = data.url {
                a.url = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(navigation_item_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<sitev1::DeleteNavigationItemRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(sitev1::delete_navigation_item_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        navigation_items::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
