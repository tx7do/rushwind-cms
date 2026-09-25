//! The content-model face: model + field-definition CRUD with their
//! translation rows and the relation-config JSON round-trip (the
//! reference's modeler service).
use std::sync::Arc;

use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait, Set, TransactionTrait};
use tonic::{Request, Response, Status};

use crate::state::{bad, db_status, ts_to_proto, AppState};
use store::entities::{
    content_model_translations, content_models, field_definition_translations, field_definitions,
};
use store::paging::fetch_paged;

use proto::proto::content::service::v1 as contentv1;

// ── ContentModel ─────────────────────────────────────────────────────

fn content_model_proto(r: content_models::Model) -> contentv1::ContentModel {
    contentv1::ContentModel {
        id: Some(r.id as u32),
        name: r.name,
        code: r.code,
        description: r.description,
        sort_order: r.sort_order.map(|v| v as u32),
        created_at: r.created_at.and_then(ts_to_proto),
        updated_at: r.updated_at.and_then(ts_to_proto),
        ..Default::default()
    }
}

/// The varchar field type → its proto ordinal.
fn field_type_enum(s: &str) -> Option<i32> {
    contentv1::field_definition::Type::from_str_name(s).map(|e| e as i32)
}

/// The proto ordinal → the varchar field type the golden schema stores.
fn field_type_name(v: i32) -> Option<String> {
    contentv1::field_definition::Type::try_from(v)
        .ok()
        .map(|e| e.as_str_name().to_string())
}

fn json_to_options(v: &sea_orm::JsonValue) -> std::collections::HashMap<String, String> {
    v.as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

/// The typed-JSON relation config ↔ its golden-JSON keys
/// (filter_category_id carries the reference's omitempty).
fn relation_config_from_json(v: &sea_orm::JsonValue) -> Option<contentv1::RelationConfig> {
    let obj = v.as_object()?;
    Some(contentv1::RelationConfig {
        target_entity_type: obj
            .get("target_entity_type")
            .and_then(|x| x.as_str())
            .map(str::to_owned),
        allow_cross_tenant: obj.get("allow_cross_tenant").and_then(|x| x.as_bool()),
        filter_category_id: obj
            .get("filter_category_id")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32),
    })
}

fn relation_config_to_json(rc: &contentv1::RelationConfig) -> sea_orm::JsonValue {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "target_entity_type".into(),
        serde_json::json!(rc.target_entity_type.clone().unwrap_or_default()),
    );
    obj.insert(
        "allow_cross_tenant".into(),
        serde_json::json!(rc.allow_cross_tenant.unwrap_or(false)),
    );
    if let Some(v) = rc.filter_category_id.filter(|v| *v != 0) {
        obj.insert("filter_category_id".into(), serde_json::json!(v));
    }
    serde_json::Value::Object(obj)
}

fn field_translation_proto(
    r: field_definition_translations::Model,
) -> contentv1::FieldDefinitionTranslation {
    contentv1::FieldDefinitionTranslation {
        id: Some(r.id as u32),
        field_definition_id: r.field_definition_id.map(|v| v as u32),
        language_code: r.language_code,
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        created_by: r.created_by.map(|v| v as u32),
        ..Default::default()
    }
}

fn field_definition_proto(
    r: field_definitions::Model,
    translations: Vec<field_definition_translations::Model>,
) -> contentv1::FieldDefinition {
    contentv1::FieldDefinition {
        id: Some(r.id as u32),
        content_model_id: r.content_model_id.map(|v| v as u32),
        name: r.name,
        r#type: r.r#type.as_deref().and_then(field_type_enum),
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        is_required: r.is_required,
        validation_regex: r.validation_regex,
        options: r.options.as_ref().map(json_to_options).unwrap_or_default(),
        relation_config: r
            .relation_config
            .as_ref()
            .and_then(relation_config_from_json),
        sort_order: r.sort_order.map(|v| v as u32),
        translations: translations
            .into_iter()
            .map(field_translation_proto)
            .collect(),
        created_by: r.created_by.map(|v| v as u32),
        ..Default::default()
    }
}

/// The model's field definitions + their translations, recreated
/// wholesale (the reference's replace-fields convention).
async fn batch_create_fields<C: ConnectionTrait>(
    db: &C,
    model_id: i64,
    fields: &[contentv1::FieldDefinition],
) -> Result<(), Status> {
    for f in fields {
        if f.name.as_deref().unwrap_or_default().is_empty() {
            return Err(bad("field definition name is required"));
        }
        let mut a = field_definitions::ActiveModel {
            content_model_id: Set(Some(model_id)),
            name: Set(f.name.clone()),
            label: Set(f.label.clone()),
            description: Set(f.description.clone()),
            placeholder: Set(f.placeholder.clone()),
            is_required: Set(f.is_required),
            validation_regex: Set(f.validation_regex.clone()),
            sort_order: Set(f.sort_order.map(|v| v as i64)),
            created_by: Set(f.created_by.map(|v| v as i64)),
            created_at: Set(Some(store::now())),
            ..Default::default()
        };
        if let Some(name) = f.r#type.and_then(field_type_name) {
            a.r#type = Set(Some(name));
        }
        if !f.options.is_empty() {
            a.options = Set(serde_json::to_value(&f.options).ok());
        }
        if let Some(rc) = &f.relation_config {
            a.relation_config = Set(Some(relation_config_to_json(rc)));
        }
        let created = content_model_repo::insert_field_definitions(db, a).await?;
        for tr in &f.translations {
            content_model_repo::insert_field_definition_translations(
                db,
                field_definition_translations::ActiveModel {
                    field_definition_id: Set(Some(created.id)),
                    language_code: Set(tr.language_code.clone()),
                    label: Set(tr.label.clone()),
                    description: Set(tr.description.clone()),
                    placeholder: Set(tr.placeholder.clone()),
                    created_by: Set(tr.created_by.map(|v| v as i64)),
                    created_at: Set(Some(store::now())),
                    ..Default::default()
                },
            )
            .await?;
        }
    }
    Ok(())
}

async fn batch_create_model_translations<C: ConnectionTrait>(
    db: &C,
    model_id: i64,
    translations: &[contentv1::ContentModelTranslation],
) -> Result<(), Status> {
    for tr in translations {
        if tr.language_code.as_deref().unwrap_or_default().is_empty() {
            continue;
        }
        content_model_repo::insert_content_model_translations(
            db,
            content_model_translations::ActiveModel {
                content_model_id: Set(Some(model_id)),
                language_code: Set(tr.language_code.clone()),
                name: Set(tr.name.clone()),
                description: Set(tr.description.clone()),
                created_by: Set(tr.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok(())
}

pub struct ContentModelService {
    pub state: Arc<AppState>,
}

#[async_trait::async_trait]
impl contentv1::content_model_service_server::ContentModelService for ContentModelService {
    async fn list(
        &self,
        request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::ListContentModelResponse>, Status> {
        let (rows, total) = fetch_paged(
            &self.state.db,
            content_models::Entity::find(),
            &request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.message))?;
        Ok(Response::new(contentv1::ListContentModelResponse {
            items: rows.into_iter().map(content_model_proto).collect(),
            total,
        }))
    }

    async fn get(
        &self,
        request: Request<contentv1::GetContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(contentv1::get_content_model_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let row = content_model_repo::content_models_by_id(&self.state.db, id as i64).await?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn count(
        &self,
        _request: Request<proto::proto::pagination::PagingRequest>,
    ) -> Result<Response<contentv1::CountContentModelResponse>, Status> {
        let total = content_models::Entity::find()
            .count(&self.state.db)
            .await
            .map_err(db_status)?;
        Ok(Response::new(contentv1::CountContentModelResponse {
            count: total,
        }))
    }

    async fn create(
        &self,
        request: Request<contentv1::CreateContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = content_model_repo::insert_content_models(
            &txn,
            content_models::ActiveModel {
                tenant_id: Set(data.tenant_id.map(|v| v as i64)),
                name: Set(data.name),
                code: Set(data.code),
                description: Set(data.description),
                sort_order: Set(data.sort_order.map(|v| v as i64)),
                created_by: Set(data.created_by.map(|v| v as i64)),
                created_at: Set(Some(store::now())),
                ..Default::default()
            },
        )
        .await?;
        batch_create_fields(&txn, row.id, &data.fields).await?;
        batch_create_model_translations(&txn, row.id, &data.translations).await?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn update(
        &self,
        request: Request<contentv1::UpdateContentModelRequest>,
    ) -> Result<Response<contentv1::ContentModel>, Status> {
        let req = request.into_inner();
        let Some(data) = req.data else {
            return Err(bad("data required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let row = content_model_repo::content_models_by_id(&txn, req.id as i64).await?;
        let mut a: content_models::ActiveModel = row.into();
        if let Some(v) = data.name {
            a.name = Set(Some(v));
        }
        if let Some(v) = data.description {
            a.description = Set(Some(v));
        }
        if let Some(v) = data.sort_order {
            a.sort_order = Set(Some(v as i64));
        }
        a.updated_at = Set(Some(store::now()));
        let row = content_model_repo::update_content_models(&txn, a).await?;
        // 字段定义与翻译整体替换（对位 replace-fields / replace-translations）
        let old_field_ids = content_model_repo::content_model_field_ids(&txn, req.id as i64).await?;
        content_model_repo::delete_field_definition_translations_of(&txn, &old_field_ids).await?;
        content_model_repo::delete_field_definitions_of(&txn, req.id as i64).await?;
        batch_create_fields(&txn, req.id as i64, &data.fields).await?;
        content_model_repo::delete_content_model_translations_of(&txn, req.id as i64).await?;
        batch_create_model_translations(&txn, req.id as i64, &data.translations).await?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(content_model_proto(row)))
    }

    async fn delete(
        &self,
        request: Request<contentv1::DeleteContentModelRequest>,
    ) -> Result<Response<pbjson_types::Empty>, Status> {
        let req = request.into_inner();
        let Some(contentv1::delete_content_model_request::QueryBy::Id(id)) = req.query_by else {
            return Err(bad("query_by required"));
        };
        let txn = self.state.db.begin().await.map_err(db_status)?;
        let model_id = id as i64;
        // 级联清理：字段翻译 → 字段定义 → 模型翻译 → 模型
        let field_ids = content_model_repo::content_model_field_ids(&txn, model_id).await?;
        content_model_repo::delete_field_definition_translations_of(&txn, &field_ids).await?;
        content_model_repo::delete_field_definitions_of(&txn, model_id).await?;
        content_model_repo::delete_content_model_translations_of(&txn, model_id).await?;
        content_model_repo::content_models_by_id(&txn, model_id).await?;
        content_models::Entity::delete_by_id(model_id)
            .exec(&txn)
            .await
            .map_err(db_status)?;
        txn.commit().await.map_err(db_status)?;
        Ok(Response::new(pbjson_types::Empty {}))
    }

    async fn list_field_definitions(
        &self,
        request: Request<contentv1::ListFieldDefinitionsRequest>,
    ) -> Result<Response<contentv1::ListFieldDefinitionsResponse>, Status> {
        let req = request.into_inner();
        if req.content_model_id == 0 {
            return Err(bad("invalid parameter"));
        }
        let rows = content_model_repo::field_definitions_of(&self.state.db, req.content_model_id as i64).await?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations = content_model_repo::field_definition_translations_of(&self.state.db, r.id).await?;
            items.push(field_definition_proto(r, translations));
        }
        let total = items.len() as u64;
        Ok(Response::new(contentv1::ListFieldDefinitionsResponse {
            items,
            total,
        }))
    }
}
use crate::data::{content_model_repo};

