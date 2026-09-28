//! The `post` repository (post_repo.go).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use super::language_repo::language_of;
use crate::state::StatusResult;
use store::entities::page_translations;

/// The client-visible fields of a page translation.
pub(crate) struct PageTranslationFields {
    pub language_code: Option<String>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub cover_image: Option<String>,
    pub full_path: Option<String>,
    pub seo: Option<sea_orm::JsonValue>,
    pub created_by: Option<i64>,
}

pub(crate) enum PageTranslationQuery {
    Id(i64),
    Identifier { page_id: i64, language_code: String },
}

// ── page translations ────────────────────────────────────────────────

pub(crate) async fn page_translation_exists(
    db: &DatabaseConnection,
    page_id: i64,
    language_code: &str,
) -> StatusResult<bool> {
    let count = page_translations::Entity::find()
        .filter(
            Condition::all()
                .add(page_translations::Column::PageId.eq(page_id))
                .add(page_translations::Column::LanguageCode.eq(language_code)),
        )
        .count(db)
        .await
        .map_err(crate::db_status)?;
    Ok(count > 0)
}

/// The (page_id, language_code) row with the smallest id; `None` when
/// missing (not an error, as in the reference).
pub(crate) async fn get_page_translation(
    db: &DatabaseConnection,
    page_id: i64,
    language_code: &str,
) -> StatusResult<Option<page_translations::Model>> {
    page_translations::Entity::find()
        .filter(
            Condition::all()
                .add(page_translations::Column::PageId.eq(page_id))
                .add(page_translations::Column::LanguageCode.eq(language_code)),
        )
        .order_by_asc(page_translations::Column::Id)
        .one(db)
        .await
        .map_err(crate::db_status)
}

pub(crate) async fn create_page_translation(
    db: &DatabaseConnection,
    page_id: i64,
    f: PageTranslationFields,
) -> StatusResult<page_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    // The reference reuses the post wording here.
    if page_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    let mut a = page_translations::ActiveModel {
        page_id: Set(Some(page_id)),
        language_code: Set(f.language_code),
        created_at: Set(Some(store::now())),
        ..Default::default()
    };
    if let Some(v) = f.title {
        a.title = Set(Some(v));
    }
    if let Some(v) = f.slug {
        a.slug = Set(Some(v));
    }
    if let Some(v) = f.cover_image {
        a.cover_image = Set(Some(v));
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

/// Note the reference's quirk kept verbatim: a page translation update
/// rewrites ONLY title / cover_image / full_path / seo — the page_id,
/// language_code and slug of the request are validated but not written.
pub(crate) async fn update_page_translation(
    db: &DatabaseConnection,
    id: i64,
    page_id: i64,
    f: PageTranslationFields,
    allow_missing: bool,
) -> StatusResult<page_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    if page_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    if !page_translation_exists(db, page_id, language_of(&f.language_code)).await? {
        if allow_missing {
            return create_page_translation(db, page_id, f).await;
        }
        return Err(crate::not_found_status("translation"));
    }
    let row = page_translations::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("translation"))?;
    let mut a: page_translations::ActiveModel = row.into();
    if let Some(v) = f.title {
        a.title = Set(Some(v));
    }
    if let Some(v) = f.cover_image {
        a.cover_image = Set(Some(v));
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

pub(crate) async fn delete_page_translation(
    db: &DatabaseConnection,
    query: PageTranslationQuery,
) -> StatusResult<()> {
    let filter = match query {
        PageTranslationQuery::Id(id) => {
            if id == 0 {
                return Err(crate::state::bad("id must be greater than 0"));
            }
            Condition::all().add(page_translations::Column::Id.eq(id))
        }
        PageTranslationQuery::Identifier {
            page_id,
            language_code,
        } => {
            if page_id == 0 {
                return Err(crate::state::bad("page id must be greater than 0"));
            }
            if language_code.is_empty() {
                return Err(crate::state::bad("language code is required"));
            }
            Condition::all()
                .add(page_translations::Column::PageId.eq(page_id))
                .add(page_translations::Column::LanguageCode.eq(language_code))
        }
    };
    page_translations::Entity::delete_many()
        .filter(filter)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
