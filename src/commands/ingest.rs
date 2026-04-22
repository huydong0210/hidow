use std::path::Path;
use anyhow::Result;
use colored::Colorize;

use crate::db;
use crate::parser;

/// Ingest wiki pages into SurrealDB.
pub async fn run(db_url: &str, wiki_path: &str, full: bool, dry_run: bool, file: Option<&str>) -> Result<()> {
    let wiki = Path::new(wiki_path);

    // Parse
    let pages = if let Some(single_file) = file {
        println!("{}", format!("📄 Parsing single file: {}", single_file).cyan());
        let page = parser::parse_wiki_file(Path::new(single_file))?;
        vec![page]
    } else {
        println!("{}", format!("📥 Scanning {}...", wiki_path).cyan());
        parser::parse_wiki_dir(wiki)?
    };

    println!(
        "  Found {} pages ({} modules, {} entities, {} concepts, {} flows)",
        pages.len().to_string().bold(),
        pages.iter().filter(|p| p.frontmatter.page_type == "module").count(),
        pages.iter().filter(|p| p.frontmatter.page_type == "entity").count(),
        pages.iter().filter(|p| p.frontmatter.page_type == "concept").count(),
        pages.iter().filter(|p| p.frontmatter.page_type == "flow").count(),
    );

    if dry_run {
        println!("\n{}", "🔍 Dry run — no changes will be written.".yellow());
        for page in &pages {
            println!("  {} {}:{} — {}", "→".dimmed(), page.frontmatter.page_type, page.slug, page.frontmatter.title);
        }
        let total_rels: usize = pages.iter().map(|p| p.frontmatter.relationships.len()).sum();
        let total_brs: usize = pages.iter().map(|p| p.frontmatter.business_rules.len()).sum();
        println!("\n  Would create: {} nodes, {} edges, {} business rules", pages.len(), total_rels, total_brs);
        return Ok(());
    }

    // Connect
    let conn = db::connect(db_url, "nimp", "wiki").await?;
    println!("  ✅ Connected to {}", db_url.green());

    // Determine which pages need updating
    let pages_to_load = if full || file.is_some() {
        println!("{}", "🔄 Full reload mode".yellow());
        pages.iter().collect::<Vec<_>>()
    } else {
        // Smart diff
        let changed = db::loader::get_changed_pages(&conn, &pages).await?;
        let unchanged = pages.len() - changed.len();
        println!(
            "  {} changed, {} unchanged",
            changed.len().to_string().yellow(),
            unchanged.to_string().dimmed()
        );
        if changed.is_empty() {
            println!("\n{}", "✅ Wiki and graph are in sync. Nothing to do.".green());
            return Ok(());
        }
        changed
    };

    // Load
    let (nodes, edges, brs) =
        db::loader::load_pages(&conn, &pages_to_load.iter().map(|p| (*p).clone()).collect::<Vec<_>>(), full).await?;

    println!(
        "\n{} {} nodes, {} edges, {} business rules",
        "✅ Sync complete:".green().bold(),
        nodes.to_string().bold(),
        edges.to_string().bold(),
        brs.to_string().bold(),
    );

    Ok(())
}
