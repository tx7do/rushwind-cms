//! The language service (language_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, AppState};
use store::entities::sys_languages;
use store::paging::fetch_paged;

fn language_proto(r: sys_languages::Model) -> proto::proto::dict::service::v1::Language {
    proto::proto::dict::service::v1::Language {
        id: Some(r.id as u32),
        language_code: r.language_code,
        language_name: r.language_name,
        native_name: r.native_name,
        is_default: r.is_default,
        is_enabled: r.is_enabled,
        sort_order: r.sort_order.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
        created_at: r.created_at.and_then(crate::state::ts_to_proto),
        updated_at: r.updated_at.and_then(crate::state::ts_to_proto),
        ..Default::default()
    }
}

pub struct LanguageService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::language_service_server::LanguageService
    for LanguageService
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::ListLanguageResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_languages::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(
            proto::proto::dict::service::v1::ListLanguageResponse {
                items: rows.into_iter().map(language_proto).collect(),
                total,
            },
        ))
    }

    async fn count(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::CountLanguageResponse>, Status> {
        let total = sys_languages::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        let _ = request;
        Ok(Response::new(
            proto::proto::dict::service::v1::CountLanguageResponse { count: total },
        ))
    }

    async fn get(
        &self,
        request: Request<proto::proto::dict::service::v1::GetLanguageRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::Language>, Status> {
        let req = request.into_inner();
        use proto::proto::dict::service::v1::get_language_request::QueryBy;
        let row = match req.query_by {
            Some(QueryBy::Id(id)) => {
                sys_languages::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(QueryBy::Code(code)) => {
                sys_languages::Entity::find()
                    .filter(sys_languages::Column::LanguageCode.eq(code))
                    .one(&self.state.db)
                    .await
            }
            None => return Err(bad("query_by required")),
        }
        .map_err(db_status)?
        .ok_or_else(|| not_found("language"))?;
        Ok(Response::new(language_proto(row)))
    }

    async fn create(
        &self,
        request: Request<proto::proto::dict::service::v1::CreateLanguageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_languages::ActiveModel {
            language_code: Set(Some(data.language_code.unwrap_or_default())),
            language_name: Set(Some(data.language_name.unwrap_or_default())),
            native_name: Set(data.native_name),
            is_default: Set(data.is_default.or(Some(false))),
            is_enabled: Set(data.is_enabled.or(Some(true))),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn batch_create(
        &self,
        request: Request<proto::proto::dict::service::v1::BatchCreateLanguagesRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        for data in req.items {
            sys_languages::ActiveModel {
                language_code: Set(Some(data.language_code.unwrap_or_default())),
                language_name: Set(Some(data.language_name.unwrap_or_default())),
                native_name: Set(data.native_name),
                is_default: Set(data.is_default.or(Some(false))),
                is_enabled: Set(data.is_enabled.or(Some(true))),
                sort_order: Set(data.sort_order.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                updated_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<proto::proto::dict::service::v1::UpdateLanguageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_languages::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("language"))?;
        let mut a: sys_languages::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.language_code {
                a.language_code = Set(Some(v));
            }
            if let Some(v) = data.language_name {
                a.language_name = Set(Some(v));
            }
            if let Some(v) = data.native_name {
                a.native_name = Set(Some(v));
            }
            if let Some(v) = data.is_default {
                a.is_default = Set(Some(v));
            }
            if let Some(v) = data.is_enabled {
                a.is_enabled = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
        }
        a.updated_at = Set(Some(store::now()));
        a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<proto::proto::dict::service::v1::DeleteLanguageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(proto::proto::dict::service::v1::delete_language_request::QueryBy::Id(id)) =
            req.query_by
        else {
            return Err(bad("query_by required"));
        };
        let id = id as i64;
        language_repo::delete_languages(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}use crate::data::{language_repo};

