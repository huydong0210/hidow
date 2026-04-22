use anyhow::Result;
use colored::Colorize;

use crate::db;

/// Initialize SurrealDB schema.
pub async fn run(db_url: &str) -> Result<()> {
    println!("{}", "🚀 Initializing SurrealDB schema...".cyan().bold());

    let conn = db::connect(db_url, "nimp", "wiki").await?;
    println!("  ✅ Connected to {}", db_url.green());

    db::schema::define_schema(&conn).await?;
    println!("  ✅ Node tables defined: module, entity, concept, flow, question, business_rule");
    println!("  ✅ Edge tables defined: depends_on, produces, consumes, contains, part_of, implements, uses, triggers, affects");

    println!("\n{}", "✅ Schema initialized successfully!".green().bold());
    Ok(())
}
