pub mod schema;
pub mod loader;
pub mod queries;
pub mod embed;

use std::path::Path;
use anyhow::{Context, Result};
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

/// Type alias for the database connection.
/// All modules use this type instead of concrete engine types.
pub type DbConn = Surreal<Db>;

/// Connect to embedded SurrealDB (SurrealKV engine).
/// Creates the data directory if it doesn't exist.
pub async fn connect(data_dir: &str, ns: &str, db_name: &str) -> Result<DbConn> {
    // Ensure data directory exists
    let path = Path::new(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create data directory: {}", data_dir))?;
    }

    let db = Surreal::new::<SurrealKv>(data_dir).await
        .with_context(|| format!("Failed to open embedded DB at: {}", data_dir))?;

    db.use_ns(ns).use_db(db_name).await?;
    Ok(db)
}

/// Check if the database has been initialized (schema exists).
pub async fn is_initialized(db: &DbConn) -> Result<bool> {
    let result: Vec<serde_json::Value> = db
        .query("INFO FOR DB;")
        .await?
        .take(0)?;
    // If we get info back and tables exist, schema is initialized
    if let Some(info) = result.first() {
        if let Some(tables) = info.get("tables") {
            if let Some(obj) = tables.as_object() {
                return Ok(!obj.is_empty());
            }
        }
    }
    Ok(false)
}
