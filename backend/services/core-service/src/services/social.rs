//! The social domain services: Comment (moderated CRUD) and Interaction
//! (likes/watches/counters over the interaction counter table).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{comments, interaction_counters};
use store::paging::fetch_paged;

use proto::proto::comment::service::v1 as commentv1;
use proto::proto::interaction::service::v1 as interactionv1;

fn content_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "POST" => 1,
        "PAGE" => 2,
        _ => return None,
    })
}

fn content_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "POST",
            2 => "PAGE",
            _ => return None,
        }
        .to_string(),
    )
}

fn author_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "USER" => 1,
        "GUEST" => 2,
        _ => return None,
    })
}

fn comment_status_num(name: &str) -> Option<i32> {
    Some(match name {
        "PENDING" => 1,
        "APPROVED" => 2,
        "REJECTED" => 3,
        "SPAM" => 4,
        _ => return None,
    })
}

fn comment_proto(r: comments::Model) -> commentv1::Comment {
    commentv1::Comment {
        id: Some(r.id as u32),
        content_type: r.content_type.as_deref().and_then(content_type_num),
        object_id: r.object_id.map(|v| v as u32),
        content: r.content,
        author_id: r.author_id.map(|v| v as u32),
        author_name: r.author_name,
        author_email: r.author_email,
        author_url: r.author_url,
        author_type: r.author_type.as_deref().and_then(author_type_num),
        status: r.status.as_deref().and_then(comment_status_num),
        ip_address: r.ip_address,
        location: r.location,
        user_agent: r.user_agent,
        detected_language: r.detected_language,
        is_spam: r.is_spam,
        is_sticky: r.is_sticky,
        reply_to_id: r.reply_to_id.map(|v| v as u32),
        parent_id: r.parent_id.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct CommentServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl commentv1::comment_service_server::CommentService for CommentServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<commentv1::ListCommentResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            comments::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(commentv1::ListCommentResponse {
            items: rows.into_iter().map(comment_proto).collect(),
            total,
        }))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<commentv1::CountCommentResponse>, Status> {
        let total = comments::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(commentv1::CountCommentResponse {
            count: total,
        }))
    }

    async fn get(
        &self,
        request: Request<commentv1::GetCommentRequest>,
    ) -> Result<Response<commentv1::Comment>, Status> {
        let req = request.into_inner();
        let id = req.id;
        let row = comments::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("comment"))?;
        Ok(Response::new(comment_proto(row)))
    }

    async fn create(
        &self,
        request: Request<commentv1::CreateCommentRequest>,
    ) -> Result<Response<commentv1::Comment>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = comments::ActiveModel {
            content_type: Set(data.content_type.and_then(content_type_name)),
            object_id: Set(data.object_id.map(|v| v as i64)),
            content: Set(Some(data.content.unwrap_or_default())),
            author_id: Set(data.author_id.map(|v| v as i64)),
            author_name: Set(data.author_name),
            author_email: Set(data.author_email),
            author_url: Set(data.author_url),
            status: Set(Some("PENDING".to_string())),
            ip_address: Set(data.ip_address),
            user_agent: Set(data.user_agent),
            reply_to_id: Set(data.reply_to_id.map(|v| v as i64)),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            updated_at: Set(Some(store::now())),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        Ok(Response::new(comment_proto(row)))
    }

    async fn update(
        &self,
        request: Request<commentv1::UpdateCommentRequest>,
    ) -> Result<Response<commentv1::Comment>, Status> {
        let req = request.into_inner();
        let row = comments::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("comment"))?;
        let mut a: comments::ActiveModel = row.into();
        if let Some(data) = req.data {
            if let Some(v) = data.content {
                a.content = Set(Some(v));
            }
            if let Some(v) = data.is_sticky {
                a.is_sticky = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        Ok(Response::new(comment_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<commentv1::DeleteCommentRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(commentv1::delete_comment_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        comments::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Interaction ──────────────────────────────────────────────────────

/// The metric column value of a counter row (LIKE=1 / WATCH=2 style,
/// the interaction proto's enum numbers).
fn metric_num(metric: i32) -> i16 {
    match metric {
        2 => 2,
        _ => 1,
    }
}

#[allow(dead_code)] // the like/unlike/watch write paths land with the interaction phase
async fn bump_counter(
    db: &sea_orm::DatabaseConnection,
    tenant_id: i64,
    _target_type: &str,
    target_id: i64,
    metric: i16,
    delta: i64,
) -> Result<i64, Status> {
    let row = interaction_counters::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(interaction_counters::Column::TargetType.eq(1))
                .add(interaction_counters::Column::TargetId.eq(target_id))
                .add(interaction_counters::Column::Metric.eq(metric)),
        )
        .one(db)
        .await
        .map_err(db_status)?;
    match row {
        Some(row) => {
            let next = (row.count.unwrap_or(0) + delta).max(0);
            let mut a: interaction_counters::ActiveModel = row.into();
            a.count = Set(Some(next));
            a.update(db).await.map_err(db_status)?;
            Ok(next)
        }
        None => {
            if delta <= 0 {
                return Ok(0);
            }
            interaction_counters::ActiveModel {
                tenant_id: Set(Some(tenant_id)),
                target_type: Set(Some(1)),
                target_id: Set(Some(target_id)),
                metric: Set(Some(metric)),
                count: Set(Some(delta)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(db_status)?;
            Ok(delta)
        }
    }
}

async fn read_counter(
    db: &sea_orm::DatabaseConnection,
    _target_type: &str,
    target_id: i64,
    metric: i16,
) -> i64 {
    interaction_counters::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(interaction_counters::Column::TargetType.eq(1))
                .add(interaction_counters::Column::TargetId.eq(target_id))
                .add(interaction_counters::Column::Metric.eq(metric)),
        )
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|r| r.count)
        .unwrap_or(0)
}

pub struct InteractionServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl interactionv1::interaction_service_server::InteractionService for InteractionServiceImpl {
    async fn get_counts(
        &self,
        request: Request<interactionv1::GetCountsRequest>,
    ) -> Result<Response<interactionv1::GetCountsResponse>, Status> {
        let req = request.into_inner();
        let mut counts = std::collections::HashMap::new();
        for target_id in req.target_ids {
            let mut map = interactionv1::CountMap::default();
            for metric in req.metrics.clone() {
                let value =
                    read_counter(&self.state.db, "", target_id as i64, metric_num(metric)).await;
                map.counts.push(interactionv1::MetricCount {
                    metric,
                    count: value,
                });
            }
            counts.insert(target_id, map);
        }
        Ok(Response::new(interactionv1::GetCountsResponse { counts }))
    }
}

pub struct InteractionAdminServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl interactionv1::interaction_admin_service_server::InteractionAdminService
    for InteractionAdminServiceImpl
{
    async fn purge_target_interactions(
        &self,
        _request: Request<interactionv1::PurgeTargetInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeTargetInteractionsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn purge_user_interactions(
        &self,
        _request: Request<interactionv1::PurgeUserInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeUserInteractionsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reset_counter(
        &self,
        _request: Request<interactionv1::ResetCounterRequest>,
    ) -> Result<Response<interactionv1::ResetCounterResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}

impl InteractionAdminServiceImpl {
    /// The admin counters read — shared with the public face.
    pub async fn get_counts(
        &self,
        request: Request<interactionv1::GetCountsRequest>,
    ) -> Result<Response<interactionv1::GetCountsResponse>, Status> {
        <InteractionServiceImpl as interactionv1::interaction_service_server::InteractionService>::get_counts(
            &InteractionServiceImpl {
                state: Arc::clone(&self.state),
            },
            request,
        )
        .await
    }
}
