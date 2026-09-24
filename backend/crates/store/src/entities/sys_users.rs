//! Generated from the golden DDL (sql/schema.sql) — DO NOT EDIT.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub remark: Option<String>,
    pub tenant_id: Option<i64>,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub realname: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub telephone: Option<String>,
    pub avatar: Option<String>,
    pub address: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
    pub gender: Option<String>,
    pub last_login_at: Option<DateTimeWithTimeZone>,
    pub last_login_ip: Option<String>,
    pub locked_until: Option<DateTimeWithTimeZone>,
    pub status: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
