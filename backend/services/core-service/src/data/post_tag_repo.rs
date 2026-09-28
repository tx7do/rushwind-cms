//! The `post_tag` relation repository (post_tag_repo.go).

use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

use crate::state::StatusResult;
use store::entities::post_tags;

pub async fn link_post_tag(db: &DatabaseConnection, post_id: i64, tag_id: i64) -> StatusResult<()> {
    post_tags::ActiveModel {
        post_id: Set(post_id),
        tag_id: Set(tag_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(crate::db_status)?;
    Ok(())
}
