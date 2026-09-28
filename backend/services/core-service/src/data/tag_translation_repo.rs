//! The `post` repository (post_repo.go).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use super::language_repo::language_of;
use crate::state::StatusResult;
use store::entities::tag_translations;

pub(crate) use super::category_translation_repo::NamedTranslationFields;

pub(crate) enum TagTranslationQuery {
    Id(i64),
    Identifier { tag_id: i64, language_code: String },
}

// ── tag translations ─────────────────────────────────────────────────

pub(crate) async fn tag_translation_exists(
    db: &DatabaseConnection,
    tag_id: i64,
    language_code: &str,
) -> StatusResult<bool> {
    let count = tag_translations::Entity::find()
        .filter(
            Condition::all()
                .add(tag_translations::Column::TagId.eq(tag_id))
                .add(tag_translations::Column::LanguageCode.eq(language_code)),
        )
        .count(db)
        .await
        .map_err(crate::db_status)?;
    Ok(count > 0)
}

/// The (tag_id, language_code) row with the smallest id; `None` when
/// missing (not an error, as in the reference).
pub(crate) async fn get_tag_translation(
    db: &DatabaseConnection,
    tag_id: i64,
    language_code: &str,
) -> StatusResult<Option<tag_translations::Model>> {
    tag_translations::Entity::find()
        .filter(
            Condition::all()
                .add(tag_translations::Column::TagId.eq(tag_id))
                .add(tag_translations::Column::LanguageCode.eq(language_code)),
        )
        .order_by_asc(tag_translations::Column::Id)
        .one(db)
        .await
        .map_err(crate::db_status)
}

pub(crate) async fn create_tag_translation(
    db: &DatabaseConnection,
    tag_id: i64,
    f: NamedTranslationFields,
) -> StatusResult<tag_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    // The reference reuses the post wording here.
    if tag_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    let mut a = tag_translations::ActiveModel {
        tag_id: Set(Some(tag_id)),
        language_code: Set(f.language_code),
        created_at: Set(Some(store::now())),
        ..Default::default()
    };
    if let Some(v) = f.name {
        a.name = Set(Some(v));
    }
    if let Some(v) = f.slug {
        a.slug = Set(Some(v));
    }
    if let Some(v) = f.description {
        a.description = Set(Some(v));
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

pub(crate) async fn update_tag_translation(
    db: &DatabaseConnection,
    id: i64,
    tag_id: i64,
    f: NamedTranslationFields,
    allow_missing: bool,
) -> StatusResult<tag_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    if tag_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    if !tag_translation_exists(db, tag_id, language_of(&f.language_code)).await? {
        if allow_missing {
            return create_tag_translation(db, tag_id, f).await;
        }
        return Err(crate::not_found_status("translation"));
    }
    let row = tag_translations::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("translation"))?;
    let mut a: tag_translations::ActiveModel = row.into();
    a.tag_id = Set(Some(tag_id));
    a.language_code = Set(f.language_code);
    if let Some(v) = f.name {
        a.name = Set(Some(v));
    }
    if let Some(v) = f.slug {
        a.slug = Set(Some(v));
    }
    if let Some(v) = f.description {
        a.description = Set(Some(v));
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

pub(crate) async fn delete_tag_translation(
    db: &DatabaseConnection,
    query: TagTranslationQuery,
) -> StatusResult<()> {
    let filter = match query {
        TagTranslationQuery::Id(id) => {
            if id == 0 {
                return Err(crate::state::bad("id must be greater than 0"));
            }
            Condition::all().add(tag_translations::Column::Id.eq(id))
        }
        TagTranslationQuery::Identifier {
            tag_id,
            language_code,
        } => {
            if tag_id == 0 {
                return Err(crate::state::bad("tag id must be greater than 0"));
            }
            if language_code.is_empty() {
                return Err(crate::state::bad("language code is required"));
            }
            Condition::all()
                .add(tag_translations::Column::TagId.eq(tag_id))
                .add(tag_translations::Column::LanguageCode.eq(language_code))
        }
    };
    tag_translations::Entity::delete_many()
        .filter(filter)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
