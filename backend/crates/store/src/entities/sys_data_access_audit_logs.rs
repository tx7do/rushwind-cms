//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_data_access_audit_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub tenant_id: Option<i64>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub geo_location: Option<Json>,
    pub device_info: Option<Json>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub data_source: Option<String>,
    pub table_name: Option<String>,
    pub data_id: Option<String>,
    pub access_type: Option<String>,
    pub sql_digest: Option<String>,
    pub sql_text: Option<String>,
    pub affected_rows: Option<i64>,
    pub latency_ms: Option<i64>,
    pub success: Option<bool>,
    pub sensitive_level: Option<String>,
    pub data_masked: Option<bool>,
    pub masking_rules: Option<String>,
    pub business_purpose: Option<String>,
    pub data_category: Option<String>,
    pub db_user: Option<String>,
    pub log_hash: Option<String>,
    pub signature: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
