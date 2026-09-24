//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "field_definitions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub sort_order: Option<i64>,
    pub tenant_id: Option<i64>,
    pub content_model_id: Option<i64>,
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    pub is_required: Option<bool>,
    pub validation_regex: Option<String>,
    pub options: Option<Json>,
    pub relation_config: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
