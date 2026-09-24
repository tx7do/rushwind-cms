//! The misc domain faces: OrgUnit, Position, LoginPolicy, ContentModel,
//! MediaAsset, Task, InternalMessage×3 — full CRUD over the golden
//! schema (write paths per the reference's repo layer; MediaAsset
//! carries its full CRUD — the reference's media library face).

use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
    TransactionTrait,
};
use tonic::{Request, Response, Status};

use crate::data::misc_repo as repo;
use crate::state::{bad, db_status, not_found, ts_from_proto, ts_to_proto, AppState};
use store::entities::{
    content_model_translations, content_models, field_definition_translations, field_definitions,
    internal_message_categories, internal_message_recipients, internal_messages, media_assets,
    sys_login_policies, sys_org_units, sys_positions, sys_tasks,
};
use store::paging::fetch_paged;

use proto::proto::authentication::service::v1 as authv1;
use proto::proto::content::service::v1 as contentv1;
use proto::proto::identity::service::v1 as identityv1;
use proto::proto::internal_message::service::v1 as imv1;
use proto::proto::media::service::v1 as mediav1;
use proto::proto::task::service::v1 as taskv1;

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

/// ComputeTreePath — the reference's materialized path convention.
fn compute_tree_path(parent_path: &str, node_id: i64) -> String {
    if parent_path.is_empty() {
        return "/".to_string();
    }
    let mut parent = parent_path.to_string();
    if !parent.ends_with('/') {
        parent.push('/');
    }
    format!("{parent}{node_id}/")
}

/// protojson Timestamp → the timestamptz column form.
fn ts_to_column(
    v: Option<pbjson_types::Timestamp>,
) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    v.and_then(|v| ts_from_proto(&v))
}

// ── OrgUnit ──────────────────────────────────────────────────────────

fn org_unit_proto(r: sys_org_units::Model) -> identityv1::OrgUnit {
    identityv1::OrgUnit {
        id: Some(r.id as u32),
        name: Some(r.name),
        code: r.code,
        path: r.path,
        status: Some(1),
        sort_order: r.sort_order.map(|v| v as u32),
        leader_id: r.leader_id.map(|v| v as u32),
        tenant_id: r.tenant_id.map(|v| v as u32),
        remark: r.remark,
        description: r.description,
        r#type: Some(1),
        ..Default::default()
    }
}

pub struct OrgUnitServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::org_unit_service_server::OrgUnitService for OrgUnitServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::ListOrgUnitResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_org_units::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(identityv1::ListOrgUnitResponse {
            items: rows.into_iter().map(org_unit_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<identityv1::CountOrgUnitResponse>, Status> {
        let total = sys_org_units::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(identityv1::CountOrgUnitResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<identityv1::GetOrgUnitRequest>,
    ) -> Result<Response<identityv1::OrgUnit>, Status> {
        let req = request.into_inner();
        let Some(identityv1::get_org_unit_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::org_units_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(org_unit_proto(row)))
    }

    async fn create(
        &self,
        request: Request<identityv1::CreateOrgUnitRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let mut a = sys_org_units::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            name: Set(data.name.unwrap_or_default()),
            code: Set(data.code),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            leader_id: Set(data.leader_id.map(|v| v as i64)),
            description: Set(data.description),
            remark: Set(data.remark),
            external_id: Set(data.external_id),
            is_legal_entity: Set(data.is_legal_entity),
            registration_number: Set(data.registration_number),
            tax_id: Set(data.tax_id),
            legal_entity_org_id: Set(data.legal_entity_org_id.map(|v| v as i64)),
            address: Set(data.address),
            phone: Set(data.phone),
            email: Set(data.email),
            timezone: Set(data.timezone),
            country: Set(data.country),
            latitude: Set(data.latitude),
            longitude: Set(data.longitude),
            start_at: Set(ts_to_column(data.start_at)),
            end_at: Set(ts_to_column(data.end_at)),
            contact_user_id: Set(data.contact_user_id.map(|v| v as i64)),
            created_by: Set(data.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(id) = data.id {
            a.id = Set(id as i64);
        }
        if let Some(name) = data
            .status
            .and_then(enum_name!(identityv1::org_unit::Status))
        {
            a.status = Set(name);
        }
        if let Some(name) = data.r#type.and_then(enum_name!(identityv1::org_unit::Type)) {
            a.r#type = Set(name);
        }
        let row = repo::insert_org_units(&txn, a).await?;
        // The tree path is derived (parent's path + own id) and written
        // back — the reference computes it in the same transaction.
        let parent_path = match data.parent_id {
            Some(pid) => repo::org_units_by_id(&txn, pid as i64)
                .await?
                .path
                .unwrap_or_default(),
            None => String::new(),
        };
        let new_id = row.id;
        let mut a: sys_org_units::ActiveModel = row.into();
        a.path = Set(Some(compute_tree_path(&parent_path, new_id)));
        repo::update_org_units(&txn, a).await?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<identityv1::UpdateOrgUnitRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 不存在则创建
        if req.allow_missing.unwrap_or(false)
            && repo::org_units_by_id(&self.state.db, req.id as i64)
                .await
                .is_err()
        {
            let mut create_data = data;
            create_data.created_by = create_data.updated_by;
            create_data.updated_by = None;
            Self {
                state: Arc::clone(&self.state),
            }
            .create(Request::new(identityv1::CreateOrgUnitRequest {
                data: Some(create_data),
            }))
            .await?;
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        let mut a = sys_org_units::ActiveModel {
            id: Set(req.id as i64),
            ..Default::default()
        };
        if let Some(v) = data.name {
            a.name = Set(v);
        }
        if let Some(v) = data.code {
            a.code = Set(Some(v));
        }
        if let Some(name) = data
            .status
            .and_then(enum_name!(identityv1::org_unit::Status))
        {
            a.status = Set(name);
        }
        if let Some(name) = data.r#type.and_then(enum_name!(identityv1::org_unit::Type)) {
            a.r#type = Set(name);
        }
        if let Some(v) = data.parent_id {
            a.parent_id = Set(Some(v as i64));
        }
        if let Some(v) = data.sort_order {
            a.sort_order = Set(Some(v as i64));
        }
        if let Some(v) = data.leader_id {
            a.leader_id = Set(Some(v as i64));
        }
        if let Some(v) = data.description {
            a.description = Set(Some(v));
        }
        if let Some(v) = data.remark {
            a.remark = Set(Some(v));
        }
        if let Some(v) = data.external_id {
            a.external_id = Set(Some(v));
        }
        if let Some(v) = data.is_legal_entity {
            a.is_legal_entity = Set(Some(v));
        }
        if let Some(v) = data.registration_number {
            a.registration_number = Set(Some(v));
        }
        if let Some(v) = data.tax_id {
            a.tax_id = Set(Some(v));
        }
        if let Some(v) = data.legal_entity_org_id {
            a.legal_entity_org_id = Set(Some(v as i64));
        }
        if let Some(v) = data.address {
            a.address = Set(Some(v));
        }
        if let Some(v) = data.phone {
            a.phone = Set(Some(v));
        }
        if let Some(v) = data.email {
            a.email = Set(Some(v));
        }
        if let Some(v) = data.timezone {
            a.timezone = Set(Some(v));
        }
        if let Some(v) = data.country {
            a.country = Set(Some(v));
        }
        if let Some(v) = data.latitude {
            a.latitude = Set(Some(v));
        }
        if let Some(v) = data.longitude {
            a.longitude = Set(Some(v));
        }
        if data.start_at.is_some() {
            a.start_at = Set(ts_to_column(data.start_at));
        }
        if data.end_at.is_some() {
            a.end_at = Set(ts_to_column(data.end_at));
        }
        if let Some(v) = data.contact_user_id {
            a.contact_user_id = Set(Some(v as i64));
        }
        a.updated_at = Set(Some(store::now()));
        sys_org_units::Entity::update_many()
            .set(a)
            .filter(sys_org_units::Column::Id.eq(req.id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<identityv1::DeleteOrgUnitRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(identityv1::delete_org_unit_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        // 级联删除全部子孙节点
        let mut ids = repo::org_unit_descendant_ids(&self.state.db, id as i64).await?;
        ids.push(id as i64);
        sys_org_units::Entity::delete_many()
            .filter(sys_org_units::Column::Id.is_in(ids))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
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

pub struct PositionServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::position_service_server::PositionService for PositionServiceImpl {
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
        let row = repo::positions_by_id(&self.state.db, id as i64).await?;
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
            && repo::positions_by_id(&self.state.db, req.id as i64)
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
        repo::delete_positions(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── LoginPolicy ──────────────────────────────────────────────────────

fn login_policy_proto(r: sys_login_policies::Model) -> authv1::LoginPolicy {
    authv1::LoginPolicy {
        id: Some(r.id as u32),
        target_id: r.target_id.map(|v| v as u32),
        method: r.method.map(|_| 1),
        value: r.value,
        reason: r.reason,
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct LoginPolicyServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl authv1::login_policy_service_server::LoginPolicyService for LoginPolicyServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<authv1::ListLoginPolicyResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_login_policies::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(authv1::ListLoginPolicyResponse {
            items: rows.into_iter().map(login_policy_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<authv1::GetLoginPolicyRequest>,
    ) -> Result<Response<authv1::LoginPolicy>, Status> {
        let req = request.into_inner();
        let Some(authv1::get_login_policy_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::login_policies_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(login_policy_proto(row)))
    }

    async fn create(
        &self,
        request: Request<authv1::CreateLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let mut a = sys_login_policies::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            target_id: Set(data.target_id.map(|v| v as i64)),
            value: Set(data.value),
            reason: Set(data.reason),
            created_by: Set(data.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(name) = data.r#type.and_then(enum_name!(authv1::login_policy::Type)) {
            a.r#type = Set(Some(name));
        }
        if let Some(name) = data
            .method
            .and_then(enum_name!(authv1::login_policy::Method))
        {
            a.method = Set(Some(name));
        }
        a.insert(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<authv1::UpdateLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 不存在则创建
        if req.allow_missing.unwrap_or(false)
            && repo::login_policies_by_id(&self.state.db, req.id as i64)
                .await
                .is_err()
        {
            let mut create_data = data;
            create_data.created_by = create_data.updated_by;
            create_data.updated_by = None;
            Self {
                state: Arc::clone(&self.state),
            }
            .create(Request::new(authv1::CreateLoginPolicyRequest {
                data: Some(create_data),
            }))
            .await?;
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        let mut a = <sys_login_policies::ActiveModel as std::default::Default>::default();
        if let Some(v) = data.target_id {
            a.target_id = Set(Some(v as i64));
        }
        if let Some(name) = data.r#type.and_then(enum_name!(authv1::login_policy::Type)) {
            a.r#type = Set(Some(name));
        }
        if let Some(name) = data
            .method
            .and_then(enum_name!(authv1::login_policy::Method))
        {
            a.method = Set(Some(name));
        }
        if let Some(v) = data.value {
            a.value = Set(Some(v));
        }
        if let Some(v) = data.reason {
            a.reason = Set(Some(v));
        }
        a.updated_at = Set(Some(store::now()));
        sys_login_policies::Entity::update_many()
            .set(a)
            .filter(sys_login_policies::Column::Id.eq(req.id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<authv1::DeleteLoginPolicyRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(authv1::delete_login_policy_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_login_policies(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── ContentModel ─────────────────────────────────────────────────────

fn content_model_proto(r: content_models::Model) -> contentv1::ContentModel {
    contentv1::ContentModel {
        id: Some(r.id as u32),
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// The varchar field type → its proto ordinal.
fn field_type_enum(s: &str) -> Option<i32> {
    contentv1::field_definition::Type::from_str_name(s).map(|e| e as i32)
}

/// The proto ordinal → the varchar field type the golden schema stores.
fn field_type_name(v: i32) -> Option<String> {
    contentv1::field_definition::Type::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn json_to_options(v: &sea_orm::JsonValue) -> std::collections::HashMap<String, String> {
    v.as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

/// The typed-JSON relation config ↔ its golden-JSON keys
/// (filter_category_id carries the reference's omitempty).
fn relation_config_from_json(v: &sea_orm::JsonValue) -> Option<contentv1::RelationConfig> {
    let obj = v.as_object()?;
    Some(contentv1::RelationConfig {
        target_entity_type: obj
            .get("target_entity_type")
            .and_then(|x| x.as_str())
            .map(str::to_owned),
        allow_cross_tenant: obj.get("allow_cross_tenant").and_then(|x| x.as_bool()),
        filter_category_id: obj
            .get("filter_category_id")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32),
    })
}

fn relation_config_to_json(rc: &contentv1::RelationConfig) -> sea_orm::JsonValue {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "target_entity_type".into(),
        serde_json::json!(rc.target_entity_type.clone().unwrap_or_default()),
    );
    obj.insert(
        "allow_cross_tenant".into(),
        serde_json::json!(rc.allow_cross_tenant.unwrap_or(false)),
    );
    if let Some(v) = rc.filter_category_id.filter(|v| *v != 0) {
        obj.insert("filter_category_id".into(), serde_json::json!(v));
    }
    serde_json::Value::Object(obj)
}

fn field_translation_proto(
    r: field_definition_translations::Model,
) -> contentv1::FieldDefinitionTranslation {
    contentv1::FieldDefinitionTranslation {
        id: Some(r.id as u32),
        field_definition_id: r.field_definition_id.map(|v| v as u32),
        language_code: r.language_code,
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        created_by: r.created_by.map(|v| v as u32),
        ..Default::default()
    }
}

fn field_definition_proto(
    r: field_definitions::Model,
    translations: Vec<field_definition_translations::Model>,
) -> contentv1::FieldDefinition {
    contentv1::FieldDefinition {
        id: Some(r.id as u32),
        content_model_id: r.content_model_id.map(|v| v as u32),
        name: r.name,
        r#type: r.r#type.as_deref().and_then(field_type_enum),
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        is_required: r.is_required,
        validation_regex: r.validation_regex,
        options: r.options.as_ref().map(json_to_options).unwrap_or_default(),
        relation_config: r
            .relation_config
            .as_ref()
            .and_then(relation_config_from_json),
        sort_order: r.sort_order.map(|v| v as u32),
        translations: translations
            .into_iter()
            .map(field_translation_proto)
            .collect(),
        created_by: r.created_by.map(|v| v as u32),
        ..Default::default()
    }
}

/// The model's field definitions + their translations, recreated
/// wholesale (the reference's replace-fields convention).
async fn batch_create_fields<C: ConnectionTrait>(
    db: &C,
    model_id: i64,
    fields: &[contentv1::FieldDefinition],
) -> Result<(), Status> {
    for f in fields {
        if f.name.as_deref().unwrap_or_default().is_empty() {
            return Err(bad("field definition name is required"));
        }
        let mut a = field_definitions::ActiveModel {
            content_model_id: Set(Some(model_id)),
            name: Set(f.name.clone()),
            label: Set(f.label.clone()),
            description: Set(f.description.clone()),
            placeholder: Set(f.placeholder.clone()),
            is_required: Set(f.is_required),
            validation_regex: Set(f.validation_regex.clone()),
            sort_order: Set(f.sort_order.map(|v| v as i64)),
            created_by: Set(f.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(name) = f.r#type.and_then(field_type_name) {
            a.r#type = Set(Some(name));
        }
        if !f.options.is_empty() {
            a.options = Set(serde_json::to_value(&f.options).ok());
        }
        if let Some(rc) = &f.relation_config {
            a.relation_config = Set(Some(relation_config_to_json(rc)));
        }
        let created = repo::insert_field_definitions(db, a).await?;
        for tr in &f.translations {
            repo::insert_field_definition_translations(
                db,
                field_definition_translations::ActiveModel {
                    field_definition_id: Set(Some(created.id)),
                    language_code: Set(tr.language_code.clone()),
                    label: Set(tr.label.clone()),
                    description: Set(tr.description.clone()),
                    placeholder: Set(tr.placeholder.clone()),
                    created_by: Set(tr.created_by.map(|v| v as i64)),
                    created_at: Set(Some(store::now())),
                    ..Default::default()
                },
            )
            .await?;
        }
    }
    Ok(())
}

async fn batch_create_model_translations<C: ConnectionTrait>(
    db: &C,
    model_id: i64,
    translations: &[contentv1::ContentModelTranslation],
) -> Result<(), Status> {
    for tr in translations {
        if tr.language_code.as_deref().unwrap_or_default().is_empty() {
            continue;
        }
        repo::insert_content_model_translations(
            db,
            content_model_translations::ActiveModel {
                content_model_id: Set(Some(model_id)),
                language_code: Set(tr.language_code.clone()),
                name: Set(tr.name.clone()),
                description: Set(tr.description.clone()),
                created_by: Set(tr.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok(())
}

pub struct ContentModelServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::content_model_service_server::ContentModelService for ContentModelServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListContentModelResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            content_models::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(contentv1::ListContentModelResponse {
            items: rows.into_iter().map(content_model_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_content_model_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::content_models_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::CountContentModelResponse>, Status> {
        let total = content_models::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(contentv1::CountContentModelResponse {
            count: total,
        }))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreateContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = repo::insert_content_models(
            &txn,
            content_models::ActiveModel {
                tenant_id: Set(data.tenant_id.map(|v| v as i64)),
                name: Set(data.name),
                code: Set(data.code),
                description: Set(data.description),
                sort_order: Set(data.sort_order.map(|v| v as i64)),
                created_by: Set(data.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
        batch_create_fields(&txn, row.id, &data.fields).await?;
        batch_create_model_translations(&txn, row.id, &data.translations).await?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdateContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = repo::content_models_by_id(&txn, req.id as i64).await?;
        let mut a: content_models::ActiveModel = row.into();
        if let Some(v) = data.name {
            a.name = Set(Some(v));
        }
        if let Some(v) = data.description {
            a.description = Set(Some(v));
        }
        if let Some(v) = data.sort_order {
            a.sort_order = Set(Some(v as i64));
        }
        a.updated_at = Set(Some(store::now()));
        let row = repo::update_content_models(&txn, a).await?;
        // 字段定义与翻译整体替换（对位 replace-fields / replace-translations）
        let old_field_ids = repo::content_model_field_ids(&txn, req.id as i64).await?;
        repo::delete_field_definition_translations_of(&txn, &old_field_ids).await?;
        repo::delete_field_definitions_of(&txn, req.id as i64).await?;
        batch_create_fields(&txn, req.id as i64, &data.fields).await?;
        repo::delete_content_model_translations_of(&txn, req.id as i64).await?;
        batch_create_model_translations(&txn, req.id as i64, &data.translations).await?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeleteContentModelRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_content_model_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let model_id = id as i64;
        // 级联清理：字段翻译 → 字段定义 → 模型翻译 → 模型
        let field_ids = repo::content_model_field_ids(&txn, model_id).await?;
        repo::delete_field_definition_translations_of(&txn, &field_ids).await?;
        repo::delete_field_definitions_of(&txn, model_id).await?;
        repo::delete_content_model_translations_of(&txn, model_id).await?;
        repo::content_models_by_id(&txn, model_id).await?;
        content_models::Entity::delete_by_id(model_id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn list_field_definitions(
        &self,
        request: Request<contentv1::ListFieldDefinitionsRequest>,
    ) -> Result<Response<contentv1::ListFieldDefinitionsResponse>, Status> {
        let req = request.into_inner();
        if req.content_model_id == 0 {
            return Err(bad("invalid parameter"));
        }
        let rows = repo::field_definitions_of(&self.state.db, req.content_model_id as i64).await?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations = repo::field_definition_translations_of(&self.state.db, r.id).await?;
            items.push(field_definition_proto(r, translations));
        }
        let total = items.len() as u64;
        Ok(Response::new(contentv1::ListFieldDefinitionsResponse {
            items,
            total,
        }))
    }
}

// ── MediaAsset ───────────────────────────────────────────────────────

fn asset_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "ASSET_TYPE_IMAGE",
            2 => "ASSET_TYPE_VIDEO",
            3 => "ASSET_TYPE_DOCUMENT",
            4 => "ASSET_TYPE_AUDIO",
            5 => "ASSET_TYPE_ARCHIVE",
            100 => "ASSET_TYPE_OTHER",
            _ => return None,
        }
        .to_string(),
    )
}

fn processing_status_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "PROCESSING_STATUS_UPLOADING",
            2 => "PROCESSING_STATUS_PROCESSING",
            3 => "PROCESSING_STATUS_COMPLETED",
            4 => "PROCESSING_STATUS_FAILED",
            _ => return None,
        }
        .to_string(),
    )
}

fn media_asset_proto(r: media_assets::Model) -> mediav1::MediaAsset {
    mediav1::MediaAsset {
        id: Some(r.id as u32),
        filename: r.filename.clone(),
        r#type: r.r#type.clone().map(|_| 1),
        mime_type: r.mime_type.clone(),
        size: r.size.map(|v| v as u64),
        storage_path: r.storage_path.clone(),
        url: r.url.clone(),
        width: r.width.map(|v| v as u32),
        height: r.height.map(|v| v as u32),
        duration: r.duration.map(|v| v as u32),
        alt_text: r.alt_text.clone(),
        title: r.title.clone(),
        caption: r.caption.clone(),
        processing_status: r.processing_status.clone().map(|_| 1),
        processing_error: r.processing_error.clone(),
        file_hash: r.file_hash.clone(),
        file_id: r.file_id.map(|v| v as u32),
        reference_count: r.reference_count.map(|v| v as u32),
        is_private: r.is_private,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct MediaAssetServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl mediav1::media_asset_service_server::MediaAssetService for MediaAssetServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<mediav1::ListMediaAssetResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            media_assets::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(mediav1::ListMediaAssetResponse {
            items: rows.into_iter().map(media_asset_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<mediav1::GetMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let id = req.id;
        let row = repo::media_assets_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn create(
        &self,
        request: Request<mediav1::CreateMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::insert_media_assets(
            &self.state.db,
            media_assets::ActiveModel {
                filename: Set(data.filename),
                r#type: Set(data.r#type.and_then(asset_type_name)),
                mime_type: Set(data.mime_type),
                size: Set(data.size.map(|v| v as i64)),
                storage_path: Set(data.storage_path),
                url: Set(data.url),
                width: Set(data.width.map(|v| v as i64)),
                height: Set(data.height.map(|v| v as i64)),
                duration: Set(data.duration.map(|v| v as i64)),
                alt_text: Set(data.alt_text),
                title: Set(data.title),
                caption: Set(data.caption),
                processing_status: Set(data.processing_status.and_then(processing_status_name)),
                processing_error: Set(data.processing_error),
                file_hash: Set(data.file_hash),
                file_id: Set(data.file_id.map(|v| v as i64)),
                folder_id: Set(data.folder_id.map(|v| v as i64)),
                is_private: Set(data.is_private),
                created_by: Set(data.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn update(
        &self,
        request: Request<mediav1::UpdateMediaAssetRequest>,
    ) -> Result<Response<mediav1::MediaAsset>, Status> {
        let req = request.into_inner();
        let row = repo::media_assets_by_id(&self.state.db, req.id as i64).await?;
        let mut a: media_assets::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.r#type {
                a.r#type = Set(asset_type_name(v));
            }
            if let Some(v) = data.filename {
                a.filename = Set(Some(v));
            }
            if let Some(v) = data.mime_type {
                a.mime_type = Set(Some(v));
            }
            if let Some(v) = data.size {
                a.size = Set(Some(v as i64));
            }
            if let Some(v) = data.storage_path {
                a.storage_path = Set(Some(v));
            }
            if let Some(v) = data.url {
                a.url = Set(Some(v));
            }
            if let Some(v) = data.width {
                a.width = Set(Some(v as i64));
            }
            if let Some(v) = data.height {
                a.height = Set(Some(v as i64));
            }
            if let Some(v) = data.duration {
                a.duration = Set(Some(v as i64));
            }
            if let Some(v) = data.alt_text {
                a.alt_text = Set(Some(v));
            }
            if let Some(v) = data.title {
                a.title = Set(Some(v));
            }
            if let Some(v) = data.caption {
                a.caption = Set(Some(v));
            }
            if let Some(v) = data.processing_status {
                a.processing_status = Set(processing_status_name(v));
            }
            if let Some(v) = data.processing_error {
                a.processing_error = Set(Some(v));
            }
            if let Some(v) = data.file_hash {
                a.file_hash = Set(Some(v));
            }
            if let Some(v) = data.file_id {
                a.file_id = Set(Some(v as i64));
            }
            if let Some(v) = data.folder_id {
                a.folder_id = Set(Some(v as i64));
            }
            if let Some(v) = data.is_private {
                a.is_private = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = repo::update_media_assets(&self.state.db, a).await?;
        Ok(Response::new(media_asset_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<mediav1::DeleteMediaAssetRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let id = match req.query_by {
            Some(mediav1::delete_media_asset_request::QueryBy::Id(id)) => id as i64,
            _ => return Err(bad("query_by required")),
        };
        repo::delete_media_assets(&self.state.db, id).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Task ─────────────────────────────────────────────────────────────

fn task_proto(r: sys_tasks::Model) -> taskv1::Task {
    taskv1::Task {
        id: Some(r.id as u32),
        r#type: r.r#type.clone().map(|_| 1),
        type_name: r.type_name.clone(),
        cron_spec: r.cron_spec.clone(),
        enable: r.enable,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct TaskServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl taskv1::task_service_server::TaskService for TaskServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<taskv1::ListTaskResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_tasks::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(taskv1::ListTaskResponse {
            items: rows.into_iter().map(task_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<taskv1::CountTaskResponse>, Status> {
        let total = sys_tasks::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(taskv1::CountTaskResponse { count: total }))
    }
}

// ── InternalMessage cluster (minimal reads) ──────────────────────────

fn internal_message_proto(r: internal_messages::Model) -> imv1::InternalMessage {
    imv1::InternalMessage {
        id: Some(r.id as u32),
        title: r.title.clone(),
        content: r.content.clone(),
        sender_id: Some(r.sender_id as u32),
        category_id: r.category_id.map(|v| v as u32),
        status: r.status.clone().map(|_| 1),
        r#type: r.r#type.clone().map(|_| 1),
        created_at: r.created_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct InternalMessageServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_service_server::InternalMessageService for InternalMessageServiceImpl {
    async fn list_message(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_messages::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageResponse {
            items: rows.into_iter().map(internal_message_proto).collect(),
            total,
        }))
    }

    async fn get_message(
        &self,
        request: Request<imv1::GetInternalMessageRequest>,
    ) -> Result<Response<imv1::InternalMessage>, Status> {
        let req = request.into_inner();
        let Some(imv1::get_internal_message_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = internal_messages::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("internal message"))?;
        Ok(Response::new(internal_message_proto(row)))
    }
}

fn internal_message_category_proto(
    r: internal_message_categories::Model,
) -> imv1::InternalMessageCategory {
    imv1::InternalMessageCategory {
        id: Some(r.id as u32),
        name: r.name,
        code: r.code,
        icon_url: r.icon_url,
        sort_order: r.sort_order.map(|v| v as u32),
        is_enabled: r.is_enabled,
        parent_id: r.parent_id.map(|v| v as u32),
        depth: r.depth,
        path: r.path,
        tenant_id: r.tenant_id.map(|v| v as u32),
        created_by: r.created_by.map(|v| v as u32),
        updated_by: r.updated_by.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// 沿 parent 链上溯校验 parentId 不构成环（对位 ensureParentAcyclic）。
async fn ensure_parent_acyclic(
    db: &impl ConnectionTrait,
    self_id: u32,
    parent_id: u32,
) -> Result<(), Status> {
    if parent_id == 0 {
        return Ok(());
    }
    if parent_id == self_id {
        return Err(bad("parent cannot be itself"));
    }
    let mut current = parent_id;
    while current != 0 {
        let row = internal_message_categories::Entity::find_by_id(current as i64)
            .one(db)
            .await
            .map_err(db_status)?;
        let Some(row) = row else {
            // 父节点不存在，交由外键约束兜底
            return Ok(());
        };
        match row.parent_id {
            Some(pid) if pid as u32 == self_id => {
                return Err(bad("parent cannot be itself or its descendant"));
            }
            Some(pid) => current = pid as u32,
            None => return Ok(()),
        }
    }
    Ok(())
}

pub struct InternalMessageCategoryServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_category_service_server::InternalMessageCategoryService
    for InternalMessageCategoryServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageCategoryResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_message_categories::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageCategoryResponse {
            items: rows
                .into_iter()
                .map(internal_message_category_proto)
                .collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::CountInternalMessageCategoryResponse>, Status> {
        let total = internal_message_categories::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(imv1::CountInternalMessageCategoryResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<imv1::GetInternalMessageCategoryRequest>,
    ) -> Result<Response<imv1::InternalMessageCategory>, Status> {
        let req = request.into_inner();
        let Some(imv1::get_internal_message_category_request::QueryBy::Id(id)) = req.query_by
        else {
            return Err(bad("query_by required"));
        };
        let row = repo::internal_message_categories_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(internal_message_category_proto(row)))
    }

    async fn create(
        &self,
        request: Request<imv1::CreateInternalMessageCategoryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        ensure_parent_acyclic(
            &self.state.db,
            data.id.unwrap_or(0),
            data.parent_id.unwrap_or(0),
        )
        .await?;
        let mut a = internal_message_categories::ActiveModel {
            tenant_id: Set(data.tenant_id.map(|v| v as i64)),
            name: Set(data.name),
            code: Set(data.code),
            icon_url: Set(data.icon_url),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_enabled: Set(data.is_enabled),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
            depth: Set(data.depth),
            path: Set(data.path),
            created_by: Set(data.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(id) = data.id {
            a.id = Set(id as i64);
        }
        a.insert(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn update(
        &self,
        request: Request<imv1::UpdateInternalMessageCategoryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // 不存在则创建
        if req.allow_missing.unwrap_or(false)
            && repo::internal_message_categories_by_id(&self.state.db, req.id as i64)
                .await
                .is_err()
        {
            let mut create_data = data;
            create_data.created_by = create_data.updated_by;
            create_data.updated_by = None;
            Self {
                state: Arc::clone(&self.state),
            }
            .create(Request::new(imv1::CreateInternalMessageCategoryRequest {
                data: Some(create_data),
            }))
            .await?;
            return Ok(Response::new(pbjson_types::Empty {}));
        }
        // 变更 parentId 时校验不成环
        if let Some(parent_id) = data.parent_id {
            ensure_parent_acyclic(&self.state.db, req.id, parent_id).await?;
        }
        let mut a = <internal_message_categories::ActiveModel as std::default::Default>::default();
        if let Some(v) = data.name {
            a.name = Set(Some(v));
        }
        if let Some(v) = data.code {
            a.code = Set(Some(v));
        }
        if let Some(v) = data.icon_url {
            a.icon_url = Set(Some(v));
        }
        if let Some(v) = data.sort_order {
            a.sort_order = Set(Some(v as i64));
        }
        if let Some(v) = data.is_enabled {
            a.is_enabled = Set(Some(v));
        }
        if let Some(v) = data.parent_id {
            a.parent_id = Set(Some(v as i64));
        }
        if let Some(v) = data.depth {
            a.depth = Set(Some(v));
        }
        if let Some(v) = data.path {
            a.path = Set(Some(v));
        }
        a.updated_at = Set(Some(store::now()));
        internal_message_categories::Entity::update_many()
            .set(a)
            .filter(internal_message_categories::Column::Id.eq(req.id as i64))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn delete(
        &self,
        request: Request<imv1::DeleteInternalMessageCategoryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(imv1::delete_internal_message_category_request::QueryBy::Id(id)) = req.query_by
        else {
            return Err(bad("query_by required"));
        };
        // 级联删除全部子孙节点
        let mut ids =
            repo::internal_message_category_descendant_ids(&self.state.db, id as i64).await?;
        ids.push(id as i64);
        internal_message_categories::Entity::delete_many()
            .filter(internal_message_categories::Column::Id.is_in(ids))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

fn internal_message_recipient_proto(
    r: internal_message_recipients::Model,
) -> imv1::InternalMessageRecipient {
    imv1::InternalMessageRecipient {
        id: Some(r.id as u32),
        ..Default::default()
    }
}

pub struct InternalMessageRecipientServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl imv1::internal_message_recipient_service_server::InternalMessageRecipientService
    for InternalMessageRecipientServiceImpl
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<imv1::ListInternalMessageRecipientResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            internal_message_recipients::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(imv1::ListInternalMessageRecipientResponse {
            items: rows
                .into_iter()
                .map(internal_message_recipient_proto)
                .collect(),
            total,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_tree_path ───────────────────────────────────────────
    // The reference's materialized path convention: a root row's path
    // is exactly "/", every other row appends `id/` to its parent.

    #[test]
    fn tree_path_root_is_the_bare_slash() {
        // The parent path is irrelevant for the empty form — a root
        // insert passes "" regardless of the node id.
        assert_eq!(compute_tree_path("", 1), "/");
        assert_eq!(compute_tree_path("", 42), "/");
    }

    #[test]
    fn tree_path_appends_the_id_under_the_parent() {
        assert_eq!(compute_tree_path("/", 2), "/2/");
        assert_eq!(compute_tree_path("/1/", 2), "/1/2/");
        assert_eq!(compute_tree_path("/1", 2), "/1/2/");
        assert_eq!(compute_tree_path("/1/2/", 3), "/1/2/3/");
    }

    #[test]
    fn tree_path_normalizes_a_missing_parent_trailing_slash() {
        // A bare segment still becomes a proper directory form.
        assert_eq!(compute_tree_path("1", 2), "1/2/");
    }
}
