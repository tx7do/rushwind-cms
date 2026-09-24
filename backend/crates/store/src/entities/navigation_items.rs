//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "navigation_items")]
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
    pub link_type: Option<String>,
    pub navigation_id: Option<i64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub object_id: Option<i64>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_open_new_tab: Option<bool>,
    pub is_invalid: Option<bool>,
    pub required_permission: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
