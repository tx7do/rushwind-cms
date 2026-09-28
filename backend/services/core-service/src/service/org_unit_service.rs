//! The org-unit service — the org-tree CRUD over the golden schema;
//! the materialized path walk follows the reference's ComputeTreePath
//! (org_unit_service.go).

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, ts_from_proto, AppState};
use store::entities::sys_org_units;
use store::paging::fetch_paged;

use crate::data::org_unit_repo;
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
pub(crate) fn ts_to_column(
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

pub struct OrgUnitService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl identityv1::org_unit_service_server::OrgUnitService for OrgUnitService {
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
        let row = org_unit_repo::org_units_by_id(&self.state.db, id as i64).await?;
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
        let row = org_unit_repo::insert_org_units(&txn, a).await?;
        // The tree path is derived (parent's path + own id) and written
        // back — the reference computes it in the same transaction.
        let parent_path = match data.parent_id {
            Some(pid) => org_unit_repo::org_units_by_id(&txn, pid as i64)
                .await?
                .path
                .unwrap_or_default(),
            None => String::new(),
        };
        let new_id = row.id;
        let mut a: sys_org_units::ActiveModel = row.into();
        a.path = Set(Some(compute_tree_path(&parent_path, new_id)));
        org_unit_repo::update_org_units(&txn, a).await?;
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
            && org_unit_repo::org_units_by_id(&self.state.db, req.id as i64)
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
        let mut ids = org_unit_repo::org_unit_descendant_ids(&self.state.db, id as i64).await?;
        ids.push(id as i64);
        sys_org_units::Entity::delete_many()
            .filter(sys_org_units::Column::Id.is_in(ids))
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

#[cfg(test)]
mod tests {
    use super::compute_tree_path;

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
