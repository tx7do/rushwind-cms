//! ContentRepo — posts/categories/tags/pages with their translation
//! sub-tables, relation tables, and the search SQL (the data layer of
//! the content domain, mirroring the reference's
//! `internal/data/post_repo.go` family).

use sea_orm::sea_query::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn insert_post_translation(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn category_translations_of(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn insert_category_translation(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn tag_translations_of(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn insert_tag_translation(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn page_translations_of(
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

// Pre-existing plumbing the faces not yet routed through the repo
// layer still reach for; kept until the call sites consolidate here.
#[allow(dead_code)]
pub(crate) async fn insert_page_translation(
    db: &DatabaseConnection,
    a: page_translations::ActiveModel,
) -> StatusResult<()> {
    a.insert(db).await.map_err(crate::db_status)?;
    Ok(())
}

// ── translation sub-tables ───────────────────────────────────────────
//
// The per-language CRUD faces of the four content entities (the
// reference's *TranslationRepo family). The service layer only unpacks
// the request; the validation, derived columns and the delete query
// modes live here, mirroring the reference's data layer.

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

/// Category and tag translations share this shape.
pub(crate) struct NamedTranslationFields {
    pub language_code: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub full_path: Option<String>,
    pub seo: Option<sea_orm::JsonValue>,
    pub created_by: Option<i64>,
}

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

/// The two delete modes: by the translation row's own id, or by the
/// (main-row id, language code) identifier.
pub(crate) enum PostTranslationQuery {
    Id(i64),
    Identifier { post_id: i64, language_code: String },
}

pub(crate) enum CategoryTranslationQuery {
    Id(i64),
    Identifier {
        category_id: i64,
        language_code: String,
    },
}

pub(crate) enum TagTranslationQuery {
    Id(i64),
    Identifier { tag_id: i64, language_code: String },
}

pub(crate) enum PageTranslationQuery {
    Id(i64),
    Identifier { page_id: i64, language_code: String },
}

// The derived-column passes of the reference's PrepareTranslation: the
// `<[^>]+>` tag stripping, the `\s+` collapsing and the rule-based
// 100-char sentence excerpt. The slug auto-generation (slug.MakeLang)
// has no Rust port — a client-supplied slug passes through as-is, the
// same simplification the main-row create face already makes.

/// Strip HTML tags exactly like the `<[^>]+>` regex (a lone `<` without
/// a closing `>` stays literal).
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) if end > 1 => rest = &rest[start + end + 1..],
            _ => {
                out.push('<');
                rest = &rest[start + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// The rule-based excerpt (GenerateSummaryByRule with maxLength=100,
/// bySentence=true): single-line plain text, cut at the last
/// sentence-end punctuation within the first 100 chars.
fn generate_summary(content: &str) -> String {
    let plain = collapse_spaces(&strip_html(content));
    let plain = plain.trim();
    if plain.is_empty() {
        return "暂无摘要".to_string();
    }
    let runes: Vec<char> = plain.chars().collect();
    let mut summary: String = if runes.len() <= 100 {
        plain.to_string()
    } else {
        let truncated = &runes[..100];
        let last_end = truncated
            .iter()
            .rposition(|c| matches!(c, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';'))
            .map_or(100, |i| i + 1);
        truncated[..last_end].iter().collect()
    };
    if summary.chars().count() < runes.len() {
        summary.push_str("...");
    }
    summary
}

/// `\s+` (Go's whitespace class) collapses to a single space.
fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for c in s.chars() {
        if matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{0C}') {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// RawChars: the rune count of the tag-stripped content.
fn raw_chars(content: &str) -> i64 {
    strip_html(content).chars().count() as i64
}

fn language_of(code: &Option<String>) -> &str {
    code.as_deref().unwrap_or("")
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_html ──────────────────────────────────────────────────
    // The `<[^>]+>` port: leftmost matches, a lone `<` without a later
    // `>` stays literal. Mirrors the Go summary package's stripping.

    #[test]
    fn strip_html_removes_simple_and_nested_tags() {
        assert_eq!(strip_html("<p>Hello World</p>"), "Hello World");
        assert_eq!(
            strip_html("<div><p>Test</p><span>Content</span></div>"),
            "TestContent"
        );
        assert_eq!(
            strip_html("<div><p>This is <strong>bold</strong> text</p></div>"),
            "This is bold text"
        );
    }

    #[test]
    fn strip_html_removes_tags_with_attributes_and_self_closing() {
        assert_eq!(
            strip_html(r#"<p class="content" id="main">Hello</p>"#),
            "Hello"
        );
        assert_eq!(
            strip_html("Text<br/>More text<hr/>Final"),
            "TextMore textFinal"
        );
    }

    #[test]
    fn strip_html_keeps_lone_lt_literal() {
        // `<` without a closing `>` never forms a match (the regex
        // needs `[^>]+>`).
        assert_eq!(strip_html("a < b"), "a < b");
        assert_eq!(strip_html("a<"), "a<");
        // `<>` is empty between the brackets — no match, kept verbatim.
        assert_eq!(strip_html("<>"), "<>");
    }

    #[test]
    fn strip_html_matches_leftmost_span() {
        // `< b and 3<4` is one `[^>]+` run — the whole span vanishes.
        assert_eq!(strip_html("a < b and 3<4>5"), "a 5");
        assert_eq!(strip_html("<a<b>"), "");
        assert_eq!(strip_html("3<4>5"), "35");
        assert_eq!(strip_html(""), "");
    }

    // ── collapse_spaces ─────────────────────────────────────────────

    #[test]
    fn collapse_spaces_runs_whitespace_to_single_space() {
        assert_eq!(
            collapse_spaces("Hello    World    Test"),
            "Hello World Test"
        );
        assert_eq!(
            collapse_spaces("Hello\n\nWorld\t\tTest"),
            "Hello World Test"
        );
        assert_eq!(collapse_spaces(" \t\n\r\u{0C} mix "), " mix ");
        assert_eq!(collapse_spaces("nochange"), "nochange");
        assert_eq!(collapse_spaces(""), "");
    }

    // ── generate_summary ────────────────────────────────────────────
    // GenerateSummaryByRule(content, 100, true): the fixed 100-char
    // sentence-cut form. Mirrors the Go summary_test.go cases whose
    // expectations survive the fixed parameters.

    #[test]
    fn generate_summary_strips_and_collapses() {
        assert_eq!(generate_summary("<p>Hello World</p>"), "Hello World");
        assert_eq!(
            generate_summary("<div><p>Test</p><span>Content</span></div>"),
            "TestContent"
        );
        assert_eq!(
            generate_summary(r#"<p class="content" id="main">Hello</p>"#),
            "Hello"
        );
        assert_eq!(
            generate_summary("Text<br/>More text<hr/>Final"),
            "TextMore textFinal"
        );
        assert_eq!(
            generate_summary("Hello    World    Test"),
            "Hello World Test"
        );
        assert_eq!(
            generate_summary("Hello\n\nWorld\t\tTest"),
            "Hello World Test"
        );
        assert_eq!(generate_summary("   Hello World   "), "Hello World");
        assert_eq!(
            generate_summary("<p>  Hello   \n  World  </p>"),
            "Hello World"
        );
    }

    #[test]
    fn generate_summary_empty_falls_back_to_placeholder() {
        assert_eq!(generate_summary(""), "暂无摘要");
        assert_eq!(generate_summary("   \n\t  "), "暂无摘要");
        assert_eq!(
            generate_summary("<p></p><div></div><span></span>"),
            "暂无摘要"
        );
    }

    #[test]
    fn generate_summary_short_text_passes_through_without_ellipsis() {
        assert_eq!(
            generate_summary("This is a sentence."),
            "This is a sentence."
        );
        assert_eq!(
            generate_summary("<p>Vue3 暗黑模式教程是前端开发的重要知识点。</p>"),
            "Vue3 暗黑模式教程是前端开发的重要知识点。"
        );
        assert_eq!(
            generate_summary("<p>Vue3 <strong>暗黑模式</strong>教程。通过 CSS 变量实现。</p>"),
            "Vue3 暗黑模式教程。通过 CSS 变量实现。"
        );
        assert_eq!(
            generate_summary("一二三四五，六七八九十。"),
            "一二三四五，六七八九十。"
        );
    }

    #[test]
    fn generate_summary_exact_100_chars_gets_no_ellipsis() {
        let text: String = "a".repeat(100);
        assert_eq!(generate_summary(&text), text);
    }

    #[test]
    fn generate_summary_over_100_without_punctuation_hard_cuts() {
        // Mirrors the reference's "very long content" case at the fixed
        // 100-char window.
        let text = format!("a{}", "b".repeat(100));
        let want = format!("a{}", "b".repeat(99)) + "...";
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_cuts_at_last_sentence_end_within_window() {
        // 162 runes: the 100-char window keeps both 。, so the cut is
        // 62 runes + ellipsis.
        let text = format!(
            "{}。{}。{}",
            "一".repeat(30),
            "二".repeat(30),
            "三".repeat(100)
        );
        let want = format!("{}。{}。...", "一".repeat(30), "二".repeat(30));
        assert_eq!(generate_summary(&text), want);

        // The window's last 。 lands mid-text; the tail after it drops.
        let text = format!("{}。{}", "一".repeat(60), "x".repeat(60));
        let want = format!("{}。...", "一".repeat(60));
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_sentence_end_past_window_is_ignored() {
        // The only 。 sits beyond the 100-char window — hard cut.
        let text = format!("{}。{}", "x".repeat(100), "。尾");
        let want = format!("{}...", "x".repeat(100));
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_counts_ascii_punctuation_as_sentence_end() {
        // '.' within the window wins as the last sentence end.
        let text = format!("{} {} {}", "a".repeat(50), "b. c", "d".repeat(60));
        assert_eq!(
            generate_summary(&text),
            format!("{} b.", "a".repeat(50)) + "..."
        );
    }

    // ── raw_chars ───────────────────────────────────────────────────

    #[test]
    fn raw_chars_counts_stripped_runes() {
        assert_eq!(raw_chars(""), 0);
        assert_eq!(raw_chars("<p></p>"), 0);
        assert_eq!(raw_chars("<p>Hello</p>"), 5);
        assert_eq!(raw_chars("<div><p>Test</p><span>Content</span></div>"), 11);
        // Astral planes count per char, not per byte.
        assert_eq!(raw_chars("Hello 👋"), 7);
        assert_eq!(raw_chars("你好"), 2);
    }
}
