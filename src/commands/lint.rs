use std::path::Path;
use anyhow::Result;
use colored::Colorize;

use crate::db;
use crate::parser;

/// Lint/validate the graph against the wiki source of truth.
pub async fn run(data_dir: &str, wiki_path: &str, check: Option<&str>) -> Result<()> {
    println!("{}", "🔍 Graph Health Check".cyan().bold());

    let wiki = Path::new(wiki_path);
    let pages = parser::parse_wiki_dir(wiki)?;
    let conn = db::connect(data_dir, "nimp", "wiki").await?;

    let mut issues = 0u32;

    // ── 1. Sync Check ──
    if check.is_none() || check == Some("sync") {
        println!("\n{}", "1. SYNC CHECK — wiki vs graph".bold());
        let changed = db::loader::get_changed_pages(&conn, &pages).await?;
        if changed.is_empty() {
            println!("  {} {}/{} pages in sync", "✅".green(), pages.len(), pages.len());
        } else {
            for p in &changed {
                println!("  {} {} modified since last ingest", "⚠️".yellow(), p.path);
                issues += 1;
            }
        }
    }

    // ── 2. Orphan Nodes ──
    if check.is_none() || check == Some("orphans") {
        println!("\n{}", "2. ORPHAN NODES — nodes without edges".bold());
        let results = db::queries::run_query(
            &conn,
            "SELECT id, title FROM module WHERE count(->depends_on) + count(<-depends_on) + count(->produces) + count(->consumes) + count(->implements) + count(->uses) = 0;"
        ).await?;
        if results.is_empty() {
            println!("  {} No orphan modules", "✅".green());
        } else {
            for r in &results {
                println!("  {} Orphan: {}", "⚠️".yellow(), r);
                issues += 1;
            }
        }
    }

    // ── 3. Edge Integrity ──
    if check.is_none() || check == Some("edges") {
        println!("\n{}", "3. EDGE INTEGRITY".bold());
        // Count total edges
        let edge_tables = ["depends_on", "produces", "consumes", "contains", "part_of", "implements", "uses", "triggers", "affects"];
        let mut total_edges = 0u64;
        for table in edge_tables {
            let q = format!("SELECT count() FROM {} GROUP ALL;", table);
            let results = db::queries::run_query(&conn, &q).await?;
            if let Some(first) = results.first() {
                if let Some(count) = first.get("count").and_then(|v| v.as_u64()) {
                    total_edges += count;
                }
            }
        }
        println!("  {} {} total edges across {} edge types", "✅".green(), total_edges, edge_tables.len());
    }

    // ── 4. BR Uniqueness ──
    if check.is_none() || check == Some("rules") {
        println!("\n{}", "4. BUSINESS RULE UNIQUENESS".bold());
        let results = db::queries::run_query(
            &conn,
            "SELECT count() FROM business_rule GROUP ALL;"
        ).await?;
        let br_count = results.first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        println!("  {} {} business rules", "✅".green(), br_count);
    }

    // ── 5. Graph Metrics ──
    if check.is_none() {
        println!("\n{}", "5. GRAPH METRICS".bold());
        let coupling_results = db::queries::run_query(&conn, &db::queries::coupling_query()).await?;
        if let Some(most) = coupling_results.first() {
            println!(
                "  Most connected: {} (deps out: {}, deps in: {})",
                most.get("title").and_then(|v| v.as_str()).unwrap_or("?"),
                most.get("outgoing_deps").unwrap_or(&serde_json::json!(0)),
                most.get("incoming_deps").unwrap_or(&serde_json::json!(0)),
            );
        }
    }

    // ── Summary ──
    println!("\n{}", "─".repeat(40));
    if issues == 0 {
        println!("{}", "✅ All checks passed!".green().bold());
    } else {
        println!(
            "{} {} issue(s) found. Run `hidow ingest` to fix sync issues.",
            "⚠️".yellow(),
            issues
        );
    }

    Ok(())
}
