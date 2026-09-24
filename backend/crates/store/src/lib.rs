//! The shared data layer of the two CMS deployments: the generated
//! entity tree, the paging/filter select assembly, the bootstrap
//! migration (golden DDL + demo data), and shared query helpers.
//!
//! The entity modules are GENERATED from the golden DDL
//! (`backend/sql/schema.sql`) by `scripts/gen-entities.py` — regenerate
//! on schema re-dump, never hand-edit.

pub mod auth;
pub mod bootstrap;
pub mod crypto;
pub mod entities;
pub mod paging;

use sea_orm::DatabaseConnection;

/// The golden DDL, embedded.
pub const SCHEMA_SQL: &str = include_str!("../../../sql/schema.sql");
/// The system-default seed (empty-DB bootstrap), embedded.
pub const SEED_SQL: &str = include_str!("../../../sql/seed.sql");
/// The demo seed data, embedded.
pub const DEMO_DATA_SQL: &str = include_str!("../../../sql/demo-data.sql");

/// DB failure → the Unknown envelope shape (500, empty reason).
pub fn db_err(e: sea_orm::DbErr) -> rushwind_http_binding::envelope::StatusError {
    rushwind_http_binding::envelope::internal_error(format!("db: {e}"))
}

/// A plain error string → the Unknown envelope shape.
pub fn db_err_internal(message: &str) -> rushwind_http_binding::envelope::StatusError {
    rushwind_http_binding::envelope::internal_error(message.to_string())
}

/// Now, as a timestamptz value (the schema's timestamp flavor).
pub fn now() -> chrono::DateTime<chrono::FixedOffset> {
    chrono::Utc::now().fixed_offset()
}

/// The tables exist check — the bootstrap gate (fresh databases get the
/// golden DDL + demo seed; existing ones are left alone).
pub async fn tables_present(db: &DatabaseConnection) -> Result<bool, sea_orm::DbErr> {
    use sea_orm::ConnectionTrait as _;
    let row = db
        .query_one_raw(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
               WHERE n.nspname='public' AND c.relname='sys_users' LIMIT 1"#
                .to_string(),
        ))
        .await?;
    Ok(row.is_some())
}
