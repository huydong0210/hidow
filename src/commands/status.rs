use anyhow::Result;
use colored::Colorize;

use crate::db;

/// Show graph status overview.
pub async fn run(data_dir: &str) -> Result<()> {
    println!("{}", "📊 NIMP Graph Status".cyan().bold());
    println!("  Data dir: {}", data_dir.green());

    let conn = db::connect(data_dir, "nimp", "wiki").await?;
    println!("  {}", "✅ Connected".green());
    println!("  Namespace: {} | DB: {}", "nimp".bold(), "wiki".bold());

    // Count nodes
    let tables = ["module", "entity", "concept", "flow", "question", "business_rule"];
    println!("\n  {}", "Nodes:".bold());
    let mut total_nodes = 0u64;
    for table in tables {
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
    let edge_tables = ["depends_on", "produces", "consumes", "contains", "part_of", "implements", "uses", "triggers", "affects"];
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
