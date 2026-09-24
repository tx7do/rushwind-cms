//! The internal-message category face — the taxonomy CRUD the
//! message flow sorts under; the acyclicity guard keeps the parent
//! chain a tree.
use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use tonic::{Request, Response, Status};

use crate::data::misc_repo as repo;
use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::internal_message_categories;
use store::paging::fetch_paged;

use proto::proto::internal_message::service::v1 as imv1;

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

