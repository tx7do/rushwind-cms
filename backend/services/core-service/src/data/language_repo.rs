//! The `language` repository (language_repo.go).

use sea_orm::{DatabaseConnection, EntityTrait};

use crate::state::{db_status, not_found_status, StatusResult};
use store::entities::sys_languages;

pub async fn languages_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> StatusResult<sys_languages::Model> {
    sys_languages::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(db_status)?
        .ok_or_else(|| not_found_status("languages"))
}

pub async fn insert_languages(
    db: &DatabaseConnection,
    a: sys_languages::ActiveModel,
) -> StatusResult<sys_languages::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.insert(db).await.map_err(db_status)
}

pub async fn update_languages(
    db: &DatabaseConnection,
    a: sys_languages::ActiveModel,
) -> StatusResult<sys_languages::Model> {
    use sea_orm::ActiveModelTrait as _;
    a.update(db).await.map_err(db_status)
}

pub async fn delete_languages(db: &DatabaseConnection, id: i64) -> StatusResult<()> {
    sys_languages::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(db_status)?;
    Ok(())
}

// ── the language-code → column-value helper ──────────────────────────

pub(crate) fn language_of(code: &Option<String>) -> &str {
    code.as_deref().unwrap_or("")
}
