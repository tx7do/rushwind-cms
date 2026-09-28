//! The `post` repository (post_repo.go).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use super::language_repo::language_of;
use super::search_repo::generate_summary;
use super::search_repo::raw_chars;
use crate::state::StatusResult;
use store::entities::post_translations;

/// The client-visible fields of a post translation. A missing `summary`
/// and the `word_count` are server-derived here, never trusted from the
/// client.
pub(crate) struct PostTranslationFields {
    pub language_code: Option<String>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub original_content: Option<String>,
    pub full_path: Option<String>,
    pub seo: Option<sea_orm::JsonValue>,
    pub created_by: Option<i64>,
}

/// The two delete modes: by the translation row's own id, or by the
/// (main-row id, language code) identifier.
pub(crate) enum PostTranslationQuery {
    Id(i64),
    Identifier { post_id: i64, language_code: String },
}

// ── post translations ────────────────────────────────────────────────

pub(crate) async fn post_translation_exists(
    db: &DatabaseConnection,
    post_id: i64,
    language_code: &str,
) -> StatusResult<bool> {
    let count = post_translations::Entity::find()
        .filter(
            Condition::all()
                .add(post_translations::Column::PostId.eq(post_id))
                .add(post_translations::Column::LanguageCode.eq(language_code)),
        )
        .count(db)
        .await
        .map_err(crate::db_status)?;
    Ok(count > 0)
}

/// The (post_id, language_code) row with the smallest id — the reference
/// keeps its historic duplicate rows deterministic by taking the first.
/// A missing translation is `None`, not an error (same as the reference).
pub(crate) async fn get_post_translation(
    db: &DatabaseConnection,
    post_id: i64,
    language_code: &str,
) -> StatusResult<Option<post_translations::Model>> {
    post_translations::Entity::find()
        .filter(
            Condition::all()
                .add(post_translations::Column::PostId.eq(post_id))
                .add(post_translations::Column::LanguageCode.eq(language_code)),
        )
        .order_by_asc(post_translations::Column::Id)
        .one(db)
        .await
        .map_err(crate::db_status)
}

pub(crate) async fn create_post_translation(
    db: &DatabaseConnection,
    post_id: i64,
    f: PostTranslationFields,
) -> StatusResult<post_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    if post_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    // Summary falls back to the rule excerpt; word_count always recounts
    // the stripped content (both server-derived, client values ignored).
    let summary = match f.summary {
        Some(s) if !s.is_empty() => s,
        _ => generate_summary(f.content.as_deref().unwrap_or("")),
    };
    let mut a = post_translations::ActiveModel {
        post_id: Set(Some(post_id)),
        language_code: Set(f.language_code),
        summary: Set(Some(summary)),
        word_count: Set(Some(raw_chars(f.content.as_deref().unwrap_or("")))),
        created_at: Set(Some(store::now())),
        ..Default::default()
    };
    if let Some(v) = f.title {
        a.title = Set(Some(v));
    }
    if let Some(v) = f.slug {
        a.slug = Set(Some(v));
    }
    if let Some(v) = f.content {
        a.content = Set(Some(v));
    }
    if let Some(v) = f.original_content {
        a.original_content = Set(Some(v));
    }
    if let Some(v) = f.full_path {
        a.full_path = Set(Some(v));
    }
    if let Some(v) = f.seo {
        a.seo = Set(Some(v));
    }
    if let Some(v) = f.created_by {
        a.created_by = Set(Some(v));
    }
    a.insert(db).await.map_err(crate::db_status)
}

/// The reference's update-by-identifier gate: a missing
/// (post_id, language_code) row either turns into a create
/// (allow_missing) or 404s. The row itself is then updated by its own id.
pub(crate) async fn update_post_translation(
    db: &DatabaseConnection,
    id: i64,
    post_id: i64,
    f: PostTranslationFields,
    allow_missing: bool,
) -> StatusResult<post_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    if post_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    if !post_translation_exists(db, post_id, language_of(&f.language_code)).await? {
        if allow_missing {
            return create_post_translation(db, post_id, f).await;
        }
        return Err(crate::not_found_status("translation"));
    }
    let row = post_translations::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("translation"))?;
    let mut a: post_translations::ActiveModel = row.into();
    a.post_id = Set(Some(post_id));
    a.language_code = Set(f.language_code);
    // word_count is re-derived from the submitted content (the reference
    // always recounts, ignoring the client value).
    a.word_count = Set(Some(raw_chars(f.content.as_deref().unwrap_or(""))));
    // The remaining columns follow the reference's SetNillable shape: a
    // field the client omitted stays untouched (never nulled).
    if let Some(v) = f.title {
        a.title = Set(Some(v));
    }
    if let Some(v) = f.slug {
        a.slug = Set(Some(v));
    }
    if let Some(v) = f.summary {
        a.summary = Set(Some(v));
    }
    if let Some(v) = f.content {
        a.content = Set(Some(v));
    }
    if let Some(v) = f.original_content {
        a.original_content = Set(Some(v));
    }
    if let Some(v) = f.full_path {
        a.full_path = Set(Some(v));
    }
    if let Some(v) = f.seo {
        a.seo = Set(Some(v));
    }
    a.updated_at = Set(Some(store::now()));
    a.update(db).await.map_err(crate::db_status)
}

/// The two delete modes: by the translation row's own id, or by the
/// (post_id, language_code) identifier. A zero-match delete is not an
/// error (same as the reference).
pub(crate) async fn delete_post_translation(
    db: &DatabaseConnection,
    query: PostTranslationQuery,
) -> StatusResult<()> {
    let filter = match query {
        PostTranslationQuery::Id(id) => {
            if id == 0 {
                return Err(crate::state::bad("id must be greater than 0"));
            }
            Condition::all().add(post_translations::Column::Id.eq(id))
        }
        PostTranslationQuery::Identifier {
            post_id,
            language_code,
        } => {
            if post_id == 0 {
                return Err(crate::state::bad("post id must be greater than 0"));
            }
            if language_code.is_empty() {
                return Err(crate::state::bad("language code is required"));
            }
            Condition::all()
                .add(post_translations::Column::PostId.eq(post_id))
                .add(post_translations::Column::LanguageCode.eq(language_code))
        }
    };
    post_translations::Entity::delete_many()
        .filter(filter)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
