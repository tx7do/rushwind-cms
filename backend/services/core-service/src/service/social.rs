//! The social domain services: Comment (moderated CRUD) and Interaction
//! (likes/watches/counters over the interaction counter table).

use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use tonic::{Request, Response, Status};

use crate::data::{content_repo, social_repo as repo};
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{
    comment_likes, comments, interaction_counters, post_likes, post_watches, site_settings,
};
use store::paging::fetch_paged;

use proto::proto::audit::service::v1 as auditv1;
use proto::proto::comment::service::v1 as commentv1;
use proto::proto::content::service::v1 as contentv1;
use proto::proto::interaction::service::v1 as interactionv1;

/// The operator user id off the gRPC metadata the BFF forwards
/// (`x-user-id` from the verified claims).
fn operator_of<T>(request: &tonic::Request<T>) -> Result<i64, Status> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .ok_or_else(|| tonic::Status::unauthenticated("user identity required"))
}

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

fn author_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "USER",
            2 => "GUEST",
            _ => return None,
        }
        .to_string(),
    )
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

fn comment_status_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "PENDING",
            2 => "APPROVED",
            3 => "REJECTED",
            4 => "SPAM",
            _ => return None,
        }
        .to_string(),
    )
}

/// The trusted inner tenant stamp: the anonymous chain's explicit
/// `x-md-global-tenant-id` (the reference's metadata channel) wins,
/// else the operator bag's `x-tenant-id`.
fn request_tenant_of<T>(request: &tonic::Request<T>) -> i64 {
    for name in ["x-md-global-tenant-id", "x-tenant-id"] {
        if let Some(v) = request.metadata().get(name).and_then(|v| v.to_str().ok()) {
            if let Ok(n) = v.parse::<i64>() {
                if n >= 0 {
                    return n;
                }
            }
        }
    }
    0
}

/// The site-settings boolean (the newest row for the key): missing row
/// or unreadable store reads as enabled — the switch only turns things
/// off explicitly (the reference's boolSetting).
async fn bool_setting(db: &sea_orm::DatabaseConnection, key: &str) -> bool {
    use sea_orm::QueryOrder as _;
    match site_settings::Entity::find()
        .filter(site_settings::Column::Key.eq(key))
        .order_by_desc(site_settings::Column::CreatedAt)
        .one(db)
        .await
    {
        Ok(Some(s)) => s
            .value
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(true),
        _ => true,
    }
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
        let row = repo::comments_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(comment_proto(row)))
    }

    async fn create(
        &self,
        request: Request<commentv1::CreateCommentRequest>,
    ) -> Result<Response<commentv1::Comment>, Status> {
        // The anonymous chain's tenant stamp rides the trusted inner
        // metadata; logged-in calls carry it in the operator bag.
        // Read before into_inner() consumes the request.
        let tenant_id = request_tenant_of(&request);
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        // The site comment policy (site_settings, the reference's
        // repo-level gate): enable_comments=false closes commenting for
        // everyone; allow_guest_comments=false bars guests only.
        if !bool_setting(&self.state.db, "enable_comments").await {
            return Err(forbidden("comments are disabled"));
        }
        if data.author_type == Some(commentv1::comment::AuthorType::Guest as i32)
            && !bool_setting(&self.state.db, "allow_guest_comments").await
        {
            return Err(forbidden("guest comments are disabled"));
        }
        let row = comments::ActiveModel {
            tenant_id: Set(Some(tenant_id)),
            content_type: Set(data.content_type.and_then(content_type_name)),
            object_id: Set(data.object_id.map(|v| v as i64)),
            content: Set(Some(data.content.unwrap_or_default())),
            author_id: Set(data.author_id.map(|v| v as i64)),
            author_name: Set(data.author_name),
            author_email: Set(data.author_email),
            author_url: Set(data.author_url),
            author_type: Set(data.author_type.and_then(author_type_name)),
            status: Set(Some(
                data.status
                    .and_then(comment_status_name)
                    .unwrap_or_else(|| "PENDING".to_string()),
            )),
            ip_address: Set(data.ip_address),
            user_agent: Set(data.user_agent),
            reply_to_id: Set(data.reply_to_id.map(|v| v as i64)),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
            created_by: Set(data.created_by.map(|v| v as i64)),
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
        repo::delete_comments(&self.state.db, id as i64).await?;
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
impl interactionv1::interaction_service_server::InteractionService for InteractionServiceImpl {
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
            let Ok(post) = content_repo::post_by_id(&self.state.db, post_id).await else {
                continue;
            };
            let Ok((translations, category_ids, tag_ids)) =
                content_repo::post_relations(&self.state.db, post_id).await
            else {
                continue;
            };
            items.push(crate::service::content::post_proto(
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

// ── Interaction admin (the清算 face) ─────────────────────────────────

/// The interaction_counters target_type column values (POST=1 /
/// COMMENT=2, the proto's enum numbers).
fn target_type_num(target_type: i32) -> Option<i16> {
    match target_type {
        1 => Some(1),
        2 => Some(2),
        _ => None,
    }
}

/// The admin清算 gate (the reference's requireAdminOperator): the
/// operator identity must ride the metadata (401 otherwise) and the
/// operator must sit in the platform/system context — tenant 0 in the
/// metadata bag (403 otherwise).
fn require_admin_operator<T>(request: &Request<T>) -> Result<(i64, i64), Status> {
    let user_id = operator_of(request)?;
    let tenant_id = request
        .metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    if tenant_id != 0 {
        return Err(forbidden("platform admin only"));
    }
    Ok((tenant_id, user_id))
}

/// The purge/reset audit row (the reference's writeAudit: one
/// OperationAuditLog with the DELETE action; a failed audit write never
/// fails the RPC).
async fn write_purge_audit(
    db: &sea_orm::DatabaseConnection,
    operator_tenant_id: i64,
    operator_user_id: i64,
    resource_type: &str,
    resource_id: &str,
    success: bool,
) {
    let entry = auditv1::OperationAuditLog {
        tenant_id: Some(operator_tenant_id as u32),
        user_id: Some(operator_user_id as u32),
        resource_type: Some(resource_type.to_string()),
        resource_id: Some(resource_id.to_string()),
        action: Some(auditv1::operation_audit_log::ActionType::Delete as i32),
        success: Some(success),
        ..Default::default()
    };
    let _ = crate::service::audit::insert_operation_audit(db, entry).await;
}

/// The target-wide ledger wipe + counter zero-out in one transaction
/// (the reference's PurgeTargetInteractions repo tail).
async fn purge_target_interactions_inner(
    db: &sea_orm::DatabaseConnection,
    target_type: i16,
    target_id: i64,
) -> Result<u32, Status> {
    let txn = db.begin().await.map_err(db_status)?;
    // The like ledgers follow the target family; the watches are
    // post-only but deleted unconditionally like the reference.
    let mut affected: u32 = match target_type {
        1 => repo::delete_post_likes_of_post(&txn, target_id).await?,
        _ => repo::delete_comment_likes_of_comment(&txn, target_id).await?,
    } as u32;
    affected += repo::delete_post_watches_of_post(&txn, target_id).await? as u32;
    // The counters zero out to row deletion: LIKE always, WATCH for
    // whatever row exists on this target.
    interaction_counters::Entity::delete_many()
        .filter(interaction_counters::Column::TargetType.eq(target_type))
        .filter(interaction_counters::Column::TargetId.eq(target_id))
        .filter(interaction_counters::Column::Metric.is_in([1i16, 2]))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    txn.commit().await.map_err(db_status)?;
    Ok(affected)
}

/// The user-wide ledger wipe (the reference's PurgeUserInteractions):
/// every ledger row of the user goes and each removed row rolls its
/// target's counter back one step, all inside one transaction (the
/// reference splits it into 200-row short transactions; the port keeps
/// the single-transaction semantics).
async fn purge_user_interactions_inner(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
) -> Result<u32, Status> {
    let txn = db.begin().await.map_err(db_status)?;
    let mut affected: u32 = 0;
    // post likes (LIKE metric, target=post)
    for post_id in repo::post_like_targets_of_user(&txn, user_id).await? {
        repo::adjust_counter(&txn, 0, 1, post_id, 1, -1).await?;
        affected += 1;
    }
    post_likes::Entity::delete_many()
        .filter(post_likes::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    // comment likes (LIKE metric, target=comment)
    for comment_id in repo::comment_like_targets_of_user(&txn, user_id).await? {
        repo::adjust_counter(&txn, 0, 2, comment_id, 1, -1).await?;
        affected += 1;
    }
    comment_likes::Entity::delete_many()
        .filter(comment_likes::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    // post watches (WATCH metric, target=post)
    for post_id in repo::post_watch_targets_of_user(&txn, user_id).await? {
        repo::adjust_counter(&txn, 0, 1, post_id, 2, -1).await?;
        affected += 1;
    }
    post_watches::Entity::delete_many()
        .filter(post_watches::Column::UserId.eq(user_id))
        .exec(&txn)
        .await
        .map_err(db_status)?;
    txn.commit().await.map_err(db_status)?;
    Ok(affected)
}

/// The counter recompute (the reference's ResetCounter): count the
/// ledger truth for (target, metric), then push the counter row to it
/// inside one transaction.
async fn reset_counter_inner(
    db: &sea_orm::DatabaseConnection,
    target_type: i32,
    target_id: i64,
    metric: i32,
) -> Result<i64, Status> {
    // The ledger recount per metric family; WATCH counts the post
    // watches whatever the target type rides in on (like the reference).
    let recount: i64 = match (metric, target_type) {
        (1, 1) => repo::count_ledger(db, 1, 1, target_id).await? as i64,
        (1, 2) => repo::count_ledger(db, 1, 2, target_id).await? as i64,
        (1, _) => return Err(bad("invalid target type")),
        (2, _) => repo::count_ledger(db, 2, 1, target_id).await? as i64,
        _ => return Err(bad("invalid metric")),
    };
    let row_type = target_type_num(target_type).unwrap_or(1);
    let metric = metric_num(metric);
    let txn = db.begin().await.map_err(db_status)?;
    let delta = recount - repo::counter_of(&txn, row_type, target_id, metric).await?;
    if delta != 0 {
        repo::adjust_counter(&txn, 0, row_type, target_id, metric, delta).await?;
    }
    txn.commit().await.map_err(db_status)?;
    Ok(recount)
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
        request: Request<interactionv1::PurgeTargetInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeTargetInteractionsResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let target_type =
            target_type_num(req.target_type).ok_or_else(|| bad("invalid target type"))?;
        let target_id = req.target_id as i64;

        let outcome = purge_target_interactions_inner(&self.state.db, target_type, target_id).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_counter",
            &target_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(
            interactionv1::PurgeTargetInteractionsResponse {
                affected_rows: outcome?,
            },
        ))
    }

    async fn purge_user_interactions(
        &self,
        request: Request<interactionv1::PurgeUserInteractionsRequest>,
    ) -> Result<Response<interactionv1::PurgeUserInteractionsResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let user_id = req.user_id as i64;

        let outcome = purge_user_interactions_inner(&self.state.db, user_id).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_user_ledger",
            &user_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(
            interactionv1::PurgeUserInteractionsResponse {
                affected_rows: outcome?,
            },
        ))
    }

    async fn reset_counter(
        &self,
        request: Request<interactionv1::ResetCounterRequest>,
    ) -> Result<Response<interactionv1::ResetCounterResponse>, Status> {
        let (op_tid, op_uid) = require_admin_operator(&request)?;
        let req = request.into_inner();
        let target_id = req.target_id as i64;

        let outcome =
            reset_counter_inner(&self.state.db, req.target_type, target_id, req.metric).await;
        write_purge_audit(
            &self.state.db,
            op_tid,
            op_uid,
            "interaction_counter_reset",
            &target_id.to_string(),
            outcome.is_ok(),
        )
        .await;
        Ok(Response::new(interactionv1::ResetCounterResponse {
            recount: outcome?,
        }))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A request carrying the given metadata headers.
    fn req_with(headers: &[(&'static str, &'static str)]) -> tonic::Request<()> {
        let mut request = tonic::Request::new(());
        for (name, value) in headers {
            request.metadata_mut().insert(*name, value.parse().unwrap());
        }
        request
    }

    // ── request_tenant_of ───────────────────────────────────────────

    #[test]
    fn tenant_global_header_wins_over_operator_bag() {
        let request = req_with(&[("x-md-global-tenant-id", "5"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 5);
    }

    #[test]
    fn tenant_falls_back_to_the_operator_bag_header() {
        let request = req_with(&[("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);
    }

    #[test]
    fn tenant_invalid_or_negative_global_falls_through() {
        // Non-numeric global → the bag header answers.
        let request = req_with(&[("x-md-global-tenant-id", "abc"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);

        // Negative global is rejected (n >= 0 gate) → the bag header.
        let request = req_with(&[("x-md-global-tenant-id", "-3"), ("x-tenant-id", "7")]);
        assert_eq!(request_tenant_of(&request), 7);

        // Negative everywhere → 0.
        let request = req_with(&[("x-md-global-tenant-id", "-3"), ("x-tenant-id", "-1")]);
        assert_eq!(request_tenant_of(&request), 0);

        // Non-numeric everywhere → 0.
        let request = req_with(&[
            ("x-md-global-tenant-id", "abc"),
            ("x-tenant-id", "not-a-number"),
        ]);
        assert_eq!(request_tenant_of(&request), 0);
    }

    #[test]
    fn tenant_missing_headers_and_explicit_zero_read_zero() {
        assert_eq!(request_tenant_of(&req_with(&[])), 0);
        assert_eq!(request_tenant_of(&req_with(&[("x-tenant-id", "abc")])), 0);
        // Zero is a legal tenant value (the platform scope).
        assert_eq!(
            request_tenant_of(&req_with(&[("x-md-global-tenant-id", "0")])),
            0
        );
        assert_eq!(request_tenant_of(&req_with(&[("x-tenant-id", "0")])), 0);
    }

    // ── enum converters ─────────────────────────────────────────────

    #[test]
    fn content_type_round_trips_and_rejects_unknowns() {
        for (name, num) in [("POST", 1), ("PAGE", 2)] {
            assert_eq!(content_type_num(name), Some(num), "{name}");
            assert_eq!(content_type_name(num).as_deref(), Some(name), "{num}");
        }
        for name in ["COMMENT", "MEDIA", ""] {
            assert_eq!(content_type_num(name), None, "{name}");
        }
        for num in [0, 3, -1, 100] {
            assert_eq!(content_type_name(num), None, "{num}");
        }
    }

    #[test]
    fn author_type_round_trips_and_rejects_unknowns() {
        for (name, num) in [("USER", 1), ("GUEST", 2)] {
            assert_eq!(author_type_num(name), Some(num), "{name}");
            assert_eq!(author_type_name(num).as_deref(), Some(name), "{num}");
        }
        for name in ["ADMIN", "SYSTEM", ""] {
            assert_eq!(author_type_num(name), None, "{name}");
        }
        for num in [0, 3, -1, 100] {
            assert_eq!(author_type_name(num), None, "{num}");
        }
    }

    #[test]
    fn comment_status_round_trips_and_rejects_unknowns() {
        for (name, num) in [
            ("PENDING", 1),
            ("APPROVED", 2),
            ("REJECTED", 3),
            ("SPAM", 4),
        ] {
            assert_eq!(comment_status_num(name), Some(num), "{name}");
            assert_eq!(comment_status_name(num).as_deref(), Some(name), "{num}");
        }
        for name in ["TRASHED", "approved", ""] {
            assert_eq!(comment_status_num(name), None, "{name}");
        }
        for num in [0, 5, -1, 100] {
            assert_eq!(comment_status_name(num), None, "{num}");
        }
    }
}
