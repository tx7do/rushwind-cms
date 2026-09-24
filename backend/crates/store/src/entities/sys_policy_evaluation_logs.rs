//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_policy_evaluation_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub tenant_id: Option<i64>,
    pub user_id: i64,
    pub membership_id: i64,
    pub permission_id: i64,
    pub policy_id: Option<i64>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub result: bool,
    pub effect_details: Option<String>,
    pub scope_sql: Option<String>,
    pub ip_address: Option<String>,
    pub trace_id: Option<String>,
    pub evaluation_context: Option<String>,
    pub log_hash: Option<String>,
    pub signature: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
