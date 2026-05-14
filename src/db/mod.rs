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

/// Fixed namespace for all hidow instances.
pub const NAMESPACE: &str = "hidow";

/// Connect to embedded SurrealDB (SurrealKV engine) for a specific instance.
/// Each instance = a separate database within the "hidow" namespace.
pub async fn connect(data_dir: &str, instance: &str) -> Result<DbConn> {
    // Ensure data directory exists
    let path = Path::new(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create data directory: {}", data_dir))?;
    }

    let db = Surreal::new::<SurrealKv>(data_dir).await
        .with_context(|| format!("Failed to open embedded DB at: {}", data_dir))?;

    db.use_ns(NAMESPACE).use_db(instance).await?;
    Ok(db)
}

/// Connect to SurrealDB at namespace level (for listing instances).
pub async fn connect_ns(data_dir: &str) -> Result<DbConn> {
    let path = Path::new(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create data directory: {}", data_dir))?;
    }

    let db = Surreal::new::<SurrealKv>(data_dir).await
        .with_context(|| format!("Failed to open embedded DB at: {}", data_dir))?;

    db.use_ns(NAMESPACE).await?;
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

/// Edge tables — these are fixed (TYPE RELATION).
pub const EDGE_TABLES: &[&str] = &[
    "depends_on", "produces", "consumes", "contains",
    "part_of", "implements", "uses", "triggers", "affects",
];

/// Get all node table names from DB (excludes edge tables and business_rule).
pub async fn node_tables(db: &DbConn) -> Result<Vec<String>> {
    let result: Vec<serde_json::Value> = db
        .query("INFO FOR DB;")
        .await?
        .take(0)?;

    let mut tables = Vec::new();
    if let Some(info) = result.first() {
        if let Some(tb) = info.get("tables").and_then(|v| v.as_object()) {
            for name in tb.keys() {
                // Skip edge tables and business_rule
                if !EDGE_TABLES.contains(&name.as_str()) && name != "business_rule" {
                    tables.push(name.clone());
                }
            }
        }
    }
    tables.sort();
    Ok(tables)
}

/// Build a FROM clause string for queries: "module, entity, concept, ..."
pub async fn node_tables_clause(db: &DbConn) -> Result<String> {
    let tables = node_tables(db).await?;
    Ok(tables.join(", "))
}
