//! The dict-entry service (dict_entry_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, AppState};
use store::entities::{sys_dict_entries, sys_dict_types};
use store::paging::fetch_paged;

// ── DictEntry ────────────────────────────────────────────────────────

fn dict_entry_proto(r: sys_dict_entries::Model) -> proto::proto::dict::service::v1::DictEntry {
    proto::proto::dict::service::v1::DictEntry {
        id: Some(r.id as u32),
        type_id: r.type_id.map(|v| v as u32),
        entry_value: Some(r.entry_value),
        numeric_value: r.numeric_value,
        is_enabled: r.is_enabled,
        sort_order: r.sort_order.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
        created_at: r.created_at.and_then(crate::state::ts_to_proto),
        updated_at: r.updated_at.and_then(crate::state::ts_to_proto),
        ..Default::default()
    }
}

pub struct DictEntryService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::dict_entry_service_server::DictEntryService
    for DictEntryService
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::ListDictEntryResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_dict_entries::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(
            proto::proto::dict::service::v1::ListDictEntryResponse {
                items: rows.into_iter().map(dict_entry_proto).collect(),
                total,
            },
        ))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::CountDictEntryResponse>, Status> {
        let total = sys_dict_entries::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(
            proto::proto::dict::service::v1::CountDictEntryResponse { count: total },
        ))
    }

    async fn list_by_type_code(
        &self,
        request: Request<proto::proto::dict::service::v1::ListDictEntryByTypeCodeRequest>,
    ) -> Result<Response<proto::proto::dict::service::v1::ListDictEntryByTypeCodeResponse>, Status>
    {
        let req = request.into_inner();
        let type_id = sys_dict_types::Entity::find()
            .filter(sys_dict_types::Column::TypeCode.eq(req.type_code))
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("dict type"))?
            .id;
        let rows = sys_dict_entries::Entity::find()
            .filter(sys_dict_entries::Column::TypeId.eq(type_id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(
            proto::proto::dict::service::v1::ListDictEntryByTypeCodeResponse {
                items: rows.into_iter().map(dict_entry_proto).collect(),
            },
        ))
    }

    async fn create(
        &self,
        request: Request<proto::proto::dict::service::v1::CreateDictEntryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_dict_entries::ActiveModel {
            type_id: Set(data.type_id.map(|v| v as i64)),
            entry_value: Set(data.entry_value.unwrap_or_default()),
            numeric_value: Set(data.numeric_value),
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
        request: Request<proto::proto::dict::service::v1::UpdateDictEntryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_dict_entries::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("dict entry"))?;
        let mut a: sys_dict_entries::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.type_id {
                a.type_id = Set(Some(v as i64));
            }
            if let Some(v) = data.entry_value {
                a.entry_value = Set(v);
            }
            if let Some(v) = data.numeric_value {
                a.numeric_value = Set(Some(v));
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
        request: Request<proto::proto::dict::service::v1::DeleteDictEntryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        sys_dict_entries::Entity::delete_many()
            .filter(sys_dict_entries::Column::Id.is_in(req.ids.into_iter().map(|v| v as i64)))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
