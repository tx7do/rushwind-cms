//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_user_org_units")]
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
    pub remark: Option<String>,
    pub user_id: i64,
    pub org_unit_id: i64,
    pub position_id: Option<i64>,
    pub start_at: Option<DateTimeWithTimeZone>,
    pub end_at: Option<DateTimeWithTimeZone>,
    pub assigned_at: Option<DateTimeWithTimeZone>,
    pub assigned_by: Option<i64>,
    pub is_primary: bool,
    pub status: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
