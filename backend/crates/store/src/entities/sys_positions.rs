//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_positions")]
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
    pub remark: Option<String>,
    pub tenant_id: Option<i64>,
    pub status: String,
    pub name: String,
    pub code: String,
    pub org_unit_id: i64,
    pub reports_to_position_id: Option<i64>,
    pub description: Option<String>,
    pub job_family: Option<String>,
    pub job_grade: Option<String>,
    pub level: Option<i32>,
    pub headcount: i64,
    pub is_key_position: bool,
    pub r#type: String,
    pub start_at: Option<DateTimeWithTimeZone>,
    pub end_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
