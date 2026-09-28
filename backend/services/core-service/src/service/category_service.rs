//! The category service — CRUD with the translation sub-tables
//! (the reference's category_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::category_translation_repo as repo;
use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::{categories, category_translations};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

use crate::data::{category_repo, category_translation_repo};
use crate::service::content_support::{seo_of, seo_to_json, status_from_i32, status_to_i32};

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

pub struct CategoryService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::category_service_server::CategoryService for CategoryService {
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
        let row = category_repo::category_by_id(&self.state.db, id as i64).await?;
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
        let row = category_repo::category_by_id(&self.state.db, req.id as i64).await?;
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
        category_repo::delete_category(&self.state.db, id as i64).await?;
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
        let row = category_translation_repo::get_category_translation(
            &self.state.db,
            id,
            &req.locale.unwrap_or_default(),
        )
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
        category_translation_repo::delete_category_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

/// The category translation embedded upsert (shared shape with tags).
pub(crate) async fn upsert_category_translation_row(
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
