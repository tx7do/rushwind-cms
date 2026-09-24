//! ContentRepo — posts/categories/tags/pages with their translation
//! sub-tables, relation tables, and the search SQL (the data layer of
//! the content domain, mirroring the reference's
//! `internal/data/post_repo.go` family).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

use crate::state::StatusResult;
use store::entities::{
    categories, category_translations, page_translations, pages, post_categories, post_tags,
    post_translations, posts, tag_translations, tags,
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

pub async fn insert_post_translation(
    db: &DatabaseConnection,
    a: post_translations::ActiveModel,
) -> StatusResult<()> {
    a.insert(db).await.map_err(crate::db_status)?;
    Ok(())
}

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

/// The search fallback: (post_id, language, title) hits.
pub async fn search_hits(
    db: &DatabaseConnection,
    keyword: &str,
    language: &str,
    limit: i64,
    offset: i64,
) -> StatusResult<Vec<(i64, String, String)>> {
    let rows = db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT p.id AS post_id, t.language_code AS lang, t.title AS title
               FROM posts p
               JOIN post_translations t ON t.post_id = p.id
               WHERE p.status = 'POST_STATUS_PUBLISHED'
                 AND (t.title ILIKE $1 OR t.content ILIKE $1 OR t.summary ILIKE $1)
                 AND ($2 = '' OR t.language_code = $2)
               ORDER BY p.created_at DESC
               LIMIT $3 OFFSET $4"#,
            [
                keyword.to_string().into(),
                language.to_string().into(),
                limit.into(),
                offset.into(),
            ],
        ))
        .await
        .map_err(crate::db_status)?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.try_get::<i64>("", "post_id").unwrap_or(0),
                r.try_get::<String>("", "lang").unwrap_or_default(),
                r.try_get::<String>("", "title").unwrap_or_default(),
            )
        })
        .collect())
}

// ── categories ───────────────────────────────────────────────────────

pub async fn category_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<categories::Model> {
    categories::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("category"))
}

pub async fn category_translations_of(
    db: &DatabaseConnection,
    category_id: i64,
) -> StatusResult<Vec<category_translations::Model>> {
    category_translations::Entity::find()
        .filter(category_translations::Column::CategoryId.eq(category_id))
        .all(db)
        .await
        .map_err(crate::db_status)
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

pub async fn insert_category_translation(
    db: &DatabaseConnection,
    a: category_translations::ActiveModel,
) -> StatusResult<()> {
    a.insert(db).await.map_err(crate::db_status)?;
    Ok(())
}

// ── tags ─────────────────────────────────────────────────────────────

pub async fn tag_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<tags::Model> {
    tags::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("tag"))
}

pub async fn tag_translations_of(
    db: &DatabaseConnection,
    tag_id: i64,
) -> StatusResult<Vec<tag_translations::Model>> {
    tag_translations::Entity::find()
        .filter(tag_translations::Column::TagId.eq(tag_id))
        .all(db)
        .await
        .map_err(crate::db_status)
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

pub async fn insert_tag_translation(
    db: &DatabaseConnection,
    a: tag_translations::ActiveModel,
) -> StatusResult<()> {
    a.insert(db).await.map_err(crate::db_status)?;
    Ok(())
}

// ── pages ────────────────────────────────────────────────────────────

pub async fn page_by_id(db: &DatabaseConnection, id: i64) -> StatusResult<pages::Model> {
    pages::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("page"))
}

pub async fn page_translations_of(
    db: &DatabaseConnection,
    page_id: i64,
) -> StatusResult<Vec<page_translations::Model>> {
    page_translations::Entity::find()
        .filter(page_translations::Column::PageId.eq(page_id))
        .all(db)
        .await
        .map_err(crate::db_status)
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

pub async fn insert_page_translation(
    db: &DatabaseConnection,
    a: page_translations::ActiveModel,
) -> StatusResult<()> {
    a.insert(db).await.map_err(crate::db_status)?;
    Ok(())
}

// keep Condition referenced for future cross-table filters
#[allow(dead_code)]
fn _c(c: Condition) -> Condition {
    c
}
