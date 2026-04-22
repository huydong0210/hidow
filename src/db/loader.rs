use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::json;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::parser::models::WikiPage;

/// Resolve a wiki path (e.g. "wiki/modules/accounting") to a SurrealDB record ID (e.g. "module:accounting").
fn resolve_record_id(wiki_path: &str) -> Option<(String, String)> {
    // wiki_path format: "wiki/{type_plural}/{slug}"
    let parts: Vec<&str> = wiki_path.split('/').collect();
    if parts.len() < 3 {
        return None;
    }

    let type_plural = parts[1];
    let slug = parts[2..].join("_").replace('-', "_");

    let table = match type_plural {
        "modules" => "module",
        "entities" => "entity",
        "concepts" => "concept",
        "flows" => "flow",
        "questions" => "question",
        _ => return None,
    };

    Some((table.to_string(), slug))
}

/// Load all parsed wiki pages into SurrealDB.
/// Returns (nodes_created, edges_created, brs_created).
pub async fn load_pages(
    db: &Surreal<Client>,
    pages: &[WikiPage],
    clean: bool,
) -> Result<(usize, usize, usize)> {
    let mut nodes_created = 0usize;
    let mut edges_created = 0usize;
    let mut brs_created = 0usize;

    if clean {
        println!("{}", "🗑️  Cleaning existing data...".yellow());
        db.query(
            "
            DELETE module; DELETE entity; DELETE concept; DELETE flow; DELETE question;
            DELETE business_rule;
            DELETE depends_on; DELETE produces; DELETE consumes; DELETE contains;
            DELETE part_of; DELETE implements; DELETE uses; DELETE triggers; DELETE affects;
            ",
        )
        .await?;
    }

    // ── Phase 1: Create nodes ──
    println!("{}", "📦 Creating nodes...".cyan());
    for page in pages {
        let table = &page.frontmatter.page_type;
        let record_id = format!("{}:{}", table, page.slug);

        let query = format!(
            "CREATE {} SET \
                title = $title, \
                status = $status, \
                tags = $tags, \
                sources = $sources, \
                content = $content, \
                content_hash = $content_hash, \
                wiki_path = $wiki_path, \
                attributes = $attributes, \
                data_flow = $data_flow;",
            record_id
        );

        db.query(&query)
            .bind(("title", page.frontmatter.title.clone()))
            .bind(("status", page.frontmatter.status.clone()))
            .bind(("tags", page.frontmatter.tags.clone()))
            .bind(("sources", page.frontmatter.sources.clone()))
            .bind(("content", page.content.clone()))
            .bind(("content_hash", page.content_hash.clone()))
            .bind(("wiki_path", page.path.clone()))
            .bind(("attributes", json!(page.frontmatter.attributes)))
            .bind(("data_flow", json!(page.frontmatter.data_flow)))
            .await
            .with_context(|| format!("Failed to create node {}", record_id))?;

        nodes_created += 1;
        println!("  + {}", record_id.green());
    }

    // ── Phase 2: Create business_rule nodes ──
    println!("{}", "📋 Creating business rules...".cyan());
    for page in pages {
        let module_slug = &page.slug;
        for br in &page.frontmatter.business_rules {
            let br_id = br.id.replace('-', "_");
            let record_id = format!("business_rule:{}", br_id);

            db.query(&format!(
                "CREATE {} SET \
                    rule = $rule, \
                    severity = $severity, \
                    module = $module;",
                record_id
            ))
            .bind(("rule", br.rule.clone()))
            .bind(("severity", br.severity.clone()))
            .bind(("module", module_slug.clone()))
            .await
            .with_context(|| format!("Failed to create BR {}", br_id))?;

            brs_created += 1;

            // Create affects edges (BR → entity)
            for affect_path in &br.affects {
                if let Some((target_table, target_slug)) = resolve_record_id(affect_path) {
                    let relate_query = format!(
                        "RELATE {}->affects->{}:{} SET label = $label;",
                        record_id, target_table, target_slug
                    );
                    db.query(&relate_query)
                        .bind(("label", format!("Rule {} affects", br.id)))
                        .await?;
                    edges_created += 1;
                }
            }
        }
    }

    // ── Phase 3: Create relationship edges ──
    println!("{}", "🔗 Creating edges...".cyan());
    for page in pages {
        let source_table = &page.frontmatter.page_type;
        let source_id = format!("{}:{}", source_table, page.slug);

        for rel in &page.frontmatter.relationships {
            if let Some((target_table, target_slug)) = resolve_record_id(&rel.target) {
                let target_id = format!("{}:{}", target_table, target_slug);
                let edge_table = &rel.rel_type;

                let relate_query = format!(
                    "RELATE {}->{}->{}  SET label = $label;",
                    source_id, edge_table, target_id
                );

                match db
                    .query(&relate_query)
                    .bind(("label", rel.label.clone()))
                    .await
                {
                    Ok(_) => {
                        edges_created += 1;
                    }
                    Err(e) => {
                        eprintln!(
                            "  {} edge {} -> {} -> {}: {}",
                            "⚠️".yellow(),
                            source_id,
                            edge_table,
                            target_id,
                            e
                        );
                    }
                }
            } else {
                eprintln!(
                    "  {} Cannot resolve target: {}",
                    "⚠️".yellow(),
                    rel.target
                );
            }
        }
    }

    Ok((nodes_created, edges_created, brs_created))
}

/// Check which pages have changed since last ingest by comparing content hashes.
pub async fn get_changed_pages<'a>(
    db: &Surreal<Client>,
    pages: &'a [WikiPage],
) -> Result<Vec<&'a WikiPage>> {
    let mut changed = Vec::new();

    for page in pages {
        let table = &page.frontmatter.page_type;
        let record_id = format!("{}:{}", table, page.slug);

        let result: Option<serde_json::Value> = db
            .query(format!("SELECT content_hash FROM {}", record_id))
            .await?
            .take(0)?;

        match result {
            Some(val) => {
                let stored_hash = val
                    .get("content_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if stored_hash != page.content_hash {
                    changed.push(page);
                }
            }
            None => {
                // Node doesn't exist yet
                changed.push(page);
            }
        }
    }

    Ok(changed)
}
