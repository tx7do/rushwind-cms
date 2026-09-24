//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_api_audit_logs")]
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
    pub referer: Option<String>,
    pub app_version: Option<String>,
    pub http_method: Option<String>,
    pub path: Option<String>,
    pub request_uri: Option<String>,
    pub api_module: Option<String>,
    pub api_operation: Option<String>,
    pub api_description: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub latency_ms: Option<i64>,
    pub success: Option<bool>,
    pub status_code: Option<i64>,
    pub reason: Option<String>,
    pub request_header: Option<String>,
    pub request_body: Option<String>,
    pub response: Option<String>,
    pub log_hash: Option<String>,
    pub signature: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
