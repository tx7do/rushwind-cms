//! The comment service — moderated CRUD (PENDING gate, the guest
//! author type, the post/page content types), mirroring the
//! reference's comment_service.go.


use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};


use crate::data::{comment_repo};
use crate::service::context::request_tenant_of;
use crate::state::{bad, db_status, forbidden, not_found, ts_to_proto, AppState};
use store::entities::{
    comments, site_settings,
};
use store::paging::fetch_paged;

use proto::proto::comment::service::v1 as commentv1;

pub(crate) fn content_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "POST" => 1,
        "PAGE" => 2,
        _ => return None,
    })
}

pub(crate) fn content_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "POST",
            2 => "PAGE",
            _ => return None,
        }
        .to_string(),
    )
}

pub(crate) fn author_type_num(name: &str) -> Option<i32> {
    Some(match name {
        "USER" => 1,
        "GUEST" => 2,
        _ => return None,
    })
}

pub(crate) fn author_type_name(v: i32) -> Option<String> {
    Some(
        match v {
            1 => "USER",
            2 => "GUEST",
            _ => return None,
        }
        .to_string(),
    )
}

pub(crate) fn comment_status_num(name: &str) -> Option<i32> {
    Some(match name {
        "PENDING" => 1,
        "APPROVED" => 2,
        "REJECTED" => 3,
        "SPAM" => 4,
        _ => return None,
    })
}

pub(crate) fn comment_status_name(v: i32) -> Option<String> {
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

pub struct CommentService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl commentv1::comment_service_server::CommentService for CommentService {
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
        let row = comment_repo::comments_by_id(&self.state.db, id as i64).await?;
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
        comment_repo::delete_comments(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
