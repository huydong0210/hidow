use anyhow::{bail, Result};
use colored::Colorize;

use crate::db;

/// Manage hidow instances.
pub async fn run(data_dir: &str, preset: &str) -> Result<()> {
    match preset {
        "list" => list_instances(data_dir).await,
        _ => bail!("Unknown instance command '{}'. Available: list", preset),
    }
}

/// List all instances (databases) in the hidow namespace.
async fn list_instances(data_dir: &str) -> Result<()> {
    println!("{}", "📦 Hidow Instances".cyan().bold());
    println!("  Data dir: {}", data_dir.green());

    let conn = db::connect_ns(data_dir).await?;

    // Query namespace info to get all databases
    let result: Vec<serde_json::Value> = conn
        .query("INFO FOR NS;")
        .await?
        .take(0)?;

    let databases: Vec<String> = if let Some(info) = result.first() {
        if let Some(dbs) = info.get("databases").and_then(|v| v.as_object()) {
            dbs.keys().cloned().collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if databases.is_empty() {
        println!("\n  {}", "No instances found. Run `hidow -i <name> init` to create one.".dimmed());
        return Ok(());
    }

    println!("\n  {:<20} {:>8} {:>8}", "Instance".bold(), "Nodes".bold(), "Edges".bold());
    println!("  {}", "─".repeat(40));

    let mut sorted_dbs = databases;
    sorted_dbs.sort();

    for db_name in &sorted_dbs {
        let instance_conn = db::connect(data_dir, db_name).await?;

        // Count nodes
        let node_tables = db::node_tables(&instance_conn).await.unwrap_or_default();
        let mut total_nodes = 0u64;
        for table in &node_tables {
            let q = format!("SELECT count() FROM {} GROUP ALL;", table);
            let results = db::queries::run_query(&instance_conn, &q).await.unwrap_or_default();
            if let Some(first) = results.first() {
                total_nodes += first.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }
        // Add business_rule count
        let br_results = db::queries::run_query(&instance_conn, "SELECT count() FROM business_rule GROUP ALL;").await.unwrap_or_default();
        if let Some(first) = br_results.first() {
            total_nodes += first.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        }

        // Count edges
        let mut total_edges = 0u64;
        for table in db::EDGE_TABLES {
            let q = format!("SELECT count() FROM {} GROUP ALL;", table);
            let results = db::queries::run_query(&instance_conn, &q).await.unwrap_or_default();
            if let Some(first) = results.first() {
                total_edges += first.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }

        println!("  {:<20} {:>8} {:>8}", db_name.yellow(), total_nodes, total_edges);
    }

    println!("\n  {} instance(s)", sorted_dbs.len().to_string().bold());
    Ok(())
}
