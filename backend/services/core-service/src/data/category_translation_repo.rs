//! The `post` repository (post_repo.go).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use super::language_repo::language_of;
use crate::state::StatusResult;
use store::entities::category_translations;



/// Category and tag translations share this shape.
pub struct NamedTranslationFields {
    pub language_code: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub full_path: Option<String>,
    pub seo: Option<sea_orm::JsonValue>,
    pub created_by: Option<i64>,
}

pub(crate) enum CategoryTranslationQuery {
    Id(i64),
    Identifier {
        category_id: i64,
        language_code: String,
    },
}

// ── category translations ────────────────────────────────────────────

pub(crate) async fn category_translation_exists(
    db: &DatabaseConnection,
    category_id: i64,
    language_code: &str,
) -> StatusResult<bool> {
    let count = category_translations::Entity::find()
        .filter(
            Condition::all()
                .add(category_translations::Column::CategoryId.eq(category_id))
                .add(category_translations::Column::LanguageCode.eq(language_code)),
        )
        .count(db)
        .await
        .map_err(crate::db_status)?;
    Ok(count > 0)
}

/// The (category_id, language_code) row with the smallest id; `None`
/// when missing (not an error, as in the reference).
pub(crate) async fn get_category_translation(
    db: &DatabaseConnection,
    category_id: i64,
    language_code: &str,
) -> StatusResult<Option<category_translations::Model>> {
    category_translations::Entity::find()
        .filter(
            Condition::all()
                .add(category_translations::Column::CategoryId.eq(category_id))
                .add(category_translations::Column::LanguageCode.eq(language_code)),
        )
        .order_by_asc(category_translations::Column::Id)
        .one(db)
        .await
        .map_err(crate::db_status)
}

pub(crate) async fn create_category_translation(
    db: &DatabaseConnection,
    category_id: i64,
    f: NamedTranslationFields,
) -> StatusResult<category_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    // The reference reuses the post wording here.
    if category_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    let mut a = category_translations::ActiveModel {
        category_id: Set(Some(category_id)),
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

pub(crate) async fn update_category_translation(
    db: &DatabaseConnection,
    id: i64,
    category_id: i64,
    f: NamedTranslationFields,
    allow_missing: bool,
) -> StatusResult<category_translations::Model> {
    if language_of(&f.language_code).is_empty() {
        return Err(crate::state::bad("language code is required"));
    }
    if category_id == 0 {
        return Err(crate::state::bad("post id is required"));
    }
    if !category_translation_exists(db, category_id, language_of(&f.language_code)).await? {
        if allow_missing {
            return create_category_translation(db, category_id, f).await;
        }
        return Err(crate::not_found_status("translation"));
    }
    let row = category_translations::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(crate::db_status)?
        .ok_or_else(|| crate::not_found_status("translation"))?;
    let mut a: category_translations::ActiveModel = row.into();
    a.category_id = Set(Some(category_id));
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

/// Unlike the other three, the reference validates only the id mode
/// here: the identifier form is taken as-is (a zero identifier simply
/// matches nothing).
pub(crate) async fn delete_category_translation(
    db: &DatabaseConnection,
    query: CategoryTranslationQuery,
) -> StatusResult<()> {
    let filter = match query {
        CategoryTranslationQuery::Id(id) => {
            if id == 0 {
                return Err(crate::state::bad(
                    "id is required for delete category translation",
                ));
            }
            Condition::all().add(category_translations::Column::Id.eq(id))
        }
        CategoryTranslationQuery::Identifier {
            category_id,
            language_code,
        } => Condition::all()
            .add(category_translations::Column::CategoryId.eq(category_id))
            .add(category_translations::Column::LanguageCode.eq(language_code)),
    };
    category_translations::Entity::delete_many()
        .filter(filter)
        .exec(db)
        .await
        .map_err(crate::db_status)?;
    Ok(())
}
