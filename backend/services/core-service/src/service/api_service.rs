//! The api service — the sys-apis registry CRUD plus the route-table
//! sync upsert by operation id (api_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::sys_apis;
use store::paging::fetch_paged;

use proto::proto::permission::service::v1 as permissionv1;

fn api_proto(r: sys_apis::Model) -> permissionv1::Api {
    permissionv1::Api {
        id: Some(r.id as u32),
        operation: r.operation,
        path: r.path,
        method: r.method,
        module: r.module,
        module_description: r.module_description,
        description: r.description,
        scope: None,
        status: Some(1),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct ApiService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::api_service_server::ApiService for ApiService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListApiResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_apis::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListApiResponse {
            items: rows.into_iter().map(api_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountApiResponse>, Status> {
        let total = sys_apis::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountApiResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetApiRequest>,
    ) -> Result<Response<permissionv1::Api>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_api_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = api_repo::apis_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(api_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_apis::ActiveModel {
            operation: Set(Some(data.operation.clone().unwrap_or_default())),
            path: Set(Some(data.path.clone().unwrap_or_default())),
            method: Set(Some(data.method.clone().unwrap_or_default())),
            module: Set(data.module),
            module_description: Set(data.module_description),
            description: Set(data.description),
            status: Set("ON".to_string()),
            tenant_id: Set(Some(0)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<permissionv1::UpdateApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_apis::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("api"))?;
        let mut a: sys_apis::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.description {
                a.description = Set(Some(v));
            }
            if let Some(v) = data.module_description {
                a.module_description = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<permissionv1::DeleteApiRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_api_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        api_repo::delete_apis(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
    async fn sync_apis(
        &self,
        request: Request<permissionv1::SyncApisRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        // Upsert by operation: insert missing, refresh path/method of
        // existing (the BFF ships the route table it generated).
        let now = store::now();
        for api in req.apis {
            match api_repo::apis_by_operation(
                &self.state.db,
                &api.operation.clone().unwrap_or_default(),
            )
            .await?
            {
                Some(row) => {
                    let mut a: sys_apis::ActiveModel = row.into();
                    a.path = Set(api.path.clone());
                    a.method = Set(api.method.clone());
                    a.updated_at = Set(Some(now));
                    api_repo::update_apis(&self.state.db, a).await?;
                }
                None => {
                    api_repo::insert_apis(
                        &self.state.db,
                        sys_apis::ActiveModel {
                            operation: Set(Some(api.operation.clone().unwrap_or_default())),
                            path: Set(Some(api.path.clone().unwrap_or_default())),
                            method: Set(Some(api.method.clone().unwrap_or_default())),
                            module: Set(api.module.clone()),
                            description: Set(api.description.clone()),
                            status: Set("ON".to_string()),
                            tenant_id: Set(Some(0)),
                            created_at: Set(Some(now)),
                            updated_at: Set(Some(now)),
                            ..Default::default()
                        },
                    )
                    .await?;
                }
            }
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
use crate::data::api_repo;
