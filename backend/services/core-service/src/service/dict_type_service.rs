//! The dict-type service (dict_type_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, AppState};
use store::entities::sys_dict_types;
use store::paging::fetch_paged;

// ── DictType ─────────────────────────────────────────────────────────

fn dict_type_proto(r: sys_dict_types::Model) -> proto::proto::dict::service::v1::DictType {
    proto::proto::dict::service::v1::DictType {
        id: Some(r.id as u32),
        type_code: r.type_code,
        type_name: r.type_name,
        is_enabled: r.is_enabled,
        sort_order: r.sort_order.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
        created_at: r.created_at.and_then(crate::state::ts_to_proto),
        updated_at: r.updated_at.and_then(crate::state::ts_to_proto),
        ..Default::default()
    }
}

pub struct DictTypeService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::dict_type_service_server::DictTypeService
    for DictTypeService
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::ListDictTypeResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_dict_types::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(
            proto::proto::dict::service::v1::ListDictTypeResponse {
                items: rows.into_iter().map(dict_type_proto).collect(),
                total,
            },
        ))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::CountDictTypeResponse>, Status> {
        let total = sys_dict_types::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(
            proto::proto::dict::service::v1::CountDictTypeResponse { count: total },
        ))
    }

    async fn get(
        &self,
        request: Request<proto::proto::dict::service::v1::GetDictTypeRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::DictType>, Status> {
        let req = request.into_inner();
        let row = match req.query_by {
            Some(proto::proto::dict::service::v1::get_dict_type_request::QueryBy::Id(id)) => {
                sys_dict_types::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
            }
            Some(proto::proto::dict::service::v1::get_dict_type_request::QueryBy::Code(code)) => {
                sys_dict_types::Entity::find()
                    .filter(sys_dict_types::Column::TypeCode.eq(code))
                    .one(&self.state.db)
                    .await
            }
            None => return Err(bad("query_by required")),
        }
        .map_err(db_status)?
        .ok_or_else(|| not_found("dict type"))?;
        Ok(Response::new(dict_type_proto(row)))
    }

    async fn create(
        &self,
        request: Request<proto::proto::dict::service::v1::CreateDictTypeRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_dict_types::ActiveModel {
            type_code: Set(Some(data.type_code.clone().unwrap_or_default())),
            type_name: Set(Some(data.type_name.clone().unwrap_or_default())),
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

    async fn update(
        &self,
        request: Request<proto::proto::dict::service::v1::UpdateDictTypeRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_dict_types::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("dict type"))?;
        let mut a: sys_dict_types::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.type_code {
                a.type_code = Set(Some(v));
            }
            if let Some(v) = data.type_name {
                a.type_name = Set(Some(v));
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
        request: Request<proto::proto::dict::service::v1::DeleteDictTypeRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        sys_dict_types::Entity::delete_many()
            .filter(sys_dict_types::Column::Id.is_in(req.ids.into_iter().map(|v| v as i64)))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
