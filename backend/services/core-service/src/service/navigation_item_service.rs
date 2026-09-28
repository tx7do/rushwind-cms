//! The navigation item service (navigation_item_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tonic::{Request, Response, Status};

use crate::data::navigation_item_repo;
use crate::service::navigation_service::navigation_item_proto;
use crate::service::site_service::link_type_name;
use crate::state::{bad, db_status, not_found, AppState};
use store::entities::navigation_items;
use store::paging::fetch_paged;

use proto::proto::site::service::v1 as sitev1;

// ── NavigationItem ───────────────────────────────────────────────────

pub struct NavigationItemService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl sitev1::navigation_item_service_server::NavigationItemService for NavigationItemService {
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
        let row = navigation_item_repo::navigation_items_by_id(&self.state.db, id as i64).await?;
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
        navigation_item_repo::delete_navigation_items(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
