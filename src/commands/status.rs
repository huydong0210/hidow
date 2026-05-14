use anyhow::Result;
use colored::Colorize;

use crate::db;

/// Show graph status overview.
pub async fn run(data_dir: &str, instance: &str) -> Result<()> {
    println!("{}", "📊 Graph Status".cyan().bold());
    println!("  Instance: {}", instance.yellow().bold());
    println!("  Data dir: {}", data_dir.green());

    let conn = db::connect(data_dir, instance).await?;
    println!("  {}", "✅ Connected".green());
    println!("  Namespace: {} | DB: {}", db::NAMESPACE.bold(), instance.bold());

    // Count nodes (dynamic — includes custom types)
    let mut tables = db::node_tables(&conn).await.unwrap_or_default();
    tables.push("business_rule".to_string());
    println!("\n  {}", "Nodes:".bold());
    let mut total_nodes = 0u64;
    for table in &tables {
        let q = format!("SELECT count() FROM {} GROUP ALL;", table);
        let results = db::queries::run_query(&conn, &q).await?;
        let count = results.first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        total_nodes += count;
        println!("    {}: {}", format!("{:>15}", table).dimmed(), count);
    }
    println!("    {}: {}", format!("{:>15}", "TOTAL").bold(), total_nodes.to_string().bold());

    // Count edges
    let edge_tables = db::EDGE_TABLES;
    println!("\n  {}", "Edges:".bold());
    let mut total_edges = 0u64;
    for table in edge_tables {
        let q = format!("SELECT count() FROM {} GROUP ALL;", table);
        let results = db::queries::run_query(&conn, &q).await?;
        let count = results.first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if count > 0 {
            total_edges += count;
            println!("    {}: {}", format!("{:>15}", table).dimmed(), count);
        }
    }
    println!("    {}: {}", format!("{:>15}", "TOTAL").bold(), total_edges.to_string().bold());

    Ok(())
}
