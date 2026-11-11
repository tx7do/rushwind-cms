//! The stats service — dashboard aggregates over the live tables
//! (counts, weekly deltas, day-bucketed trends, top-liked posts,
//! login activity split).

use std::sync::Arc;

use sea_orm::{ConnectionTrait, PaginatorTrait};
use tonic::{Request, Response, Status};

use crate::state::{db_status, AppState};
use store::entities::{comments, posts, sys_users};

use proto::proto::stats::service::v1 as statsv1;

async fn count_of<E>(db: &sea_orm::DatabaseConnection) -> Result<i64, Status>
where
    E: sea_orm::EntityTrait,
    E::Model: sea_orm::FromQueryResult + Send + Sync + 'static,
{
    <E as sea_orm::EntityTrait>::find()
        .count(db)
        .await
        .map(|n| n as i64)
        .map_err(db_status)
}

pub struct StatsService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl statsv1::stats_service_server::StatsService for StatsService {
    async fn get_dashboard_overview(
        &self,
        _request: Request<statsv1::GetDashboardOverviewRequest>,
    ) -> Result<Response<statsv1::GetDashboardOverviewResponse>, Status> {
        let db = &self.state.db;
        let week_ago = (chrono::Utc::now() - chrono::Duration::days(7)).fixed_offset();

        let scalar = |sql: &str| {
            let sql = sql.to_string();
            async move {
                let row = db
                    .query_one_raw(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        sql,
                    ))
                    .await
                    .map_err(db_status)?
                    .ok_or_else(|| Status::internal("aggregate row missing"))?;
                row.try_get::<i64>("", "n")
                    .map_err(|e| Status::internal(format!("count column: {e}")))
            }
        };

        let user_count = count_of::<sys_users::Entity>(db).await?;
        let post_count = count_of::<posts::Entity>(db).await?;
        let comment_count = count_of::<comments::Entity>(db).await?;
        let interaction_count =
            scalar("SELECT COALESCE(SUM(count), 0)::bigint AS n FROM interaction_counters").await?;
        let new_user_count_week = scalar(&format!(
            "SELECT COUNT(*)::bigint AS n FROM sys_users WHERE created_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        let new_post_count_week = scalar(&format!(
            "SELECT COUNT(*)::bigint AS n FROM posts WHERE created_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        let new_comment_count_week = scalar(&format!(
            "SELECT COUNT(*)::bigint AS n FROM comments WHERE created_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        let new_like_count_week = scalar(&format!(
            "SELECT COALESCE(SUM(count), 0)::bigint AS n FROM interaction_counters \
             WHERE metric = 1 AND updated_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        // 媒体域与内容工作台的概览卡（2026-11 契约批次扩展）。
        let media_asset_count = scalar("SELECT COUNT(*)::bigint AS n FROM media_assets").await?;
        let media_storage_bytes =
            scalar("SELECT COALESCE(SUM(size), 0)::bigint AS n FROM media_assets").await?;
        let new_media_count_week = scalar(&format!(
            "SELECT COUNT(*)::bigint AS n FROM media_assets WHERE created_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        let new_media_storage_bytes_week = scalar(&format!(
            "SELECT COALESCE(SUM(size), 0)::bigint AS n FROM media_assets \
             WHERE created_at >= '{}'",
            week_ago.format("%Y-%m-%d %H:%M:%S%:z")
        ))
        .await?;
        // DB 存枚举名串：comments 用短名（PENDING），posts 用全限定名
        // （POST_STATUS_DRAFT）——两种口径各自与写入侧对齐。
        let pending_comment_count =
            scalar("SELECT COUNT(*)::bigint AS n FROM comments WHERE status = 'PENDING'").await?;
        let draft_post_count =
            scalar("SELECT COUNT(*)::bigint AS n FROM posts WHERE status = 'POST_STATUS_DRAFT'")
                .await?;
        let scheduled_post_count = scalar(
            "SELECT COUNT(*)::bigint AS n FROM posts WHERE status = 'POST_STATUS_SCHEDULED'",
        )
        .await?;

        Ok(Response::new(statsv1::GetDashboardOverviewResponse {
            user_count,
            post_count,
            comment_count,
            interaction_count,
            new_user_count_week,
            new_post_count_week,
            new_comment_count_week,
            new_like_count_week,
            media_asset_count,
            media_storage_bytes,
            new_media_count_week,
            new_media_storage_bytes_week,
            pending_comment_count,
            draft_post_count,
            scheduled_post_count,
        }))
    }

    async fn get_content_trend(
        &self,
        request: Request<statsv1::GetContentTrendRequest>,
    ) -> Result<Response<statsv1::GetContentTrendResponse>, Status> {
        let days = request.into_inner().days.clamp(1, 90);
        let db = &self.state.db;
        let mut users = Vec::new();
        let mut posts_trend = Vec::new();
        let mut comments_trend = Vec::new();
        for i in (0..days).rev() {
            let day = (chrono::Utc::now() - chrono::Duration::days(i as i64)).date_naive();
            let bounds = (
                day.and_hms_opt(0, 0, 0).unwrap(),
                (day + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            );
            let day_prefix = day.format("%Y-%m-%d").to_string();
            let rows = db
                .query_all_raw(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        "SELECT \
               (SELECT COUNT(*) FROM sys_users WHERE created_at::date::text = '{p}')::bigint AS u, \
                (SELECT COUNT(*) FROM posts WHERE created_at::date::text = '{p}')::bigint AS po, \
                (SELECT COUNT(*) FROM comments WHERE created_at::date::text = '{p}')::bigint AS c",
                        p = day_prefix
                    ),
                ))
                .await
                .map_err(db_status)?;
            let _ = bounds;
            if let Some(row) = rows.first() {
                let u = row.try_get::<i64>("", "u").unwrap_or(0);
                let po = row.try_get::<i64>("", "po").unwrap_or(0);
                let c = row.try_get::<i64>("", "c").unwrap_or(0);
                let date = day.format("%Y-%m-%d").to_string();
                users.push(statsv1::DailyCount {
                    date: date.clone(),
                    value: u,
                });
                posts_trend.push(statsv1::DailyCount {
                    date: date.clone(),
                    value: po,
                });
                comments_trend.push(statsv1::DailyCount { date, value: c });
            }
        }
        Ok(Response::new(statsv1::GetContentTrendResponse {
            users,
            posts: posts_trend,
            comments: comments_trend,
        }))
    }

    async fn get_interaction_stats(
        &self,
        request: Request<statsv1::GetInteractionStatsRequest>,
    ) -> Result<Response<statsv1::GetInteractionStatsResponse>, Status> {
        let top_n = request.into_inner().top_n.clamp(1, 50) as u64;
        let db = &self.state.db;

        let totals = db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT \
                   COALESCE(SUM(count) FILTER (WHERE metric = 1), 0)::bigint AS likes, \
                   COALESCE(SUM(count) FILTER (WHERE metric = 2), 0)::bigint AS watches, \
                   COALESCE(SUM(count) FILTER (WHERE metric = 3), 0)::bigint AS views \
                 FROM interaction_counters",
            ))
            .await
            .map_err(db_status)?
            .ok_or_else(|| Status::internal("totals row missing"))?;
        let total_likes = totals.try_get::<i64>("", "likes").unwrap_or(0);
        let total_watches = totals.try_get::<i64>("", "watches").unwrap_or(0);
        // 浏览口径（metric = 3 VIEW）——2026-11 契约批次扩展。
        let total_views = totals.try_get::<i64>("", "views").unwrap_or(0);

        let top = db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT target_id::bigint AS post_id, COALESCE(SUM(count), 0)::bigint AS likes \
                     FROM interaction_counters WHERE metric = 1 \
                     GROUP BY target_id ORDER BY likes DESC LIMIT {top_n}"
                ),
            ))
            .await
            .map_err(db_status)?;
        let top_liked_posts = top
            .into_iter()
            .map(|r| statsv1::InteractionTopItem {
                target_id: r.try_get::<i64>("", "post_id").unwrap_or(0),
                title: r.try_get::<String>("", "title").unwrap_or_default(),
                like_count: r.try_get::<i64>("", "likes").unwrap_or(0),
            })
            .collect();

        // 浏览量降序 TOP N（真实 PV）——标题位与点赞榜同水位（title 列在
        // post_translations，多语言标题回填随统计深化批次一并落地）。
        let top_viewed = db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT target_id::bigint AS post_id, COALESCE(SUM(count), 0)::bigint AS views \
                     FROM interaction_counters WHERE metric = 3 \
                     GROUP BY target_id ORDER BY views DESC LIMIT {top_n}"
                ),
            ))
            .await
            .map_err(db_status)?;
        let top_viewed_posts: Vec<statsv1::InteractionTopItem> = top_viewed
            .into_iter()
            .map(|r| statsv1::InteractionTopItem {
                target_id: r.try_get::<i64>("", "post_id").unwrap_or(0),
                title: String::new(),
                like_count: r.try_get::<i64>("", "views").unwrap_or(0),
            })
            .collect();

        Ok(Response::new(statsv1::GetInteractionStatsResponse {
            top_liked_posts,
            total_likes,
            total_watches,
            top_viewed_posts,
            total_views,
        }))
    }

    async fn get_login_activity(
        &self,
        request: Request<statsv1::GetLoginActivityRequest>,
    ) -> Result<Response<statsv1::GetLoginActivityResponse>, Status> {
        let days = request.into_inner().days.clamp(1, 90);
        let db = &self.state.db;
        let rows = db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT created_at::date::text AS d, \
                       COUNT(*) FILTER (WHERE success)::bigint AS s, \
                       COUNT(*) FILTER (WHERE NOT success)::bigint AS f, \
                       COUNT(DISTINCT user_id) FILTER (WHERE success)::bigint AS u \
                     FROM sys_login_audit_logs \
                     WHERE created_at >= now() - interval '{days} days' \
                     GROUP BY d ORDER BY d"
                ),
            ))
            .await
            .map_err(db_status)?;
        let (mut success, mut failed, mut active_users) = (Vec::new(), Vec::new(), Vec::new());
        for r in rows {
            let day = r.try_get::<String>("", "d").unwrap_or_default();
            success.push(statsv1::DailyCount {
                date: day.clone(),
                value: r.try_get::<i64>("", "s").unwrap_or(0),
            });
            failed.push(statsv1::DailyCount {
                date: day.clone(),
                value: r.try_get::<i64>("", "f").unwrap_or(0),
            });
            // 当日活跃用户数 = 当日成功登录的去重用户（2026-11 契约批次扩展）。
            active_users.push(statsv1::DailyCount {
                date: day,
                value: r.try_get::<i64>("", "u").unwrap_or(0),
            });
        }
        Ok(Response::new(statsv1::GetLoginActivityResponse {
            success,
            failed,
            active_users,
        }))
    }
}
