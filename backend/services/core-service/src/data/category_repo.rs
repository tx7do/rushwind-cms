//! The `category` repository (category_repo.go).

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

use crate::state::StatusResult;
use store::entities::categories;

pub async fn category_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<categories::Model> {
    categories::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("category"))
}

pub async fn insert_category(
    db: &DatabaseConnection,
    a: categories::ActiveModel,
) -> StatusResult<categories::Model> {
    a.insert(db).await.map_err(crate::db_status)
}

pub async fn update_category(
    db: &DatabaseConnection,
    a: categories::ActiveModel,
) -> StatusResult<categories::Model> {
    a.update(db).await.map_err(crate::db_status)
}

pub async fn delete_category(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    categories::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
