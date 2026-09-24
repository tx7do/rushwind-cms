//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_user_credentials")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub tenant_id: Option<i64>,
    pub user_id: Option<i64>,
    pub identity_type: Option<String>,
    pub identifier: Option<String>,
    pub credential_type: Option<String>,
    pub credential: Option<String>,
    pub is_primary: Option<bool>,
    pub status: Option<String>,
    pub extra_info: Option<Json>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub activate_token_hash: Option<String>,
    pub activate_token_expires_at: Option<DateTimeWithTimeZone>,
    pub activate_token_used_at: Option<DateTimeWithTimeZone>,
    pub reset_token_hash: Option<String>,
    pub reset_token_expires_at: Option<DateTimeWithTimeZone>,
    pub reset_token_used_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
