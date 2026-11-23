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
        id: Some(r.id),
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
        filter_category_id: obj.get("filter_category_id").and_then(|x| x.as_i64()),
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
        id: Some(r.id),
        field_definition_id: r.field_definition_id,
        language_code: r.language_code,
        label: r.label,
        description: r.description,
        placeholder: r.placeholder,
        created_by: r.created_by,
        ..Default::default()
    }
}

fn field_definition_proto(
    r: field_definitions::Model,
    translations: Vec<field_definition_translations::Model>,
) -> contentv1::FieldDefinition {
    contentv1::FieldDefinition {
        id: Some(r.id),
        content_model_id: r.content_model_id,
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
        created_by: r.created_by,
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
            created_by: Set(f.created_by),
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
                    created_by: Set(tr.created_by),
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
                created_by: Set(tr.created_by),
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
            total: total as i64,
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
        let row = content_model_repo::content_models_by_id(&self.state.db, id).await?;
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
            count: total as i64,
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
                tenant_id: Set(data.tenant_id),
                name: Set(data.name),
                code: Set(data.code),
                description: Set(data.description),
                sort_order: Set(data.sort_order.map(|v| v as i64)),
                created_by: Set(data.created_by),
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
        let row = content_model_repo::content_models_by_id(&txn, req.id).await?;
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
        let old_field_ids = content_model_repo::content_model_field_ids(&txn, req.id).await?;
        content_model_repo::delete_field_definition_translations_of(&txn, &old_field_ids).await?;
        content_model_repo::delete_field_definitions_of(&txn, req.id).await?;
        batch_create_fields(&txn, req.id, &data.fields).await?;
        content_model_repo::delete_content_model_translations_of(&txn, req.id).await?;
        batch_create_model_translations(&txn, req.id, &data.translations).await?;
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
        let model_id = id;
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
        let rows =
            content_model_repo::field_definitions_of(&self.state.db, req.content_model_id).await?;
        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let translations =
                content_model_repo::field_definition_translations_of(&self.state.db, r.id).await?;
            items.push(field_definition_proto(r, translations));
        }
        let total = items.len() as u64;
        Ok(Response::new(contentv1::ListFieldDefinitionsResponse {
            items,
            total: total as i64,
        }))
    }
}
use crate::data::content_model_repo;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── the enum / JSON bridges ───────────────────────────────────────

    #[test]
    fn field_type_bridges_the_varchar_column() {
        // Every declared member round-trips through its storage name.
        for v in 0..=7 {
            let name = field_type_name(v).unwrap_or_else(|| panic!("{v} is a declared member"));
            assert_eq!(field_type_enum(&name), Some(v), "{name}");
        }
        // The zero ordinal is a real member (TEXT), not "absent".
        assert_eq!(field_type_name(0).as_deref(), Some("FIELD_TYPE_TEXT"));
        // Undeclared ordinals and foreign names answer None — the names
        // are case-sensitive.
        for v in [-1, 8, 99] {
            assert_eq!(field_type_name(v), None, "{v}");
        }
        for name in ["TEXT", "field_type_text", ""] {
            assert_eq!(field_type_enum(name), None, "{name:?}");
        }
    }

    #[test]
    fn options_keep_the_string_values_only() {
        let stored = serde_json::json!({
            "a": "1", "b": 2, "c": true, "d": null, "e": ["x"], "f": {"g": "h"}
        });
        assert_eq!(
            json_to_options(&stored),
            HashMap::from([("a".to_string(), "1".to_string())])
        );
        // A non-object column reads as no options.
        for stored in [
            serde_json::json!(["a"]),
            serde_json::json!("x"),
            serde_json::json!(null),
        ] {
            assert!(json_to_options(&stored).is_empty(), "{stored}");
        }
    }

    #[test]
    fn relation_config_round_trips_the_golden_json() {
        let stored = serde_json::json!({
            "target_entity_type": "post",
            "allow_cross_tenant": true,
            "filter_category_id": 12
        });
        let config = relation_config_from_json(&stored).unwrap();
        assert_eq!(config.target_entity_type.as_deref(), Some("post"));
        assert_eq!(config.allow_cross_tenant, Some(true));
        assert_eq!(config.filter_category_id, Some(12));
        assert_eq!(relation_config_to_json(&config), stored);

        // filter_category_id carries the omitempty: a zero is not written
        // back, and so reads back as absent.
        let zeroed = contentv1::RelationConfig {
            target_entity_type: Some("page".into()),
            allow_cross_tenant: Some(false),
            filter_category_id: Some(0),
        };
        assert_eq!(
            relation_config_to_json(&zeroed),
            serde_json::json!({"target_entity_type": "page", "allow_cross_tenant": false})
        );
        assert_eq!(
            relation_config_from_json(&relation_config_to_json(&zeroed)).unwrap(),
            contentv1::RelationConfig {
                filter_category_id: None,
                ..zeroed
            }
        );
        // The other two keys always serialize — absent ones as their zero
        // values.
        assert_eq!(
            relation_config_to_json(&contentv1::RelationConfig::default()),
            serde_json::json!({"target_entity_type": "", "allow_cross_tenant": false})
        );
    }

    #[test]
    fn relation_config_reads_its_members_defensively() {
        // Not an object → not a config.
        for stored in [
            serde_json::json!("post"),
            serde_json::json!(["post"]),
            serde_json::json!(null),
        ] {
            assert!(relation_config_from_json(&stored).is_none(), "{stored}");
        }
        // A member of the wrong JSON type reads as absent.
        let config = relation_config_from_json(&serde_json::json!({
            "target_entity_type": 7,
            "allow_cross_tenant": "yes",
            "filter_category_id": "12"
        }))
        .unwrap();
        assert_eq!(config, contentv1::RelationConfig::default());
    }

    // ── the row → proto mapping ──────────────────────────────────────

    /// A field-definition row with every optional column absent.
    fn empty_row() -> field_definitions::Model {
        field_definitions::Model {
            id: 0,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
            sort_order: None,
            tenant_id: None,
            content_model_id: None,
            name: None,
            r#type: None,
            label: None,
            description: None,
            placeholder: None,
            is_required: None,
            validation_regex: None,
            options: None,
            relation_config: None,
        }
    }

    #[test]
    fn field_definition_proto_maps_the_row() {
        let row = field_definitions::Model {
            id: 3,
            created_by: Some(1),
            sort_order: Some(4),
            tenant_id: Some(2),
            content_model_id: Some(7),
            name: Some("title".into()),
            r#type: Some("FIELD_TYPE_TEXT".into()),
            label: Some("Title".into()),
            placeholder: Some("Title…".into()),
            is_required: Some(true),
            validation_regex: Some("^.{1,80}$".into()),
            options: Some(serde_json::json!({"max": "80"})),
            relation_config: Some(serde_json::json!({
                "target_entity_type": "post",
                "allow_cross_tenant": true
            })),
            ..empty_row()
        };
        let translation = field_definition_translations::Model {
            id: 31,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
            field_definition_id: Some(3),
            language_code: Some("zh-CN".into()),
            label: Some("标题".into()),
            description: Some("d".into()),
            placeholder: Some("p".into()),
        };
        let p = field_definition_proto(row, vec![translation]);
        assert_eq!(p.id, Some(3));
        assert_eq!(p.content_model_id, Some(7));
        assert_eq!(p.name.as_deref(), Some("title"));
        assert_eq!(p.r#type, field_type_enum("FIELD_TYPE_TEXT"));
        assert_eq!(p.label.as_deref(), Some("Title"));
        assert_eq!(p.description, None);
        assert_eq!(p.placeholder.as_deref(), Some("Title…"));
        assert_eq!(p.is_required, Some(true));
        assert_eq!(p.validation_regex.as_deref(), Some("^.{1,80}$"));
        assert_eq!(
            p.options,
            HashMap::from([("max".to_string(), "80".to_string())])
        );
        let relation = p.relation_config.unwrap();
        assert_eq!(relation.target_entity_type.as_deref(), Some("post"));
        assert_eq!(relation.allow_cross_tenant, Some(true));
        assert_eq!(p.sort_order, Some(4));
        assert_eq!(p.created_by, Some(1));
        assert_eq!(p.translations.len(), 1);
        assert_eq!(p.translations[0].id, Some(31));
        assert_eq!(p.translations[0].language_code.as_deref(), Some("zh-CN"));
        assert_eq!(p.translations[0].label.as_deref(), Some("标题"));

        // An unknown stored type and absent JSON columns read as absent —
        // never as the zero member or an empty object.
        let p = field_definition_proto(
            field_definitions::Model {
                id: 4,
                r#type: Some("WAT".into()),
                ..empty_row()
            },
            Vec::new(),
        );
        assert_eq!(p.r#type, None);
        assert!(p.options.is_empty());
        assert!(p.relation_config.is_none());
        assert!(p.translations.is_empty());
        assert_eq!(p.sort_order, None);
        assert_eq!(p.is_required, None);
    }

    #[test]
    fn content_model_proto_maps_the_row() {
        let p = content_model_proto(content_models::Model {
            id: 7,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            created_by: Some(1),
            updated_by: None,
            deleted_by: None,
            sort_order: Some(3),
            tenant_id: Some(2),
            name: Some("文章".into()),
            code: Some("post".into()),
            description: Some("d".into()),
        });
        assert_eq!(p.id, Some(7));
        assert_eq!(p.name.as_deref(), Some("文章"));
        assert_eq!(p.code.as_deref(), Some("post"));
        assert_eq!(p.description.as_deref(), Some("d"));
        assert_eq!(p.sort_order, Some(3));
        assert!(p.created_at.is_none());
    }
}
