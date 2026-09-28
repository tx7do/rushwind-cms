//! The tag service — CRUD with the translation sub-tables
//! (the reference's tag_service.go).

use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tonic::{Request, Response, Status};

use crate::data::tag_translation_repo as repo;
use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::{tag_translations, tags};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

use crate::data::{tag_repo, tag_translation_repo};
use crate::service::content_support::{seo_of, seo_to_json, status_from_i32, status_to_i32};

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

pub struct TagService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::tag_service_server::TagService for TagService {
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
        let row = tag_repo::tag_by_id(&self.state.db, id as i64).await?;
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
        let row = tag_repo::tag_by_id(&self.state.db, req.id as i64).await?;
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
        tag_repo::delete_tag(&self.state.db, id as i64).await?;
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
        let row = tag_translation_repo::get_tag_translation(
            &self.state.db,
            id,
            &req.locale.unwrap_or_default(),
        )
        .await?;
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
        tag_translation_repo::delete_tag_translation(&self.state.db, query).await?;
        Ok(Response::new(pbjson_types::Empty {}))
    }
}

/// The tag translation embedded upsert (the named-translation shape).
pub(crate) async fn upsert_tag_translation_row(
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
