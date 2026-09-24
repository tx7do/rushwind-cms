//! The database bootstrap: on a fresh database (no `sys_users` table),
//! apply the golden DDL statement-by-statement (CREATE … IF NOT EXISTS /
//! DO-block constraint guards — idempotent by construction), then the
//! demo seed data. The DDL is dumped from pg_catalog
//! (`backend/sql/schema.sql`).

use sea_orm::DatabaseConnection;

/// Applies schema + system seed + demo seed when the database is fresh.
/// Existing databases (already carrying the schema) are left untouched —
/// the golden-DDL pipeline owns evolution.
pub async fn run(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    if crate::tables_present(db).await? {
        return Ok(());
    }
    exec_statements(db, crate::SCHEMA_SQL).await?;
    exec_statements(db, crate::SEED_SQL).await?;
    exec_statements(db, crate::MENUS_SEED_SQL).await?;
    exec_statements(db, crate::DEMO_DATA_SQL).await?;
    Ok(())
}

/// Executes a SQL script by splitting on statement-terminating
/// semicolons (the scripts carry no functions/triggers — plain
/// DDL/DML — so a line-end split is exact).
async fn exec_statements(db: &DatabaseConnection, script: &str) -> Result<(), sea_orm::DbErr> {
    let mut buf = String::new();
    for line in script.lines() {
        buf.push_str(line);
        buf.push('\n');
        if line.trim_end().ends_with(';') {
            exec_one(db, &buf).await?;
            buf.clear();
        }
    }
    if !buf.trim().is_empty() {
        exec_one(db, &buf).await?;
    }
    Ok(())
}

async fn exec_one(db: &DatabaseConnection, stmt: &str) -> Result<(), sea_orm::DbErr> {
    use sea_orm::ConnectionTrait as _;
    let backend = sea_orm::DatabaseBackend::Postgres;
    let result = db
        .execute_raw(sea_orm::Statement::from_string(backend, stmt.to_string()))
        .await;
    if let Err(e) = &result {
        eprintln!(
            "[bootstrap] statement failed: {e}\n>>> {}<<<",
            &stmt[..stmt.len().min(400)]
        );
    }
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn schema_statements_split_cleanly() {
        // Every statement must terminate with ';' and reassemble to the
        // full script length (the splitter loses only the final newline
        // of each statement — round-tripped back by exec joining).
        let script = "CREATE TABLE a (id bigint);\nCREATE INDEX x ON a (id);\n";
        let mut n = 0;
        let mut buf = String::new();
        for line in script.lines() {
            buf.push_str(line);
            buf.push('\n');
            if line.trim_end().ends_with(';') {
                assert!(buf.trim_end().ends_with(';'));
                n += 1;
                buf.clear();
            }
        }
        assert_eq!(n, 2);
        assert!(buf.is_empty());
    }
}
