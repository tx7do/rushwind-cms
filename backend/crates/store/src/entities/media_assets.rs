//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "media_assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub tenant_id: Option<i64>,
    pub filename: Option<String>,
    pub r#type: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub storage_path: Option<String>,
    pub url: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration: Option<i64>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub processing_status: Option<String>,
    pub processing_error: Option<String>,
    pub file_hash: Option<String>,
    pub folder_id: Option<i64>,
    pub file_id: Option<i64>,
    pub reference_count: Option<i64>,
    pub is_private: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
