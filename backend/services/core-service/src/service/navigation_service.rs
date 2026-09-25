//! The navigation service (navigation_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::{navigation_repo};
use crate::service::site_service::{link_type_name, link_type_num, location_name, location_num};
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{navigation_items, navigations};
use store::paging::fetch_paged;

use proto::proto::site::service::v1 as sitev1;

// ── Navigation ───────────────────────────────────────────────────────

pub(crate) fn navigation_item_proto(r: navigation_items::Model) -> sitev1::NavigationItem {
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

pub struct NavigationService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::navigation_service_server::NavigationService for NavigationService {
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
        let row = navigation_repo::navigations_by_id(&self.state.db, id as i64).await?;
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
        navigation_repo::delete_navigations(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
