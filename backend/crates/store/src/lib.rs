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
/// The menu seed (the admin-react route tree), embedded.
pub const MENUS_SEED_SQL: &str = include_str!("../../../sql/menus-seed.sql");

// The DB failure mappings and the entity-time helpers live in the
// framework bridge crate; re-exported so the repositories' and
// services' paths stay put.
pub use rushwind_storage_seaorm_support::time::now;
pub use rushwind_storage_seaorm_support::{db_err, internal_error_msg as db_err_internal};

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
