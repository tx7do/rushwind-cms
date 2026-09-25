//! The post service — CRUD with the translation sub-tables
//! (the reference's post_service.go).


use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::post_translation_repo as repo;
use crate::state::{bad, db_status, not_found, ts_to_proto, AppState};
use store::entities::{
    post_categories, post_tags, post_translations, posts,
};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

use crate::data::{post_repo, post_translation_repo};
use crate::service::content_support::{seo_of, seo_to_json, status_from_i32, status_to_i32};

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

pub struct PostService {
    pub state: Arc<AppState>,
}





#[async_trait::async_trait]
impl contentv1::post_service_server::PostService for PostService {
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
        let row = post_repo::post_by_id(&self.state.db, req.id as i64).await?;
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
        post_repo::delete_post(&self.state.db, id as i64).await?;
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
            post_translation_repo::get_post_translation(&self.state.db, id, &req.locale.unwrap_or_default()).await?;
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
        let row = post_translation_repo::update_post_translation(
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
        post_translation_repo::delete_post_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

/// The embedded-array translation upsert shared by the update faces
/// (the reference's repo semantics): a (main id, language) row is
/// updated in place, a missing language inserts fresh.
pub(crate) async fn upsert_post_translation_row(
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
