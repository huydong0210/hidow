use anyhow::Result;
use colored::Colorize;

use crate::db;

/// Initialize SurrealDB schema.
pub async fn run(data_dir: &str) -> Result<()> {
    println!("{}", "🚀 Initializing SurrealDB schema...".cyan().bold());

    let conn = db::connect(data_dir, "nimp", "wiki").await?;
    println!("  ✅ Database at {}", data_dir.green());

    db::schema::define_schema(&conn).await?;
    println!("  ✅ Node tables defined: module, entity, concept, flow, question, business_rule");
    println!("  ✅ Edge tables defined: depends_on, produces, consumes, contains, part_of, implements, uses, triggers, affects");

    println!("\n{}", "✅ Schema initialized successfully!".green().bold());
    Ok(())
}
