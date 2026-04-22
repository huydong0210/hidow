use std::path::Path;
use anyhow::Result;
use colored::Colorize;

use crate::db;
use crate::parser;

/// Ingest wiki pages into SurrealDB.
pub async fn run(data_dir: &str, wiki_path: &str, full: bool, dry_run: bool, file: Option<&str>) -> Result<()> {
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
    let conn = db::connect(data_dir, "nimp", "wiki").await?;
    println!("  ✅ Database at {}", data_dir.green());

    // Auto-init schema if database is empty
    if !db::is_initialized(&conn).await? {
        println!("{}", "🚀 First run detected — initializing schema...".cyan());
        db::schema::define_schema(&conn).await?;
        println!("  ✅ Schema initialized automatically");
    }

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

    // Always generate embeddings
    generate_embeddings(&conn).await?;

    Ok(())
}

async fn generate_embeddings(conn: &db::DbConn) -> anyhow::Result<()> {
    eprintln!("\n{}", "🧠 Generating embeddings...".cyan().bold());
    let model = db::embed::init_model()?;
    let tables = ["module", "entity", "concept", "flow", "question", "overview"];
    let mut embed_count = 0;
    for table in tables {
        let q = format!("SELECT meta::id(id) AS node_id, title, tags, content FROM {};", table);
        let rows = db::queries::run_query(conn, &q).await?;
        for row in &rows {
            let node_id = row.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
            let title = row.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let content = row.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let tags: Vec<String> = row.get("tags")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let text = db::embed::prepare_embed_text(title, &tags, content);
            let vector = db::embed::embed_text(&model, &text)?;
            let vec_str = format!("{:?}", vector);
            let update_q = format!("UPDATE {}:{} SET embedding = {};", table, node_id, vec_str);
            conn.query(&update_q).await?;
            embed_count += 1;
        }
    }
    eprintln!("  {} {} embeddings generated", "✅".green(), embed_count.to_string().bold());
    Ok(())
}
