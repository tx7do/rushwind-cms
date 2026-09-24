//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub remark: Option<String>,
    pub tenant_id: Option<i64>,
    pub provider: Option<String>,
    pub bucket_name: Option<String>,
    pub file_directory: Option<String>,
    pub file_guid: Option<String>,
    pub save_file_name: Option<String>,
    pub file_name: Option<String>,
    pub extension: Option<String>,
    pub size: Option<i64>,
    pub size_format: Option<String>,
    pub link_url: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
