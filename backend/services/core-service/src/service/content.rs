//! The content domain services: Post, Category, Tag, Page — CRUD with
//! the translation sub-tables (per-language rows), the category/tag
//! relation tables, and the enum-name status mapping the golden schema
//! stores.

use std::sync::Arc;

use sea_orm::sea_query::Condition;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::content_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    categories, category_translations, page_translations, pages, post_categories, post_tags,
    post_translations, posts, tag_translations, tags,
};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

fn status_to_i32(prefix: &str, name: Option<String>) -> Option<i32> {
    // The stored enum-value name → the proto value (name ↔ number table
    // per enum: derive by ordinal of the known members).
    let name = name?;
    Some(match (prefix, name.as_str()) {
        (_, "POST_STATUS_DRAFT") => 1,
        (_, "POST_STATUS_PUBLISHED") => 2,
        (_, "POST_STATUS_SCHEDULED") => 3,
        (_, "POST_STATUS_TRASHED") => 4,
        (_, "PAGE_STATUS_DRAFT") => 1,
        (_, "PAGE_STATUS_PUBLISHED") => 2,
        (_, "PAGE_STATUS_ARCHIVED") => 3,
        (_, "CATEGORY_STATUS_ACTIVE") => 1,
        (_, "CATEGORY_STATUS_HIDDEN") => 2,
        (_, "CATEGORY_STATUS_ARCHIVED") => 3,
        (_, "TAG_STATUS_ACTIVE") => 1,
        (_, "TAG_STATUS_HIDDEN") => 2,
        (_, "TAG_STATUS_ARCHIVED") => 3,
        // Legacy labels the seed layers used before the enum alignment.
        (_, "CATEGORY_STATUS_DRAFT") => 1,
        (_, "CATEGORY_STATUS_PUBLISHED") => 2,
        (_, "TAG_STATUS_NORMAL") => 1,
        (_, "TAG_STATUS_DISABLED") => 2,
        _ => 0,
    })
}

fn status_from_i32(prefix: &str, v: i32) -> Option<String> {
    Some(
        match (prefix, v) {
            ("post", 1) => "POST_STATUS_DRAFT",
            ("post", 2) => "POST_STATUS_PUBLISHED",
            ("post", 3) => "POST_STATUS_SCHEDULED",
            ("post", 4) => "POST_STATUS_TRASHED",
            ("page", 1) => "PAGE_STATUS_DRAFT",
            ("page", 2) => "PAGE_STATUS_PUBLISHED",
            ("page", 3) => "PAGE_STATUS_ARCHIVED",
            ("category", 1) => "CATEGORY_STATUS_ACTIVE",
            ("category", 2) => "CATEGORY_STATUS_HIDDEN",
            ("category", 3) => "CATEGORY_STATUS_ARCHIVED",
            ("tag", 1) => "TAG_STATUS_ACTIVE",
            ("tag", 2) => "TAG_STATUS_HIDDEN",
            ("tag", 3) => "TAG_STATUS_ARCHIVED",
            _ => return None,
        }
        .to_string(),
    )
}

fn seo_of(json: Option<sea_orm::JsonValue>) -> Option<contentv1::SeoMeta> {
    // Field-level mapping (the prost type carries no serde derives).
    let v = json?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_owned);
    Some(contentv1::SeoMeta {
        seo_title: s("seoTitle"),
        meta_keywords: s("metaKeywords"),
        meta_description: s("metaDescription"),
        og_title: s("ogTitle"),
        og_description: s("ogDescription"),
        og_image: s("ogImage"),
        ..Default::default()
    })
}

fn seo_to_json(seo: Option<contentv1::SeoMeta>) -> Option<sea_orm::JsonValue> {
    let seo = seo?;
    serde_json::json!({
        "seoTitle": seo.seo_title,
        "metaKeywords": seo.meta_keywords,
        "metaDescription": seo.meta_description,
        "ogTitle": seo.og_title,
        "ogDescription": seo.og_description,
        "ogImage": seo.og_image,
    })
    .into()
}

// ── Post ─────────────────────────────────────────────────────────────

pub(crate) fn post_translation_proto(r: post_translations::Model) -> contentv1::PostTranslation {
    contentv1::PostTranslation {
        id: Some(r.id as u32),
        post_id: r.post_id.map(|v| v as u32),
        language_code: r.language_code,
        title: r.title,
        slug: r.slug,
        summary: r.summary,
        content: r.content,
        original_content: r.original_content,
        full_path: r.full_path,
        word_count: r.word_count.map(|v| v as u32),
        seo: seo_of(r.seo),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub(crate) fn post_proto(
    r: posts::Model,
    translations: Vec<post_translations::Model>,
    category_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> contentv1::Post {
    let available_languages = translations
        .iter()
        .filter_map(|t| t.language_code.clone())
        .collect();
    contentv1::Post {
        id: Some(r.id as u32),
        status: status_to_i32("post", r.status),
        editor_type: r.editor_type.as_deref().map(|_| 1),
        code: r.code,
        disallow_comment: r.disallow_comment,
        in_progress: r.in_progress,
        auto_summary: r.auto_summary,
        is_featured: r.is_featured,
        sort_order: r.sort_order.map(|v| v as u32),
        author_id: r.author_id.map(|v| v as u32),
        author_name: r.author_name,
        thumbnail: r.thumbnail,
        custom_fields: r
            .custom_fields
            .and_then(|j| serde_json::from_value(j).ok())
            .unwrap_or_default(),
        translations: translations
            .into_iter()
            .map(post_translation_proto)
            .collect(),
        available_languages,
        category_ids: category_ids.into_iter().map(|v| v as u32).collect(),
        tag_ids: tag_ids.into_iter().map(|v| v as u32).collect(),
        password_hash: r.password_hash,
        publish_time: r.publish_time.and_then(ts_to_proto),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

async fn post_relations(
    db: &sea_orm::DatabaseConnection,
    post_id: i64,
) -> Result<(Vec<post_translations::Model>, Vec<i64>, Vec<i64>), Status> {
    let translations = post_translations::Entity::find()
        .filter(post_translations::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(db_status)?;
    let category_ids = post_categories::Entity::find()
        .filter(post_categories::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(db_status)?
        .into_iter()
        .map(|r| r.category_id)
        .collect();
    let tag_ids = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.eq(post_id))
        .all(db)
        .await
        .map_err(db_status)?
        .into_iter()
        .map(|r| r.tag_id)
        .collect();
    Ok((translations, category_ids, tag_ids))
}

pub struct PostServiceImpl {
    pub state: Arc<AppState>,
}

/// The embedded-array translation upsert shared by the update faces
/// (the reference's repo semantics): a (main id, language) row is
/// updated in place, a missing language inserts fresh.
async fn upsert_post_translation_row(
    db: &sea_orm::DatabaseConnection,
    post_id: i64,
    t: contentv1::PostTranslation,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), Status> {
    let lang = t.language_code.clone().unwrap_or_default();
    if lang.is_empty() {
        return Ok(());
    }
    let existing = post_translations::Entity::find()
        .filter(post_translations::Column::PostId.eq(post_id))
        .filter(post_translations::Column::LanguageCode.eq(lang.clone()))
        .one(db)
        .await
        .map_err(db_status)?;
    if let Some(row) = existing {
        let mut a: post_translations::ActiveModel = row.into();
        a.title = Set(t.title);
        a.slug = Set(t.slug);
        a.summary = Set(t.summary);
        a.content = Set(t.content);
        a.full_path = Set(t.full_path);
        a.seo = Set(seo_to_json(t.seo));
        a.updated_at = Set(Some(now));
        a.update(db).await.map_err(db_status)?;
    } else {
        post_translations::ActiveModel {
            post_id: Set(Some(post_id)),
            language_code: Set(Some(lang)),
            title: Set(t.title),
            slug: Set(t.slug),
            summary: Set(t.summary),
            content: Set(t.content),
            full_path: Set(t.full_path),
            seo: Set(seo_to_json(t.seo)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

/// The category translation embedded upsert (shared shape with tags).
async fn upsert_category_translation_row(
    db: &sea_orm::DatabaseConnection,
    category_id: i64,
    t: contentv1::CategoryTranslation,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), Status> {
    let lang = t.language_code.clone().unwrap_or_default();
    if lang.is_empty() {
        return Ok(());
    }
    let existing = category_translations::Entity::find()
        .filter(category_translations::Column::CategoryId.eq(category_id))
        .filter(category_translations::Column::LanguageCode.eq(lang.clone()))
        .one(db)
        .await
        .map_err(db_status)?;
    if let Some(row) = existing {
        let mut a: category_translations::ActiveModel = row.into();
        a.name = Set(t.name);
        a.slug = Set(t.slug);
        a.description = Set(t.description);
        a.cover_image = Set(t.cover_image);
        a.full_path = Set(t.full_path);
        a.seo = Set(seo_to_json(t.seo));
        a.updated_at = Set(Some(now));
        a.update(db).await.map_err(db_status)?;
    } else {
        category_translations::ActiveModel {
            category_id: Set(Some(category_id)),
            language_code: Set(Some(lang)),
            name: Set(t.name),
            slug: Set(t.slug),
            description: Set(t.description),
            cover_image: Set(t.cover_image),
            full_path: Set(t.full_path),
            seo: Set(seo_to_json(t.seo)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

/// The tag translation embedded upsert (the named-translation shape).
async fn upsert_tag_translation_row(
    db: &sea_orm::DatabaseConnection,
    tag_id: i64,
    t: contentv1::TagTranslation,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), Status> {
    let lang = t.language_code.clone().unwrap_or_default();
    if lang.is_empty() {
        return Ok(());
    }
    let existing = tag_translations::Entity::find()
        .filter(tag_translations::Column::TagId.eq(tag_id))
        .filter(tag_translations::Column::LanguageCode.eq(lang.clone()))
        .one(db)
        .await
        .map_err(db_status)?;
    if let Some(row) = existing {
        let mut a: tag_translations::ActiveModel = row.into();
        a.name = Set(t.name);
        a.slug = Set(t.slug);
        a.description = Set(t.description);
        a.cover_image = Set(t.cover_image);
        a.full_path = Set(t.full_path);
        a.seo = Set(seo_to_json(t.seo));
        a.updated_at = Set(Some(now));
        a.update(db).await.map_err(db_status)?;
    } else {
        tag_translations::ActiveModel {
            tag_id: Set(Some(tag_id)),
            language_code: Set(Some(lang)),
            name: Set(t.name),
            slug: Set(t.slug),
            description: Set(t.description),
            cover_image: Set(t.cover_image),
            full_path: Set(t.full_path),
            seo: Set(seo_to_json(t.seo)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

/// The page translation embedded upsert.
async fn upsert_page_translation_row(
    db: &sea_orm::DatabaseConnection,
    page_id: i64,
    t: contentv1::PageTranslation,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), Status> {
    let lang = t.language_code.clone().unwrap_or_default();
    if lang.is_empty() {
        return Ok(());
    }
    let existing = page_translations::Entity::find()
        .filter(page_translations::Column::PageId.eq(page_id))
        .filter(page_translations::Column::LanguageCode.eq(lang.clone()))
        .one(db)
        .await
        .map_err(db_status)?;
    if let Some(row) = existing {
        let mut a: page_translations::ActiveModel = row.into();
        a.title = Set(t.title);
        a.slug = Set(t.slug);
        a.cover_image = Set(t.cover_image);
        a.full_path = Set(t.full_path);
        a.seo = Set(seo_to_json(t.seo));
        a.updated_at = Set(Some(now));
        a.update(db).await.map_err(db_status)?;
    } else {
        page_translations::ActiveModel {
            page_id: Set(Some(page_id)),
            language_code: Set(Some(lang)),
            title: Set(t.title),
            slug: Set(t.slug),
            cover_image: Set(t.cover_image),
            full_path: Set(t.full_path),
            seo: Set(seo_to_json(t.seo)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_status)?;
    }
    Ok(())
}

#[async_trait::async_trait]
impl contentv1::post_service_server::PostService for PostServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListPostResponse>, Status> {
        let (rows, total) =
            fetch_paged(&self.state.db, posts::Entity::find(), &request.into_inner())
                .await
                .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let (translations, category_ids, tag_ids) =
                post_relations(&self.state.db, r.id).await?;
            items.push(post_proto(r, translations, category_ids, tag_ids));
        }
        Ok(Response::new(contentv1::ListPostResponse { items, total }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetPostRequest>,
    ) -> Result<Response<contentv1::Post>, Status> {
        let req = request.into_inner();
        let id = match req.query_by {
            Some(contentv1::get_post_request::QueryBy::Id(id)) => id as i64,
            Some(contentv1::get_post_request::QueryBy::Code(code)) => {
                posts::Entity::find()
                    .filter(posts::Column::Code.eq(code))
                    .one(&self.state.db)
                    .await
                    .map_err(db_status)?
                    .ok_or_else(|| not_found("post"))?
                    .id
            }
            None => return Err(bad("query_by required")),
        };
        let row = posts::Entity::find_by_id(id)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("post"))?;
        let (translations, category_ids, tag_ids) = post_relations(&self.state.db, id).await?;
        Ok(Response::new(post_proto(
            row,
            translations,
            category_ids,
            tag_ids,
        )))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreatePostRequest>,
    ) -> Result<Response<contentv1::Post>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = posts::ActiveModel {
            // 新建文章状态缺省落草稿（显式携带状态则照收），发布仍以
            // Update 为正式切换口
            status: Set(Some(
                data.status
                    .and_then(|v| status_from_i32("post", v))
                    .unwrap_or_else(|| "POST_STATUS_DRAFT".to_string()),
            )),
            code: Set(data.code),
            disallow_comment: Set(data.disallow_comment),
            in_progress: Set(data.in_progress),
            auto_summary: Set(data.auto_summary),
            is_featured: Set(data.is_featured),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            author_id: Set(data.author_id.map(|v| v as i64)),
            author_name: Set(data.author_name),
            thumbnail: Set(data.thumbnail),
            password_hash: Set(data.password_hash),
            publish_time: Set(data
                .publish_time
                .and_then(|ts| crate::state::ts_from_proto(&ts))),
            custom_fields: Set((!data.custom_fields.is_empty())
                .then(|| serde_json::to_value(&data.custom_fields).ok())
                .flatten()),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;

        // Translations + relations.
        for t in data.translations {
            post_translations::ActiveModel {
                post_id: Set(Some(row.id)),
                language_code: Set(t.language_code),
                title: Set(t.title),
                slug: Set(t.slug),
                summary: Set(t.summary),
                content: Set(t.content),
                original_content: Set(t.original_content),
                full_path: Set(t.full_path),
                word_count: Set(t.word_count.map(|v| v as i64)),
                seo: Set(seo_to_json(t.seo)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        for cid in data.category_ids {
            post_categories::Entity::insert(post_categories::ActiveModel {
                post_id: Set(row.id),
                category_id: Set(cid as i64),
                ..Default::default()
            })
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        for tid in data.tag_ids {
            post_tags::Entity::insert(post_tags::ActiveModel {
                post_id: Set(row.id),
                tag_id: Set(tid as i64),
                ..Default::default()
            })
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        }

        let (translations, category_ids, tag_ids) = post_relations(&self.state.db, row.id).await?;
        Ok(Response::new(post_proto(
            row,
            translations,
            category_ids,
            tag_ids,
        )))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdatePostRequest>,
    ) -> Result<Response<contentv1::Post>, Status> {
        let req = request.into_inner();
        let row = repo::post_by_id(&self.state.db, req.id as i64).await?;
        let mut a: posts::ActiveModel = row.into();
        let embedded = req.data.as_ref().map(|d| d.translations.clone());
        if let Some(data) = req.data {
            if let Some(v) = data.status {
                a.status = Set(status_from_i32("post", v));
            }
            if let Some(v) = data.code {
                a.code = Set(Some(v));
            }
            if let Some(v) = data.disallow_comment {
                a.disallow_comment = Set(Some(v));
            }
            if let Some(v) = data.in_progress {
                a.in_progress = Set(Some(v));
            }
            if let Some(v) = data.is_featured {
                a.is_featured = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
            if let Some(v) = data.thumbnail {
                a.thumbnail = Set(Some(v));
            }
            if let Some(v) = data.author_name {
                a.author_name = Set(Some(v));
            }
            if let Some(v) = data.password_hash {
                a.password_hash = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(ts) = embedded {
            let now = store::now();
            for t in ts {
                upsert_post_translation_row(&self.state.db, row.id, t, now).await?;
            }
        }
        let (translations, category_ids, tag_ids) = post_relations(&self.state.db, row.id).await?;
        Ok(Response::new(post_proto(
            row,
            translations,
            category_ids,
            tag_ids,
        )))
    }

    async fn search_posts(
        &self,
        request: Request<contentv1::SearchPostsRequest>,
    ) -> Result<Response<contentv1::SearchPostsResponse>, Status> {
        let req = request.into_inner();
        let keyword = format!("%{}%", req.query.replace('%', ""));
        // SQL full-text fallback (the OpenSearch bridge lands with the
        // search phase): published posts whose any translation matches,
        // returned as (post_id, language, title) hits.
        let page = req.page.max(1) as u64;
        let page_size = req.page_size.clamp(1, 50) as u64;
        let offset = (page - 1) * page_size;
        let db = &self.state.db;
        use sea_orm::ConnectionTrait as _;
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
                    keyword.into(),
                    req.language.clone().into(),
                    (page_size as i64).into(),
                    (offset as i64).into(),
                ],
            ))
            .await
            .map_err(db_status)?;
        let items: Vec<contentv1::SearchPostHit> = rows
            .into_iter()
            .map(|r| contentv1::SearchPostHit {
                post_id: r.try_get::<i64>("", "post_id").unwrap_or(0) as u32,
                language: r.try_get::<String>("", "lang").unwrap_or_default(),
                title: r.try_get::<String>("", "title").unwrap_or_default(),
            })
            .collect();
        let total = items.len() as i32;
        Ok(Response::new(contentv1::SearchPostsResponse {
            items,
            total,
        }))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeletePostRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_post_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_post(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn translation_exists(
        &self,
        request: Request<contentv1::PostTranslationExistsRequest>,
    ) -> Result<Response<contentv1::PostTranslationExistsResponse>, Status> {
        let req = request.into_inner();
        let exists =
            repo::post_translation_exists(&self.state.db, req.post_id as i64, &req.language_code)
                .await?;
        Ok(Response::new(contentv1::PostTranslationExistsResponse {
            exists,
        }))
    }

    async fn get_translation(
        &self,
        request: Request<contentv1::GetPostRequest>,
    ) -> Result<Response<contentv1::PostTranslation>, Status> {
        let req = request.into_inner();
        // The reference reads req.GetId(): a code query degrades to id 0
        // and thus the zero translation (a missing translation is not an
        // error — the response is simply the empty message).
        let id = match req.query_by {
            Some(contentv1::get_post_request::QueryBy::Id(id)) => id as i64,
            _ => 0,
        };
        let row =
            repo::get_post_translation(&self.state.db, id, &req.locale.unwrap_or_default()).await?;
        Ok(Response::new(
            row.map(post_translation_proto).unwrap_or_default(),
        ))
    }

    async fn create_translation(
        &self,
        request: Request<contentv1::CreatePostTranslationRequest>,
    ) -> Result<Response<contentv1::PostTranslation>, Status> {
        // The ES reindex enqueue the reference performs after every
        // translation mutation (enqueuePostReindex) is skipped: no
        // OpenSearch bridge in this port yet (known backlog).
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::create_post_translation(
            &self.state.db,
            req.post_id as i64,
            repo::PostTranslationFields {
                language_code: data.language_code,
                title: data.title,
                slug: data.slug,
                summary: data.summary,
                content: data.content,
                original_content: data.original_content,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
        )
        .await?;
        Ok(Response::new(post_translation_proto(row)))
    }

    async fn update_translation(
        &self,
        request: Request<contentv1::UpdatePostTranslationRequest>,
    ) -> Result<Response<contentv1::PostTranslation>, Status> {
        // ES reindex enqueue skipped (see create_translation).
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::update_post_translation(
            &self.state.db,
            req.id as i64,
            data.post_id.map(|v| v as i64).unwrap_or(0),
            // update_mask is accepted but not applied, like the other
            // update faces of this port.
            repo::PostTranslationFields {
                language_code: data.language_code,
                title: data.title,
                slug: data.slug,
                summary: data.summary,
                content: data.content,
                original_content: data.original_content,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
            req.allow_missing.unwrap_or(false),
        )
        .await?;
        Ok(Response::new(post_translation_proto(row)))
    }

    async fn delete_translation(
        &self,
        request: Request<contentv1::DeletePostTranslationRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        // ES reindex enqueue skipped (see create_translation).
        let req = request.into_inner();
        let query = match req.query_by {
            Some(contentv1::delete_post_translation_request::QueryBy::Id(id)) => {
                repo::PostTranslationQuery::Id(id as i64)
            }
            Some(contentv1::delete_post_translation_request::QueryBy::Identifier(identifier)) => {
                repo::PostTranslationQuery::Identifier {
                    post_id: identifier.post_id as i64,
                    language_code: identifier.language_code,
                }
            }
            None => return Err(bad("query_by required")),
        };
        repo::delete_post_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Category ─────────────────────────────────────────────────────────

pub(crate) fn category_translation_proto(
    r: category_translations::Model,
) -> contentv1::CategoryTranslation {
    contentv1::CategoryTranslation {
        id: Some(r.id as u32),
        category_id: r.category_id.map(|v| v as u32),
        language_code: r.language_code,
        name: r.name,
        slug: r.slug,
        description: r.description,
        cover_image: r.cover_image,
        full_path: r.full_path,
        seo: seo_of(r.seo),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub(crate) fn category_proto(
    r: categories::Model,
    translations: Vec<category_translations::Model>,
) -> contentv1::Category {
    let available_languages = translations
        .iter()
        .filter_map(|t| t.language_code.clone())
        .collect();
    contentv1::Category {
        id: Some(r.id as u32),
        status: status_to_i32("category", r.status),
        sort_order: r.sort_order.map(|v| v as u32),
        is_nav: r.is_nav,
        icon: r.icon,
        code: r.code,
        thumbnail: r.thumbnail,
        post_count: r.post_count.map(|v| v as u32),
        direct_post_count: r.direct_post_count.map(|v| v as u32),
        depth: r.depth,
        path: r.path,
        content_model_id: r.content_model_id.map(|v| v as u32),
        parent_id: r.parent_id.map(|v| v as u32),
        custom_fields: r
            .custom_fields
            .and_then(|j| serde_json::from_value(j).ok())
            .unwrap_or_default(),
        translations: translations
            .into_iter()
            .map(category_translation_proto)
            .collect(),
        available_languages,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

async fn category_translations_of(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Result<Vec<category_translations::Model>, Status> {
    category_translations::Entity::find()
        .filter(category_translations::Column::CategoryId.eq(id))
        .all(db)
        .await
        .map_err(db_status)
}

pub struct CategoryServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::category_service_server::CategoryService for CategoryServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListCategoryResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            categories::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations = category_translations_of(&self.state.db, r.id).await?;
            items.push(category_proto(r, translations));
        }
        Ok(Response::new(contentv1::ListCategoryResponse {
            items,
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetCategoryRequest>,
    ) -> Result<Response<contentv1::Category>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_category_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::category_by_id(&self.state.db, id as i64).await?;
        let translations = category_translations_of(&self.state.db, row.id).await?;
        Ok(Response::new(category_proto(row, translations)))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreateCategoryRequest>,
    ) -> Result<Response<contentv1::Category>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = categories::ActiveModel {
            status: Set(status_from_i32("category", data.status.unwrap_or(1))),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_nav: Set(data.is_nav),
            icon: Set(data.icon),
            code: Set(data.code),
            thumbnail: Set(data.thumbnail),
            depth: Set(data.depth),
            parent_id: Set(data.parent_id.map(|v| v as i64)),
            content_model_id: Set(data.content_model_id.map(|v| v as i64)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        for t in data.translations {
            category_translations::ActiveModel {
                category_id: Set(Some(row.id)),
                language_code: Set(t.language_code),
                name: Set(t.name),
                slug: Set(t.slug),
                description: Set(t.description),
                cover_image: Set(t.cover_image),
                full_path: Set(t.full_path),
                seo: Set(seo_to_json(t.seo)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        let translations = category_translations_of(&self.state.db, row.id).await?;
        Ok(Response::new(category_proto(row, translations)))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdateCategoryRequest>,
    ) -> Result<Response<contentv1::Category>, Status> {
        let req = request.into_inner();
        let row = repo::category_by_id(&self.state.db, req.id as i64).await?;
        let mut a: categories::ActiveModel = row.into();
        let embedded = req.data.as_ref().map(|d| d.translations.clone());
        if let Some(data) = req.data {
            if let Some(v) = data.status {
                a.status = Set(status_from_i32("category", v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
            if let Some(v) = data.is_nav {
                a.is_nav = Set(Some(v));
            }
            if let Some(v) = data.icon {
                a.icon = Set(Some(v));
            }
            if let Some(v) = data.thumbnail {
                a.thumbnail = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(ts) = embedded {
            let now = store::now();
            for t in ts {
                upsert_category_translation_row(&self.state.db, row.id, t, now).await?;
            }
        }
        let translations = category_translations_of(&self.state.db, row.id).await?;
        Ok(Response::new(category_proto(row, translations)))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeleteCategoryRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_category_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_category(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn translation_exists(
        &self,
        request: Request<contentv1::CategoryTranslationExistsRequest>,
    ) -> Result<Response<contentv1::CategoryTranslationExistsResponse>, Status> {
        let req = request.into_inner();
        let exists = repo::category_translation_exists(
            &self.state.db,
            req.category_id as i64,
            &req.language_code,
        )
        .await?;
        Ok(Response::new(
            contentv1::CategoryTranslationExistsResponse { exists },
        ))
    }

    async fn get_translation(
        &self,
        request: Request<contentv1::GetCategoryRequest>,
    ) -> Result<Response<contentv1::CategoryTranslation>, Status> {
        let req = request.into_inner();
        // The reference reads req.GetId(): a code query degrades to id 0
        // and thus the zero translation (a missing translation is not an
        // error — the response is simply the empty message).
        let id = match req.query_by {
            Some(contentv1::get_category_request::QueryBy::Id(id)) => id as i64,
            _ => 0,
        };
        let row =
            repo::get_category_translation(&self.state.db, id, &req.locale.unwrap_or_default())
                .await?;
        Ok(Response::new(
            row.map(category_translation_proto).unwrap_or_default(),
        ))
    }

    async fn create_translation(
        &self,
        request: Request<contentv1::CreateCategoryTranslationRequest>,
    ) -> Result<Response<contentv1::CategoryTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::create_category_translation(
            &self.state.db,
            req.category_id as i64,
            repo::NamedTranslationFields {
                language_code: data.language_code,
                name: data.name,
                slug: data.slug,
                description: data.description,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
        )
        .await?;
        Ok(Response::new(category_translation_proto(row)))
    }

    async fn update_translation(
        &self,
        request: Request<contentv1::UpdateCategoryTranslationRequest>,
    ) -> Result<Response<contentv1::CategoryTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::update_category_translation(
            &self.state.db,
            req.id as i64,
            data.category_id.map(|v| v as i64).unwrap_or(0),
            // update_mask is accepted but not applied, like the other
            // update faces of this port.
            repo::NamedTranslationFields {
                language_code: data.language_code,
                name: data.name,
                slug: data.slug,
                description: data.description,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
            req.allow_missing.unwrap_or(false),
        )
        .await?;
        Ok(Response::new(category_translation_proto(row)))
    }

    async fn delete_translation(
        &self,
        request: Request<contentv1::DeleteCategoryTranslationRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let query = match req.query_by {
            Some(contentv1::delete_category_translation_request::QueryBy::Id(id)) => {
                repo::CategoryTranslationQuery::Id(id as i64)
            }
            Some(contentv1::delete_category_translation_request::QueryBy::Identifier(
                identifier,
            )) => repo::CategoryTranslationQuery::Identifier {
                category_id: identifier.category_id as i64,
                language_code: identifier.language_code,
            },
            None => return Err(bad("query_by required")),
        };
        repo::delete_category_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Tag ──────────────────────────────────────────────────────────────

pub(crate) fn tag_translation_proto(r: tag_translations::Model) -> contentv1::TagTranslation {
    contentv1::TagTranslation {
        id: Some(r.id as u32),
        tag_id: r.tag_id.map(|v| v as u32),
        language_code: r.language_code,
        name: r.name,
        slug: r.slug,
        description: r.description,
        cover_image: r.cover_image,
        full_path: r.full_path,
        seo: seo_of(r.seo),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub(crate) fn tag_proto(
    r: tags::Model,
    translations: Vec<tag_translations::Model>,
) -> contentv1::Tag {
    let available_languages = translations
        .iter()
        .filter_map(|t| t.language_code.clone())
        .collect();
    contentv1::Tag {
        id: Some(r.id as u32),
        status: status_to_i32("tag", r.status),
        color: r.color,
        icon: r.icon,
        group: r.r#group,
        sort_order: r.sort_order.map(|v| v as u32),
        is_featured: r.is_featured,
        code: r.code,
        post_count: r.post_count.map(|v| v as u32),
        translations: translations
            .into_iter()
            .map(tag_translation_proto)
            .collect(),
        available_languages,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct TagServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::tag_service_server::TagService for TagServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListTagResponse>, Status> {
        let (rows, total) =
            fetch_paged(&self.state.db, tags::Entity::find(), &request.into_inner())
                .await
                .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations = tag_translations::Entity::find()
                .filter(tag_translations::Column::TagId.eq(r.id))
                .all(&self.state.db)
                .await
                .map_err(db_status)?;
            items.push(tag_proto(r, translations));
        }
        Ok(Response::new(contentv1::ListTagResponse { items, total }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetTagRequest>,
    ) -> Result<Response<contentv1::Tag>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_tag_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::tag_by_id(&self.state.db, id as i64).await?;
        let translations = tag_translations::Entity::find()
            .filter(tag_translations::Column::TagId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(tag_proto(row, translations)))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreateTagRequest>,
    ) -> Result<Response<contentv1::Tag>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = tags::ActiveModel {
            status: Set(status_from_i32("tag", data.status.unwrap_or(1))),
            color: Set(data.color),
            icon: Set(data.icon),
            group: Set(data.group),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            is_featured: Set(data.is_featured),
            code: Set(data.code),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        for t in data.translations {
            tag_translations::ActiveModel {
                tag_id: Set(Some(row.id)),
                language_code: Set(t.language_code),
                name: Set(t.name),
                slug: Set(t.slug),
                description: Set(t.description),
                cover_image: Set(t.cover_image),
                full_path: Set(t.full_path),
                seo: Set(seo_to_json(t.seo)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        let translations = tag_translations::Entity::find()
            .filter(tag_translations::Column::TagId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(tag_proto(row, translations)))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdateTagRequest>,
    ) -> Result<Response<contentv1::Tag>, Status> {
        let req = request.into_inner();
        let row = repo::tag_by_id(&self.state.db, req.id as i64).await?;
        let mut a: tags::ActiveModel = row.into();
        let embedded = req.data.as_ref().map(|d| d.translations.clone());
        if let Some(data) = req.data {
            if let Some(v) = data.status {
                a.status = Set(status_from_i32("tag", v));
            }
            if let Some(v) = data.color {
                a.color = Set(Some(v));
            }
            if let Some(v) = data.icon {
                a.icon = Set(Some(v));
            }
            if let Some(v) = data.r#group {
                a.group = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
            if let Some(v) = data.is_featured {
                a.is_featured = Set(Some(v));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(ts) = embedded {
            let now = store::now();
            for t in ts {
                upsert_tag_translation_row(&self.state.db, row.id, t, now).await?;
            }
        }
        let translations = tag_translations::Entity::find()
            .filter(tag_translations::Column::TagId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(tag_proto(row, translations)))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeleteTagRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_tag_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_tag(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn translation_exists(
        &self,
        request: Request<contentv1::TagTranslationExistsRequest>,
    ) -> Result<Response<contentv1::TagTranslationExistsResponse>, Status> {
        let req = request.into_inner();
        let exists =
            repo::tag_translation_exists(&self.state.db, req.tag_id as i64, &req.language_code)
                .await?;
        Ok(Response::new(contentv1::TagTranslationExistsResponse {
            exists,
        }))
    }

    async fn get_translation(
        &self,
        request: Request<contentv1::GetTagRequest>,
    ) -> Result<Response<contentv1::TagTranslation>, Status> {
        let req = request.into_inner();
        // The reference reads req.GetId(): a code query degrades to id 0
        // and thus the zero translation (a missing translation is not an
        // error — the response is simply the empty message).
        let id = match req.query_by {
            Some(contentv1::get_tag_request::QueryBy::Id(id)) => id as i64,
            _ => 0,
        };
        let row =
            repo::get_tag_translation(&self.state.db, id, &req.locale.unwrap_or_default()).await?;
        Ok(Response::new(
            row.map(tag_translation_proto).unwrap_or_default(),
        ))
    }

    async fn create_translation(
        &self,
        request: Request<contentv1::CreateTagTranslationRequest>,
    ) -> Result<Response<contentv1::TagTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::create_tag_translation(
            &self.state.db,
            req.tag_id as i64,
            repo::NamedTranslationFields {
                language_code: data.language_code,
                name: data.name,
                slug: data.slug,
                description: data.description,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
        )
        .await?;
        Ok(Response::new(tag_translation_proto(row)))
    }

    async fn update_translation(
        &self,
        request: Request<contentv1::UpdateTagTranslationRequest>,
    ) -> Result<Response<contentv1::TagTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::update_tag_translation(
            &self.state.db,
            req.id as i64,
            data.tag_id.map(|v| v as i64).unwrap_or(0),
            // update_mask is accepted but not applied, like the other
            // update faces of this port.
            repo::NamedTranslationFields {
                language_code: data.language_code,
                name: data.name,
                slug: data.slug,
                description: data.description,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
            req.allow_missing.unwrap_or(false),
        )
        .await?;
        Ok(Response::new(tag_translation_proto(row)))
    }

    async fn delete_translation(
        &self,
        request: Request<contentv1::DeleteTagTranslationRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let query = match req.query_by {
            Some(contentv1::delete_tag_translation_request::QueryBy::Id(id)) => {
                repo::TagTranslationQuery::Id(id as i64)
            }
            Some(contentv1::delete_tag_translation_request::QueryBy::Identifier(identifier)) => {
                repo::TagTranslationQuery::Identifier {
                    tag_id: identifier.tag_id as i64,
                    language_code: identifier.language_code,
                }
            }
            None => return Err(bad("query_by required")),
        };
        repo::delete_tag_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Page ─────────────────────────────────────────────────────────────

pub(crate) fn page_translation_proto(r: page_translations::Model) -> contentv1::PageTranslation {
    contentv1::PageTranslation {
        id: Some(r.id as u32),
        page_id: r.page_id.map(|v| v as u32),
        language_code: r.language_code,
        title: r.title,
        slug: r.slug,
        cover_image: r.cover_image,
        full_path: r.full_path,
        seo: seo_of(r.seo),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub(crate) fn page_proto(
    r: pages::Model,
    translations: Vec<page_translations::Model>,
) -> contentv1::Page {
    let available_languages = translations
        .iter()
        .filter_map(|t| t.language_code.clone())
        .collect();
    contentv1::Page {
        id: Some(r.id as u32),
        status: status_to_i32("page", r.status),
        slug: r.slug,
        author_id: r.author_id.map(|v| v as u32),
        author_name: r.author_name,
        disallow_comment: r.disallow_comment,
        redirect_url: r.redirect_url,
        show_in_navigation: r.show_in_navigation,
        sort_order: r.sort_order.map(|v| v as u32),
        template: r.template,
        is_custom_template: r.is_custom_template,
        thumbnail: r.thumbnail,
        custom_fields: r
            .custom_fields
            .and_then(|j| serde_json::from_value(j).ok())
            .unwrap_or_default(),
        content_model_id: r.content_model_id.map(|v| v as u32),
        parent_id: r.parent_id.map(|v| v as u32),
        depth: r.depth,
        path: r.path,
        translations: translations
            .into_iter()
            .map(page_translation_proto)
            .collect(),
        available_languages,
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

pub struct PageServiceImpl {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::page_service_server::PageService for PageServiceImpl {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListPageResponse>, Status> {
        let (rows, total) =
            fetch_paged(&self.state.db, pages::Entity::find(), &request.into_inner())
                .await
                .map_err(|e| Status::internal(e.message))?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations = page_translations::Entity::find()
                .filter(page_translations::Column::PageId.eq(r.id))
                .all(&self.state.db)
                .await
                .map_err(db_status)?;
            items.push(page_proto(r, translations));
        }
        Ok(Response::new(contentv1::ListPageResponse { items, total }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetPageRequest>,
    ) -> Result<Response<contentv1::Page>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_page_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = repo::page_by_id(&self.state.db, id as i64).await?;
        let translations = page_translations::Entity::find()
            .filter(page_translations::Column::PageId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(page_proto(row, translations)))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreatePageRequest>,
    ) -> Result<Response<contentv1::Page>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let now = store::now();
        let row = pages::ActiveModel {
            status: Set(status_from_i32("page", data.status.unwrap_or(1))),
            slug: Set(data.slug),
            author_id: Set(data.author_id.map(|v| v as i64)),
            author_name: Set(data.author_name),
            disallow_comment: Set(data.disallow_comment),
            redirect_url: Set(data.redirect_url),
            show_in_navigation: Set(data.show_in_navigation),
            sort_order: Set(data.sort_order.map(|v| v as i64)),
            template: Set(data.template),
            is_custom_template: Set(data.is_custom_template),
            thumbnail: Set(data.thumbnail),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&self.state.db)
        .await
        .map_err(db_status)?;
        for t in data.translations {
            page_translations::ActiveModel {
                page_id: Set(Some(row.id)),
                language_code: Set(t.language_code),
                title: Set(t.title),
                slug: Set(t.slug),
                cover_image: Set(t.cover_image),
                full_path: Set(t.full_path),
                seo: Set(seo_to_json(t.seo)),
                created_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(&self.state.db)
            .await
            .map_err(db_status)?;
        }
        let translations = page_translations::Entity::find()
            .filter(page_translations::Column::PageId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(page_proto(row, translations)))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdatePageRequest>,
    ) -> Result<Response<contentv1::Page>, Status> {
        let req = request.into_inner();
        let row = repo::page_by_id(&self.state.db, req.id as i64).await?;
        let mut a: pages::ActiveModel = row.into();
        let embedded = req.data.as_ref().map(|d| d.translations.clone());
        if let Some(data) = req.data {
            if let Some(v) = data.status {
                a.status = Set(status_from_i32("page", v));
            }
            if let Some(v) = data.slug {
                a.slug = Set(Some(v));
            }
            if let Some(v) = data.template {
                a.template = Set(Some(v));
            }
            if let Some(v) = data.thumbnail {
                a.thumbnail = Set(Some(v));
            }
            if let Some(v) = data.sort_order {
                a.sort_order = Set(Some(v as i64));
            }
        }
        a.updated_at = Set(Some(store::now()));
        let row = a.update(&self.state.db).await.map_err(db_status)?;
        if let Some(ts) = embedded {
            let now = store::now();
            for t in ts {
                upsert_page_translation_row(&self.state.db, row.id, t, now).await?;
            }
        }
        let translations = page_translations::Entity::find()
            .filter(page_translations::Column::PageId.eq(row.id))
            .all(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(page_proto(row, translations)))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeletePageRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_page_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        repo::delete_page(&self.state.db, id as i64).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn translation_exists(
        &self,
        request: Request<contentv1::PageTranslationExistsRequest>,
    ) -> Result<Response<contentv1::PageTranslationExistsResponse>, Status> {
        let req = request.into_inner();
        let exists =
            repo::page_translation_exists(&self.state.db, req.page_id as i64, &req.language_code)
                .await?;
        Ok(Response::new(contentv1::PageTranslationExistsResponse {
            exists,
        }))
    }

    async fn get_translation(
        &self,
        request: Request<contentv1::GetPageRequest>,
    ) -> Result<Response<contentv1::PageTranslation>, Status> {
        let req = request.into_inner();
        // The reference reads req.GetId(): a slug query degrades to id 0
        // and thus the zero translation (a missing translation is not an
        // error — the response is simply the empty message).
        let id = match req.query_by {
            Some(contentv1::get_page_request::QueryBy::Id(id)) => id as i64,
            _ => 0,
        };
        let row =
            repo::get_page_translation(&self.state.db, id, &req.locale.unwrap_or_default()).await?;
        Ok(Response::new(
            row.map(page_translation_proto).unwrap_or_default(),
        ))
    }

    async fn create_translation(
        &self,
        request: Request<contentv1::CreatePageTranslationRequest>,
    ) -> Result<Response<contentv1::PageTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::create_page_translation(
            &self.state.db,
            req.page_id as i64,
            repo::PageTranslationFields {
                language_code: data.language_code,
                title: data.title,
                slug: data.slug,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
        )
        .await?;
        Ok(Response::new(page_translation_proto(row)))
    }

    async fn update_translation(
        &self,
        request: Request<contentv1::UpdatePageTranslationRequest>,
    ) -> Result<Response<contentv1::PageTranslation>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let row = repo::update_page_translation(
            &self.state.db,
            req.id as i64,
            data.page_id.map(|v| v as i64).unwrap_or(0),
            // update_mask is accepted but not applied, like the other
            // update faces of this port; the repo writes only the fields
            // the reference's page-translation update writes.
            repo::PageTranslationFields {
                language_code: data.language_code,
                title: data.title,
                slug: data.slug,
                cover_image: data.cover_image,
                full_path: data.full_path,
                seo: seo_to_json(data.seo),
                created_by: data.created_by.map(|v| v as i64),
            },
            req.allow_missing.unwrap_or(false),
        )
        .await?;
        Ok(Response::new(page_translation_proto(row)))
    }

    async fn delete_translation(
        &self,
        request: Request<contentv1::DeletePageTranslationRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let query = match req.query_by {
            Some(contentv1::delete_page_translation_request::QueryBy::Id(id)) => {
                repo::PageTranslationQuery::Id(id as i64)
            }
            Some(contentv1::delete_page_translation_request::QueryBy::Identifier(identifier)) => {
                repo::PageTranslationQuery::Identifier {
                    page_id: identifier.page_id as i64,
                    language_code: identifier.language_code,
                }
            }
            None => return Err(bad("query_by required")),
        };
        repo::delete_page_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// The condition helper keeps the import set honest for future filters.
#[allow(dead_code)]
fn _unused(c: Condition) -> Condition {
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── status_from_i32 ─────────────────────────────────────────────

    #[test]
    fn status_from_i32_maps_every_member_of_every_enum() {
        let table = [
            ("post", 1, "POST_STATUS_DRAFT"),
            ("post", 2, "POST_STATUS_PUBLISHED"),
            ("post", 3, "POST_STATUS_SCHEDULED"),
            ("post", 4, "POST_STATUS_TRASHED"),
            ("page", 1, "PAGE_STATUS_DRAFT"),
            ("page", 2, "PAGE_STATUS_PUBLISHED"),
            ("page", 3, "PAGE_STATUS_ARCHIVED"),
            ("category", 1, "CATEGORY_STATUS_ACTIVE"),
            ("category", 2, "CATEGORY_STATUS_HIDDEN"),
            ("category", 3, "CATEGORY_STATUS_ARCHIVED"),
            ("tag", 1, "TAG_STATUS_ACTIVE"),
            ("tag", 2, "TAG_STATUS_HIDDEN"),
            ("tag", 3, "TAG_STATUS_ARCHIVED"),
        ];
        for (prefix, v, want) in table {
            assert_eq!(
                status_from_i32(prefix, v).as_deref(),
                Some(want),
                "{prefix}/{v}"
            );
        }
    }

    #[test]
    fn status_from_i32_unknown_values_and_prefixes_yield_none() {
        // Out-of-range values.
        for (prefix, v) in [
            ("post", 0),
            ("post", 5),
            ("page", 0),
            ("page", 4),
            ("category", 0),
            ("category", 4),
            ("tag", 0),
            ("tag", 4),
        ] {
            assert_eq!(status_from_i32(prefix, v), None, "{prefix}/{v}");
        }
        // Unknown enum families never map.
        assert_eq!(status_from_i32("unknown", 1), None);
        assert_eq!(status_from_i32("", 1), None);
        assert_eq!(status_from_i32("POST", 1), None);
    }

    // ── status_to_i32 ───────────────────────────────────────────────

    #[test]
    fn status_to_i32_round_trips_the_from_i32_table() {
        for prefix in ["post", "page", "category", "tag"] {
            for v in 1..=4 {
                if let Some(name) = status_from_i32(prefix, v) {
                    assert_eq!(status_to_i32(prefix, Some(name)), Some(v), "{prefix}/{v}");
                }
            }
        }
    }

    #[test]
    fn status_to_i32_missing_name_is_none_and_unknown_name_is_zero() {
        assert_eq!(status_to_i32("post", None), None);
        // The unknown-name arm reads 0 (the proto's UNSPECIFIED), the
        // prefix never disambiguates — the stored name decides.
        assert_eq!(status_to_i32("post", Some("NOT_A_STATUS".into())), Some(0));
        assert_eq!(
            status_to_i32("tag", Some("POST_STATUS_DRAFT".into())),
            Some(1)
        );
    }
}
