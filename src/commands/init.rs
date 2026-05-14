use anyhow::Result;
use colored::Colorize;

use crate::db;

/// Initialize SurrealDB schema.
pub async fn run(data_dir: &str, instance: &str) -> Result<()> {
    println!("{}", "🚀 Initializing SurrealDB schema...".cyan().bold());
    println!("  Instance: {}", instance.yellow().bold());

    let conn = db::connect(data_dir, instance).await?;
    println!("  ✅ Database at {}", data_dir.green());

    db::schema::define_schema(&conn).await?;
    let node_tables = db::node_tables(&conn).await.unwrap_or_default();
    println!("  ✅ Node tables defined: {}", node_tables.join(", "));
    println!("  ✅ Edge tables defined: {}", db::EDGE_TABLES.join(", "));
    println!("  ✅ Business rule table defined");

    println!("\n{}", "✅ Schema initialized successfully!".green().bold());
    Ok(())
}
