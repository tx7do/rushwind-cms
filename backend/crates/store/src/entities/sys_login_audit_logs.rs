//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_login_audit_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub tenant_id: Option<i64>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub geo_location: Option<Json>,
    pub session_id: Option<String>,
    pub device_info: Option<Json>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub action_type: Option<String>,
    pub status: Option<String>,
    pub login_method: Option<String>,
    pub failure_reason: Option<String>,
    pub mfa_status: Option<String>,
    pub risk_score: Option<i64>,
    pub risk_level: Option<String>,
    pub risk_factors: Option<Json>,
    pub log_hash: Option<String>,
    pub signature: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
