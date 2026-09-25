//! The `post_category` relation repository (post_category_repo.go).

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, Set,
};

use crate::state::StatusResult;
use store::entities::post_categories;


pub async fn link_post_category(
    db: &DatabaseConnection,
    post_id: i64,
    category_id: i64,
) -> StatusResult<()> {
    post_categories::ActiveModel {
        post_id: Set(post_id),
        category_id: Set(category_id),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(crate::db_status)?;
    Ok(())
}
