//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "categories")]
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
    pub tenant_id: Option<i64>,
    pub status: Option<String>,
    pub is_nav: Option<bool>,
    pub icon: Option<String>,
    pub code: Option<String>,
    pub thumbnail: Option<String>,
    pub post_count: Option<i64>,
    pub direct_post_count: Option<i64>,
    pub depth: Option<i32>,
    pub custom_fields: Option<Json>,
    pub content_model_id: Option<i64>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
