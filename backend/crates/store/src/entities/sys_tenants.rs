//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_tenants")]
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
    pub name: Option<String>,
    pub code: Option<String>,
    pub logo_url: Option<String>,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub admin_user_id: Option<i64>,
    pub status: Option<String>,
    pub r#type: Option<String>,
    pub audit_status: Option<String>,
    pub subscription_at: Option<DateTimeWithTimeZone>,
    pub unsubscribe_at: Option<DateTimeWithTimeZone>,
    pub subscription_plan: Option<String>,
    pub expired_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
