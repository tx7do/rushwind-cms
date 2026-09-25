//! The `tag` repository (tag_repo.go).

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait,
};

use crate::state::StatusResult;
use store::entities::tags;


pub async fn tag_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<tags::Model> {
    tags::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("tag"))
}

pub async fn insert_tag(
    db: &DatabaseConnection,
    a: tags::ActiveModel,
) -> StatusResult<tags::Model> {
    a.insert(db).await.map_err(crate::db_status)
}

pub async fn update_tag(
    db: &DatabaseConnection,
    a: tags::ActiveModel,
) -> StatusResult<tags::Model> {
    a.update(db).await.map_err(crate::db_status)
}

pub async fn delete_tag(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    tags::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
