//! The dict domain services: Language, DictType, DictEntry — standard
//! CRUD over the golden schema, the gRPC faces the BFFs proxy.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::dict_repo as repo;
use crate::state::{bad, db_status, not_found, AppState};
use store::entities::{sys_dict_entries, sys_dict_types, sys_languages};
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

pub struct LanguageServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::language_service_server::LanguageService
    for LanguageServiceImpl
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
        repo::delete_languages(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

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

pub struct DictTypeServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::dict_type_service_server::DictTypeService
    for DictTypeServiceImpl
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

pub struct DictEntryServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl proto::proto::dict::service::v1::dict_entry_service_server::DictEntryService
    for DictEntryServiceImpl
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
