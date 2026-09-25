//! The `page` repository (page_repo.go).

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait,
};

use crate::state::StatusResult;
use store::entities::pages;


pub async fn page_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<pages::Model> {
    pages::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("page"))
}

pub async fn insert_page(
    db: &DatabaseConnection,
    a: pages::ActiveModel,
) -> StatusResult<pages::Model> {
    a.insert(db).await.map_err(crate::db_status)
}

pub async fn update_page(
    db: &DatabaseConnection,
    a: pages::ActiveModel,
) -> StatusResult<pages::Model> {
    a.update(db).await.map_err(crate::db_status)
}

pub async fn delete_page(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    pages::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
