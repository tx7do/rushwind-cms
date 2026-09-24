//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "comments")]
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
    pub content_type: Option<String>,
    pub object_id: Option<i64>,
    pub content: Option<String>,
    pub author_id: Option<i64>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
    pub author_type: Option<String>,
    pub status: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub user_agent: Option<String>,
    pub detected_language: Option<String>,
    pub is_spam: Option<bool>,
    pub is_sticky: Option<bool>,
    pub reply_to_id: Option<i64>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
