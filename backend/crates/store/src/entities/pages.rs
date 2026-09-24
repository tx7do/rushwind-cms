//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pages")]
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
    pub path: Option<String>,
    pub editor_type: Option<String>,
    pub tenant_id: Option<i64>,
    pub status: Option<String>,
    pub r#type: Option<String>,
    pub slug: Option<String>,
    pub author_id: Option<i64>,
    pub author_name: Option<String>,
    pub disallow_comment: Option<bool>,
    pub redirect_url: Option<String>,
    pub show_in_navigation: Option<bool>,
    pub template: Option<String>,
    pub is_custom_template: Option<bool>,
    pub thumbnail: Option<String>,
    pub custom_fields: Option<Json>,
    pub content_model_id: Option<i64>,
    pub depth: Option<i32>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
