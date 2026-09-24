//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_org_units")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub status: String,
    pub sort_order: Option<i64>,
    pub tenant_id: Option<i64>,
    pub remark: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub name: String,
    pub code: Option<String>,
    pub leader_id: Option<i64>,
    pub r#type: String,
    pub business_scopes: Option<Json>,
    pub external_id: Option<String>,
    pub is_legal_entity: Option<bool>,
    pub registration_number: Option<String>,
    pub tax_id: Option<String>,
    pub legal_entity_org_id: Option<i64>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub start_at: Option<DateTimeWithTimeZone>,
    pub end_at: Option<DateTimeWithTimeZone>,
    pub contact_user_id: Option<i64>,
    pub permission_tags: Option<Json>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
