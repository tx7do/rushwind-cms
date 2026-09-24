//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
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
    pub editor_type: Option<String>,
    pub tenant_id: Option<i64>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub disallow_comment: Option<bool>,
    pub in_progress: Option<bool>,
    pub auto_summary: Option<bool>,
    pub is_featured: Option<bool>,
    pub author_id: Option<i64>,
    pub author_name: Option<String>,
    pub thumbnail: Option<String>,
    pub password_hash: Option<String>,
    pub custom_fields: Option<Json>,
    pub publish_time: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
