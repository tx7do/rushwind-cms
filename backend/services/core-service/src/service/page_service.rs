//! The page service — CRUD with the translation sub-tables
//! (the reference's page_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::page_translation_repo as repo;
use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::{page_translations, pages};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

use crate::data::{page_repo, page_translation_repo};
use crate::service::content_support::{seo_of, seo_to_json, status_from_i32, status_to_i32};

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

pub struct PageService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::page_service_server::PageService for PageService {
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
        let row = page_repo::page_by_id(&self.state.db, id as i64).await?;
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
        let row = page_repo::page_by_id(&self.state.db, req.id as i64).await?;
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
        page_repo::delete_page(&self.state.db, id as i64).await?;
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
        let row = page_translation_repo::get_page_translation(
            &self.state.db,
            id,
            &req.locale.unwrap_or_default(),
        )
        .await?;
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
        let row = page_translation_repo::update_page_translation(
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

/// The page translation embedded upsert.
pub(crate) async fn upsert_page_translation_row(
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
