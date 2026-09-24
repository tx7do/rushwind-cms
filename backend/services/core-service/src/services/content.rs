//! The content domain services: Post, Category, Tag, Page — CRUD with
//! the translation sub-tables (per-language rows), the category/tag
//! relation tables, and the enum-name status mapping the golden schema
//! stores.

use std::sync::Arc;

use sea_orm::sea_query::Condition;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

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
            ("category", 1) => "CATEGORY_STATUS_DRAFT",
            ("category", 2) => "CATEGORY_STATUS_PUBLISHED",
            ("tag", 1) => "TAG_STATUS_NORMAL",
            ("tag", 2) => "TAG_STATUS_DISABLED",
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

fn post_translation_proto(r: post_translations::Model) -> contentv1::PostTranslation {
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

fn post_proto(
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
            status: Set(status_from_i32("post", data.status.unwrap_or(1))),
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
        let row = posts::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("post"))?;
        let mut a: posts::ActiveModel = row.into();
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
        let (translations, category_ids, tag_ids) = post_relations(&self.state.db, row.id).await?;
        Ok(Response::new(post_proto(
            row,
            translations,
            category_ids,
            tag_ids,
        )))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeletePostRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_post_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        posts::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Category ─────────────────────────────────────────────────────────

fn category_translation_proto(r: category_translations::Model) -> contentv1::CategoryTranslation {
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

fn category_proto(
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
        let row = categories::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("category"))?;
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
        let row = categories::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("category"))?;
        let mut a: categories::ActiveModel = row.into();
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
        categories::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Tag ──────────────────────────────────────────────────────────────

fn tag_translation_proto(r: tag_translations::Model) -> contentv1::TagTranslation {
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

fn tag_proto(r: tags::Model, translations: Vec<tag_translations::Model>) -> contentv1::Tag {
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
        let row = tags::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tag"))?;
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
        let row = tags::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("tag"))?;
        let mut a: tags::ActiveModel = row.into();
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
        tags::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// ── Page ─────────────────────────────────────────────────────────────

fn page_translation_proto(r: page_translations::Model) -> contentv1::PageTranslation {
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

fn page_proto(r: pages::Model, translations: Vec<page_translations::Model>) -> contentv1::Page {
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
        let row = pages::Entity::find_by_id(id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("page"))?;
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
        let row = pages::Entity::find_by_id(req.id as i64)
            .one(&self.state.db)
            .await
            .map_err(db_status)?
            .ok_or_else(|| not_found("page"))?;
        let mut a: pages::ActiveModel = row.into();
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
        pages::Entity::delete_by_id(id as i64)
            .exec(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

// The condition helper keeps the import set honest for future filters.
#[allow(dead_code)]
fn _unused(c: Condition) -> Condition {
    c
}
