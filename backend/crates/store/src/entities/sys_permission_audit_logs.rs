//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_permission_audit_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub tenant_id: Option<i64>,
    pub operator_id: Option<i64>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub action: Option<String>,
    pub old_value: Option<Json>,
    pub new_value: Option<Json>,
    pub ip_address: String,
    pub request_id: String,
    pub reason: String,
    pub log_hash: Option<String>,
    pub signature: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
