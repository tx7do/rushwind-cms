//! The interaction service — likes/watches over the interaction
//! counter table (the ledger writes), mirroring the reference's
//! interaction_service.go.

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::post_repo;
use crate::service::context::operator_of;
use crate::state::{bad, db_status, AppState};
use store::entities::{comment_likes, interaction_counters, post_likes, post_watches};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;
use proto::proto::interaction::service::v1 as interactionv1;

// ── Interaction ──────────────────────────────────────────────────────

/// The metric column value of a counter row (LIKE=1 / WATCH=2 style,
/// the interaction proto's enum numbers).
pub(crate) fn metric_num(metric: i32) -> i16 {
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

pub struct InteractionService {
    pub state: Arc<AppState>,
}

/// Like a post (the C-side user ledger row + counter bump, idempotent).
async fn like_post(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    post_id: i64,
    on: bool,
) -> Result<bool, Status> {
    use sea_orm::ActiveModelTrait;
    use store::entities::post_likes;
    let existing = post_likes::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(post_likes::Column::UserId.eq(user_id))
                .add(post_likes::Column::PostId.eq(post_id)),
        )
        .one(db)
        .await
        .map_err(db_status)?;
    let changed = match (&existing, on) {
        (None, true) => {
            post_likes::ActiveModel {
                user_id: Set(Some(user_id)),
                post_id: Set(Some(post_id)),
                tenant_id: Set(Some(0)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(db_status)?;
            true
        }
        (Some(row), false) => {
            post_likes::Entity::delete_by_id(row.id)
                .exec(db)
                .await
                .map_err(db_status)?;
            true
        }
        _ => false,
    };
    if changed {
        bump_counter(db, 0, "", post_id, 1, if on { 1 } else { -1 }).await?;
    }
    Ok(on)
}

/// Watch/unwatch mirrors the like ledger.
async fn watch_post(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    post_id: i64,
    on: bool,
) -> Result<bool, Status> {
    use sea_orm::ActiveModelTrait;
    use store::entities::post_watches;
    let existing = post_watches::Entity::find()
        .filter(
            sea_orm::Condition::all()
                .add(post_watches::Column::UserId.eq(user_id))
                .add(post_watches::Column::PostId.eq(post_id)),
        )
        .one(db)
        .await
        .map_err(db_status)?;
    let changed = match (&existing, on) {
        (None, true) => {
            post_watches::ActiveModel {
                user_id: Set(Some(user_id)),
                post_id: Set(Some(post_id)),
                tenant_id: Set(Some(0)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(db_status)?;
            true
        }
        (Some(row), false) => {
            post_watches::Entity::delete_by_id(row.id)
                .exec(db)
                .await
                .map_err(db_status)?;
            true
        }
        _ => false,
    };
    if changed {
        bump_counter(db, 0, "", post_id, 2, if on { 1 } else { -1 }).await?;
    }
    Ok(on)
}

#[async_trait::async_trait]
impl interactionv1::interaction_service_server::InteractionService for InteractionService {
    async fn like(
        &self,
        request: Request<interactionv1::LikeRequest>,
    ) -> Result<Response<interactionv1::LikeResponse>, Status> {
        let user_id = operator_of(&request)?;
        let req = request.into_inner();
        let liked = like_post(&self.state.db, user_id, req.target_id as i64, true).await?;
        let count = read_counter(&self.state.db, "", req.target_id as i64, 1).await;
        Ok(Response::new(interactionv1::LikeResponse {
            liked,
            like_count: count as i32,
        }))
    }

    async fn unlike(
        &self,
        request: Request<interactionv1::LikeRequest>,
    ) -> Result<Response<interactionv1::LikeResponse>, Status> {
        let user_id = operator_of(&request)?;
        let req = request.into_inner();
        let liked = like_post(&self.state.db, user_id, req.target_id as i64, false).await?;
        let count = read_counter(&self.state.db, "", req.target_id as i64, 1).await;
        Ok(Response::new(interactionv1::LikeResponse {
            liked,
            like_count: count as i32,
        }))
    }

    async fn watch(
        &self,
        request: Request<interactionv1::WatchRequest>,
    ) -> Result<Response<interactionv1::WatchResponse>, Status> {
        let user_id = operator_of(&request)?;
        let req = request.into_inner();
        let watched = watch_post(&self.state.db, user_id, req.post_id as i64, true).await?;
        let count = read_counter(&self.state.db, "", req.post_id as i64, 2).await;
        Ok(Response::new(interactionv1::WatchResponse {
            watched,
            watch_count: count as i32,
        }))
    }

    async fn unwatch(
        &self,
        request: Request<interactionv1::WatchRequest>,
    ) -> Result<Response<interactionv1::WatchResponse>, Status> {
        let user_id = operator_of(&request)?;
        let req = request.into_inner();
        let watched = watch_post(&self.state.db, user_id, req.post_id as i64, false).await?;
        let count = read_counter(&self.state.db, "", req.post_id as i64, 2).await;
        Ok(Response::new(interactionv1::WatchResponse {
            watched,
            watch_count: count as i32,
        }))
    }

    async fn get_interaction_status(
        &self,
        request: Request<interactionv1::GetInteractionStatusRequest>,
    ) -> Result<Response<interactionv1::GetInteractionStatusResponse>, Status> {
        let user_id = operator_of(&request)?;
        let req = request.into_inner();
        // Every requested id answers (liked=false/watched=false unless a
        // ledger row says otherwise).
        let mut statuses: std::collections::HashMap<u32, interactionv1::InteractionStatus> = req
            .target_ids
            .iter()
            .map(|&id| {
                (
                    id,
                    interactionv1::InteractionStatus {
                        liked: false,
                        watched: false,
                    },
                )
            })
            .collect();
        let ids: Vec<i64> = req.target_ids.iter().map(|&v| v as i64).collect();
        match req.target_type {
            1 => {
                // post: liked off post_likes, watched off post_watches
                let liked = post_likes::Entity::find()
                    .filter(post_likes::Column::UserId.eq(user_id))
                    .filter(post_likes::Column::PostId.is_in(ids.clone()))
                    .all(&self.state.db)
                    .await
                    .map_err(db_status)?;
                for row in liked {
                    if let Some(pid) = row.post_id {
                        if let Some(status) = statuses.get_mut(&(pid as u32)) {
                            status.liked = true;
                        }
                    }
                }
                let watched = post_watches::Entity::find()
                    .filter(post_watches::Column::UserId.eq(user_id))
                    .filter(post_watches::Column::PostId.is_in(ids))
                    .all(&self.state.db)
                    .await
                    .map_err(db_status)?;
                for row in watched {
                    if let Some(pid) = row.post_id {
                        if let Some(status) = statuses.get_mut(&(pid as u32)) {
                            status.watched = true;
                        }
                    }
                }
            }
            2 => {
                // comment: liked off comment_likes; watching is post-only
                // so watched keeps its false init
                let liked = comment_likes::Entity::find()
                    .filter(comment_likes::Column::UserId.eq(user_id))
                    .filter(comment_likes::Column::CommentId.is_in(ids))
                    .all(&self.state.db)
                    .await
                    .map_err(db_status)?;
                for row in liked {
                    if let Some(cid) = row.comment_id {
                        if let Some(status) = statuses.get_mut(&(cid as u32)) {
                            status.liked = true;
                        }
                    }
                }
            }
            _ => return Err(bad("invalid target type")),
        }
        Ok(Response::new(interactionv1::GetInteractionStatusResponse {
            statuses,
        }))
    }

    async fn list_watched_posts(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListPostResponse>, Status> {
        let user_id = operator_of(&request)?;
        let (rows, total) = fetch_paged(
            &self.state.db,
            post_watches::Entity::find().filter(post_watches::Column::UserId.eq(user_id)),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let Some(post_id) = row.post_id else { continue };
            // A watched post that no longer resolves drops out of the
            // page instead of failing it (the reference logs and skips).
            let Ok(post) = post_repo::post_by_id(&self.state.db, post_id).await else {
                continue;
            };
            let Ok((translations, category_ids, tag_ids)) =
                post_repo::post_relations(&self.state.db, post_id).await
            else {
                continue;
            };
            items.push(crate::service::post_service::post_proto(
                post,
                translations,
                category_ids,
                tag_ids,
            ));
        }
        Ok(Response::new(contentv1::ListPostResponse { items, total }))
    }

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
