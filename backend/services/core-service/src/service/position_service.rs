//! The position service — the position CRUD over the golden schema
//! (position_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::position_repo;
use crate::service::org_unit_service::ts_to_column;
use crate::state::{bad, db_status, AppState};
use store::entities::sys_positions;
use store::paging::fetch_paged;

use proto::proto::identity::service::v1 as identityv1;

/// The proto enum ordinal → the varchar value name the golden schema
/// stores (the EnumTypeConverter name-map behavior; unknown ordinals
/// stay unset).
macro_rules! enum_name {
    ($ty:ty) => {
        |v: i32| -> Option<String> {
            <$ty as TryFrom<i32>>::try_from(v)
                .ok()
                .map(|e| e.as_str_name().to_string())
        }
    };
}

// ── Position ─────────────────────────────────────────────────────────

fn position_proto(r: sys_positions::Model) -> identityv1::Position {
    identityv1::Position {
        id: Some(r.id as u32),
        name: Some(r.name),
        code: Some(r.code),
        headcount: Some(r.headcount as u32),
        sort_order: r.sort_order.map(|v| v as u32),
        status: Some(1),
        remark: r.remark,
        description: r.description,
        job_family: r.job_family,
        job_grade: r.job_grade,
        level: r.level,
        is_key_position: Some(r.is_key_position),
        tenant_id: r.tenant_id.map(|v| v as u32),
        ..Default::default()
    }
}

pub struct PositionService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::position_service_server::PositionService for PositionService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListPositionResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_positions::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListPositionResponse {
            items: rows.into_iter().map(position_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountPositionResponse>, Status> {
        let total = sys_positions::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountPositionResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetPositionRequest>,
    ) -> Result<Response<identityv1::Position>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_position_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = position_repo::positions_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(position_proto(row)))
    }

    async fn create(
        &self,
        request: Request<identityv1::CreatePositionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let mut a = sys_positions::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            name: Set(data.name.unwrap_or_default()),
            code: Set(data.code.unwrap_or_default()),
            org_unit_id: Set(data.org_unit_id.unwrap_or(0) as i64),
            // 非空设置：缺失时落 0（GetReportsToPositionId 语义）
            reports_to_position_id: Set(Some(data.reports_to_position_id.unwrap_or(0) as i64)),
            // headcount 恒置 0（对位 SetHeadcount(0)）
            headcount: Set(0),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            job_family: Set(data.job_family),
            job_grade: Set(data.job_grade),
            level: Set(data.level),
            description: Set(data.description),
            remark: Set(data.remark),
            start_at: Set(ts_to_column(data.start_at)),
            end_at: Set(ts_to_column(data.end_at)),
            created_by: Set(data.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(id) = data.id {
            a.id = Set(id as i64);
        }
        if let Some(name) = data
            .status
            .and_then(enum_name!(identityv1::position::Status))
        {
            a.status = Set(name);
        }
        if let Some(name) = data.r#type.and_then(enum_name!(identityv1::position::Type)) {
            a.r#type = Set(name);
        }
        if let Some(v) = data.is_key_position {
            a.is_key_position = Set(v);
        }
        a.insert(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdatePositionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 不存在则创建
        if req.allow_missing.unwrap_or(false)
            && position_repo::positions_by_id(&self.state.db, req.id as i64)
                .await
                .is_err()
        {
            let mut create_data = data;
            create_data.created_by = create_data.updated_by;
            create_data.updated_by = None;
            Self {
                state: Arc::clone(&self.state),
            }
            .create(Request::new(identityv1::CreatePositionRequest {
                data: Some(create_data),
            }))
            .await?;
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        let mut a = <sys_positions::ActiveModel as std::default::Default>::default();
        if let Some(v) = data.name {
            a.name = Set(v);
        }
        if let Some(v) = data.code {
            a.code = Set(v);
        }
        if let Some(v) = data.org_unit_id {
            a.org_unit_id = Set(v as i64);
        }
        if let Some(v) = data.reports_to_position_id {
            a.reports_to_position_id = Set(Some(v as i64));
        }
        if let Some(v) = data.sort_order {
            a.sort_order = Set(Some(v as i64));
        }
        if let Some(name) = data
            .status
            .and_then(enum_name!(identityv1::position::Status))
        {
            a.status = Set(name);
        }
        if let Some(name) = data.r#type.and_then(enum_name!(identityv1::position::Type)) {
            a.r#type = Set(name);
        }
        if let Some(v) = data.job_family {
            a.job_family = Set(Some(v));
        }
        if let Some(v) = data.job_grade {
            a.job_grade = Set(Some(v));
        }
        if let Some(v) = data.level {
            a.level = Set(Some(v));
        }
        if let Some(v) = data.is_key_position {
            a.is_key_position = Set(v);
        }
        if let Some(v) = data.description {
            a.description = Set(Some(v));
        }
        if let Some(v) = data.remark {
            a.remark = Set(Some(v));
        }
        if data.start_at.is_some() {
            a.start_at = Set(ts_to_column(data.start_at));
        }
        if data.end_at.is_some() {
            a.end_at = Set(ts_to_column(data.end_at));
        }
        a.updated_at = Set(Some(store::now()));
        sys_positions::Entity::update_many()
            .set(a)
            .filter(sys_positions::Column::Id.eq(req.id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeletePositionRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(identityv1::delete_position_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        position_repo::delete_positions(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}
