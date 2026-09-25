//! The permission-group service (permission_group_service.go).


use std::sync::Arc;

use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};
use tonic::{Request, Response, Status};

use proto::proto::permission::service::v1 as permissionv1;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    sys_permission_groups,
};
use store::paging::fetch_paged;


// ── PermissionGroup ──────────────────────────────────────────────────

fn group_proto(r: sys_permission_groups::Model) -> permissionv1::PermissionGroup {
    permissionv1::PermissionGroup {
        id: Some(r.id as u32),
        name: Some(r.name),
        path: r.path,
        module: r.module,
        sort_order: r.sort_order.map(|v| v as u32),
        status: Some(1),
        description: r.description,
        parent_id: r.parent_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct PermissionGroupService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl permissionv1::permission_group_service_server::PermissionGroupService
    for PermissionGroupService
{
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::ListPermissionGroupResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            sys_permission_groups::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(permissionv1::ListPermissionGroupResponse {
            items: rows.into_iter().map(group_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<permissionv1::CountPermissionGroupResponse>, Status> {
        let total = sys_permission_groups::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(permissionv1::CountPermissionGroupResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<permissionv1::GetPermissionGroupRequest>,
    ) -> Result<Response<permissionv1::PermissionGroup>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::get_permission_group_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = permission_group_repo::permission_groups_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(group_proto(row)))
    }

    async fn create(
        &self,
        request: Request<permissionv1::CreatePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        sys_permission_groups::ActiveModel {
            name: Set(data.name.clone().unwrap_or_default()),
            path: Set(data.path),
            module: Set(data.module),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            status: Set("ON".to_string()),
            description: Set(data.description),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
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
        request: Request<permissionv1::UpdatePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let row = sys_permission_groups::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("permission group"))?;
        let mut a: sys_permission_groups::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.name {
                a.name = Set(v);
            }
            if let Some(v) = data.description {
                a.description = Set(Some(v));
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
        request: Request<permissionv1::DeletePermissionGroupRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(permissionv1::delete_permission_group_request::QueryBy::Id(id)) = req.query_by
        else {
            return Err(bad("query_by required"));
        };
        permission_group_repo::delete_permission_groups(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}use crate::data::{permission_group_repo};

