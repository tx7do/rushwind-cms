//! The `post` repository (post_repo.go).

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::state::StatusResult;
use store::entities::{
    post_categories, post_tags,
    post_translations, posts,
};


pub async fn post_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<posts::Model> {
    posts::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("post"))
}

pub async fn post_by_code(db: &DatabaseConnection, code: &str) -> StatusResult<posts::Model> {
    posts::Entity::find()
        .filter(posts::Column::Code.eq(code))
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("post"))
}

pub async fn post_relations(
    db: &DatabaseConnection,
    post_id: i64,
) -> StatusResult<(Vec<post_translations::Model>, Vec<i64>, Vec<i64>)> {
    let translations = post_translations::Entity::find()
        .filter(post_translations::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(crate::db_status)?;
    let category_ids = post_categories::Entity::find()
        .filter(post_categories::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(crate::db_status)?
        .into_iter()
        .map(|r| r.category_id)
        .collect();
    let tag_ids = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(crate::db_status)?
        .into_iter()
        .map(|r| r.tag_id)
        .collect();
    Ok((translations, category_ids, tag_ids))
}

pub async fn insert_post(
    db: &DatabaseConnection,
    a: posts::ActiveModel,
) -> StatusResult<posts::Model> {
    a.insert(db).await.map_err(crate::db_status)
}

pub async fn update_post(
    db: &DatabaseConnection,
    a: posts::ActiveModel,
) -> StatusResult<posts::Model> {
    a.update(db).await.map_err(crate::db_status)
}

pub async fn delete_post(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    posts::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}


