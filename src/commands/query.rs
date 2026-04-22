use anyhow::{bail, Result};
use colored::Colorize;

use crate::db;

/// Normalize record ID: replace hyphens with underscores.
/// LLM may use wiki_path format (technical-account) instead of DB slug (technical_account).
fn normalize_id(id: &str) -> String {
    if let Some((node_type, slug)) = id.split_once(':') {
        format!("{}:{}", node_type, slug.replace('-', "_"))
    } else {
        id.to_string()
    }
}

/// Print header to stderr when json format, stdout otherwise.
macro_rules! header {
    ($fmt:expr, $format:expr $(, $arg:expr)*) => {
        if $format == "json" {
            eprintln!($fmt $(, $arg)*);
        } else {
            println!($fmt $(, $arg)*);
        }
    };
}

/// Run a predefined or custom query against the graph.
pub async fn run(
    data_dir: &str,
    preset: &str,
    args: Vec<String>,
    format: &str,
) -> Result<()> {
    let conn = db::connect(data_dir, "nimp", "wiki").await?;


    // Normalize record IDs: convert hyphens to underscores (LLM may use wiki_path format)
    let args: Vec<String> = args.into_iter().map(|a| {
        if a.contains(':') { normalize_id(&a) } else { a }
    }).collect();
    let query_str = match preset {
        "impact" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("module:technical_account");
            header!("{} {}", format, "🎯 Impact analysis for:".cyan().bold(), target.yellow());
            db::queries::impact_query(target)
        }
        "deps" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("entity:voucher");
            header!("{} {}", format, "🔗 Dependencies for:".cyan().bold(), target.yellow());
            db::queries::deps_query(target)
        }
        "rules" => {
            let severity = args.first().map(|s| s.as_str());
            header!(
                "{} {}",
                format,
                "📋 Business rules".cyan().bold(),
                severity.map_or("(all)".to_string(), |s| format!("(severity={})", s)).dimmed()
            );
            db::queries::rules_query(severity)
        }
        "coupling" => {
            header!("{}", format, "📊 Module coupling ranking".cyan().bold());
            db::queries::coupling_query()
        }
        "entity-usage" => {
            header!("{}", format, "📊 Entity usage across modules".cyan().bold());
            db::queries::entity_usage_query()
        }
        "list" => {
            let node_type = args.first().map(|s| s.as_str()).unwrap_or("all");
            let valid = ["module", "entity", "concept", "flow", "question", "overview", "all"];
            if !valid.contains(&node_type) {
                bail!("Invalid type '{}'. Available: {}", node_type, valid.join(", "));
            }
            header!("{} {}", format, "📋 Listing".cyan().bold(), node_type.yellow());
            db::queries::list_query(node_type)
        }
        "list-detail" => {
            let node_type = args.first().map(|s| s.as_str()).unwrap_or("all");
            let valid = ["module", "entity", "concept", "flow", "question", "overview", "all"];
            if !valid.contains(&node_type) {
                bail!("Invalid type '{}'. Available: {}", node_type, valid.join(", "));
            }
            header!("{} {} {}", format, "📋 Detail listing".cyan().bold(), node_type.yellow(),
                    "(title + summary + tags)".dimmed());
            db::queries::list_detail_query(node_type)
        }
        "context" => {
            let node_type = args.first().map(|s| s.as_str()).unwrap_or("");
            if node_type.is_empty() {
                bail!("Usage: hidow query context <type> (e.g. module, entity, concept, flow)");
            }
            let valid = ["module", "entity", "concept", "flow", "question", "overview"];
            if !valid.contains(&node_type) {
                bail!("Invalid type '{}'. Available: {}", node_type, valid.join(", "));
            }
            header!("{} {}", format, "📦 Full context for all".cyan().bold(), node_type.yellow());
            db::queries::context_query(node_type)
        }
        "search" => {
            let keyword = args.first().map(|s| s.as_str()).unwrap_or("");
            if keyword.is_empty() {
                bail!("Usage: hidow query search <keyword>");
            }
            header!("{} \"{}\"", format, "🔍 Search results for:".cyan().bold(), keyword.yellow());

            // Hybrid search: combine keyword + vector results via RRF
            {
                // Try to get vector results
                let model_result = db::embed::init_model();
                if let Ok(model) = model_result {
                    if let Ok(q_vec) = db::embed::embed_text(&model, keyword) {
                        let vec_json = format!("{:?}", q_vec);
                        let tables = ["module", "entity", "concept", "flow", "question", "overview"];

                        // 1. Keyword results
                        let kw_results = db::queries::run_query(&conn, &db::queries::keyword_search_for_hybrid(keyword)).await?;

                        // 2. Vector results
                        let mut vec_results: Vec<serde_json::Value> = Vec::new();
                        for table in tables {
                            let q = db::queries::semantic_search_query(table, &vec_json, 5);
                            let mut results = db::queries::run_query(&conn, &q).await?;
                            vec_results.append(&mut results);
                        }
                        vec_results.sort_by(|a, b| {
                            let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                        });

                        // 3. RRF merge (k=60)
                        let rrf_k = 60.0f64;
                        let mut rrf_scores: std::collections::HashMap<String, (f64, serde_json::Value)> = std::collections::HashMap::new();

                        for (rank, row) in kw_results.iter().enumerate() {
                            let key = format!("{}:{}",
                                row.get("node_type").and_then(|v| v.as_str()).unwrap_or(""),
                                row.get("node_id").and_then(|v| v.as_str()).unwrap_or(""),
                            );
                            let score = 1.0 / (rrf_k + rank as f64 + 1.0);
                            rrf_scores.entry(key).or_insert((0.0, row.clone())).0 += score;
                        }
                        for (rank, row) in vec_results.iter().enumerate() {
                            let key = format!("{}:{}",
                                row.get("node_type").and_then(|v| v.as_str()).unwrap_or(""),
                                row.get("node_id").and_then(|v| v.as_str()).unwrap_or(""),
                            );
                            let score = 1.0 / (rrf_k + rank as f64 + 1.0);
                            let entry = rrf_scores.entry(key).or_insert((0.0, row.clone()));
                            entry.0 += score;
                        }

                        let mut merged: Vec<(f64, serde_json::Value)> = rrf_scores.into_values().collect();
                        merged.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                        merged.truncate(10);

                        if format == "json" {
                            let json_results: Vec<serde_json::Value> = merged.iter().map(|(score, row)| {
                                let mut r = row.clone();
                                if let Some(obj) = r.as_object_mut() {
                                    obj.insert("rrf_score".to_string(), serde_json::json!(score));
                                }
                                r
                            }).collect();
                            println!("{}", serde_json::to_string_pretty(&json_results)?);
                        } else {
                            if !vec_results.is_empty() {
                                eprintln!("{}", "  (hybrid: keyword + vector)".dimmed());
                            }
                            for (i, (score, r)) in merged.iter().enumerate() {
                                let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                                let ntype = r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
                                println!("  {}. [{}] {:45} rrf: {:.6}", i + 1, ntype, title, score);
                            }
                            if merged.is_empty() {
                                println!("{}", "  (no results)".dimmed());
                            }
                        }
                        return Ok(());
                    }
                }
                // Fallback: keyword only
                let q = db::queries::search_query(keyword);
                let results = db::queries::run_query(&conn, &q).await?;
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                } else {
                    eprintln!("{}", "  (keyword only — no embeddings)".dimmed());
                    for (i, r) in results.iter().enumerate() {
                        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let ntype = r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("  {}. [{}] {}", i + 1, ntype, title);
                    }
                    if results.is_empty() {
                        println!("{}", "  (no results)".dimmed());
                    }
                }
                return Ok(());
            }

        }
        "info" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("");
            if target.is_empty() || !target.contains(':') {
                bail!("Usage: hidow query info <type:id> (e.g. module:accounting)");
            }

            if format == "json" {
                // JSON mode: return all info as structured JSON
                let info_results = db::queries::run_query(&conn, &db::queries::info_query(target)).await?;
                println!("{}", serde_json::to_string_pretty(&info_results)?);
                return Ok(());
            }

            println!("{} {}\n", "📄 Node info:".cyan().bold(), target.yellow());

            // Main info query
            let info_results = db::queries::run_query(&conn, &db::queries::info_query(target)).await?;

            // Print main info
            if let Some(row) = info_results.first() {
                if let Some(obj) = row.as_object() {
                    // Basic metadata
                    for key in &["title", "node_type", "status", "wiki_path"] {
                        if let Some(val) = obj.get(*key) {
                            let display = match val {
                                serde_json::Value::Null => "—".to_string(),
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            println!("  {}: {}", key.green(), display);
                        }
                    }
                    // Tags
                    if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
                        let tags_str: Vec<&str> = tags.iter().filter_map(|v| v.as_str()).collect();
                        println!("  {}: {}", "tags".green(), tags_str.join(", "));
                    }
                    // Sources
                    if let Some(sources) = obj.get("sources").and_then(|v| v.as_array()) {
                        let src_str: Vec<&str> = sources.iter().filter_map(|v| v.as_str()).collect();
                        println!("  {}: {}", "sources".green(), src_str.join(", "));
                    }

                    // Relationships summary
                    println!("\n  {}:", "Relationships".cyan().bold());
                    let rel_keys = [
                        ("out_depends_on", "depends_on →"),
                        ("in_depends_on", "depends_on ←"),
                        ("out_produces", "produces →"),
                        ("in_produces", "produced_by ←"),
                        ("out_consumes", "consumes →"),
                        ("in_consumes", "consumed_by ←"),
                        ("out_implements", "implements →"),
                        ("out_uses", "uses →"),
                        ("out_contains", "contains →"),
                        ("in_contains", "contained_by ←"),
                        ("out_part_of", "part_of →"),
                        ("in_part_of", "part_of ←"),
                        ("out_triggers", "triggers →"),
                        ("in_triggers", "triggered_by ←"),
                    ];
                    for (key, label) in &rel_keys {
                        if let Some(val) = obj.get(*key) {
                            let count = val.as_i64().unwrap_or(0);
                            if count > 0 {
                                println!("    {} {}", label.green(), count);
                            }
                        }
                    }
                }
            }

            // Business rules count (only for modules)
            if target.starts_with("module:") {
                let slug = target.strip_prefix("module:").unwrap_or("");
                let br_results = db::queries::run_query(&conn, &db::queries::info_rules_count(slug)).await?;
                if !br_results.is_empty() {
                    println!("\n  {}:", "Business Rules".cyan().bold());
                    let mut total = 0i64;
                    for row in &br_results {
                        let sev = row.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                        let cnt = row.get("cnt").and_then(|v| v.as_i64()).unwrap_or(0);
                        total += cnt;
                        println!("    {}: {}", sev.green(), cnt);
                    }
                    println!("    {}: {}", "total".green().bold(), total);
                }
            }

            return Ok(()); // Already printed custom format
        }
        "similar" => {
            {
                let target = args.first().map(|s| s.as_str()).unwrap_or("");
                if target.is_empty() || !target.contains(':') {
                    bail!("Usage: hidow query similar <type:id> (e.g. module:claim)");
                }
                let (table, _) = target.split_once(':').unwrap();
                header!("{} {}", format, "🔍 Similar to:".cyan().bold(), target.yellow());
                let q = db::queries::similar_query(table, target, 5);
                let results = db::queries::run_query(&conn, &q).await?;
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                } else {
                    for (i, r) in results.iter().enumerate() {
                        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ntype = r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("  {}. [{}] {:45} score: {:.4}", i + 1, ntype, title, score);
                    }
                    if results.is_empty() {
                        println!("{}", "  No embeddings found. Run: hidow ingest --embed".dimmed());
                    }
                }
                return Ok(());
            }
        }
        "semantic" => {
            {
                let question = args.first().map(|s| s.as_str()).unwrap_or("");
                if question.is_empty() {
                    bail!("Usage: hidow query semantic <question> (e.g. semantic \"tính phí\")");
                }
                header!("{} {}", format, "🧠 Semantic search:".cyan().bold(), question.yellow());
                let model = db::embed::init_model()?;
                let q_vec = db::embed::embed_text(&model, question)?;
                let vec_json = format!("{:?}", q_vec);
                let tables = ["module", "entity", "concept", "flow", "overview"];
                let mut all_results: Vec<serde_json::Value> = Vec::new();
                for table in tables {
                    let q = db::queries::semantic_search_query(table, &vec_json, 3);
                    let mut results = db::queries::run_query(&conn, &q).await?;
                    all_results.append(&mut results);
                }
                all_results.sort_by(|a, b| {
                    let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                all_results.truncate(10);
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&all_results)?);
                } else {
                    for (i, r) in all_results.iter().enumerate() {
                        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ntype = r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("  {}. [{}] {:45} score: {:.4}", i + 1, ntype, title, score);
                    }
                    if all_results.is_empty() {
                        println!("{}", "  No embeddings found. Run: hidow ingest".dimmed());
                    }
                }
                return Ok(());
            }
        }
        "ask" => {
            {
                let question = args.first().map(|s| s.as_str()).unwrap_or("");
                if question.is_empty() {
                    bail!("Usage: hidow query ask <question> [--top N] (e.g. ask \"XOL calculation\")");
                }
                // Parse --top flag from remaining args (default 3)
                let top_k: usize = args.iter()
                    .position(|a| a == "--top")
                    .and_then(|i| args.get(i + 1))
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3);

                header!("{} {} (top {})", format, "🧠 RAG context for:".cyan().bold(), question.yellow(), top_k);
                let model = db::embed::init_model()?;
                let q_vec = db::embed::embed_text(&model, question)?;
                let vec_json = format!("{:?}", q_vec);
                let tables = ["module", "entity", "concept", "flow", "question", "overview"];
                let mut all_results: Vec<serde_json::Value> = Vec::new();
                for table in tables {
                    let q = db::queries::ask_context_query(table, &vec_json, top_k);
                    let mut results = db::queries::run_query(&conn, &q).await?;
                    all_results.append(&mut results);
                }
                all_results.sort_by(|a, b| {
                    let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                all_results.truncate(top_k);

                if format == "json" {
                    // Structured output optimized for LLM system prompts
                    let context: Vec<serde_json::Value> = all_results.iter().map(|r| {
                        serde_json::json!({
                            "node": format!("{}:{}",
                                r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?"),
                                r.get("node_id").and_then(|v| v.as_str()).unwrap_or("?")),
                            "title": r.get("title").and_then(|v| v.as_str()).unwrap_or("?"),
                            "wiki_path": r.get("wiki_path").and_then(|v| v.as_str()).unwrap_or(""),
                            "score": r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            "content": r.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                        })
                    }).collect();
                    let output = serde_json::json!({
                        "question": question,
                        "context": context,
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    for (i, r) in all_results.iter().enumerate() {
                        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ntype = r.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
                        let wiki = r.get("wiki_path").and_then(|v| v.as_str()).unwrap_or("");
                        let content = r.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        let preview: String = content.chars().take(120).collect();
                        println!("\n  {}. [{}] {} (score: {:.4})", i + 1, ntype, title.green(), score);
                        println!("     wiki: {}", wiki.dimmed());
                        println!("     {}...", preview.dimmed());
                    }
                    if all_results.is_empty() {
                        println!("{}", "  No embeddings found. Run: hidow ingest".dimmed());
                    }
                }
                return Ok(());
            }
        }
        "content" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("");
            if target.is_empty() || !target.contains(':') {
                bail!("Usage: hidow query content <type:id> (e.g. module:accounting)");
            }

            let results = db::queries::run_query(&conn, &db::queries::content_query(target)).await?;

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                header!("{} {}", format, "📄 Content for:".cyan().bold(), target.yellow());
                if let Some(row) = results.first() {
                    let title = row.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                    let wiki_path = row.get("wiki_path").and_then(|v| v.as_str()).unwrap_or("?");
                    let content = row.get("content").and_then(|v| v.as_str()).unwrap_or("(no content)");
                    println!("  title: {}", title.green());
                    println!("  wiki_path: {}\n", wiki_path.dimmed());
                    println!("{}", content);
                } else {
                    println!("{}", "  (node not found)".dimmed());
                }
            }
            return Ok(());
        }
        "neighbors" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("");
            if target.is_empty() || !target.contains(':') {
                bail!("Usage: hidow query neighbors <type:id> (e.g. module:claim)");
            }

            let outgoing = db::queries::run_query(&conn, &db::queries::neighbors_outgoing_query(target)).await?;
            let incoming = db::queries::run_query(&conn, &db::queries::neighbors_incoming_query(target)).await?;
            let info = db::queries::run_query(&conn, &db::queries::info_query(target)).await?;
            let title = info.first().and_then(|r| r.get("title")).and_then(|v| v.as_str()).unwrap_or("?");

            if format == "json" {
                let output = serde_json::json!({
                    "node": target,
                    "title": title,
                    "outgoing": outgoing.first().unwrap_or(&serde_json::json!({})),
                    "incoming": incoming.first().unwrap_or(&serde_json::json!({})),
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{} {} ({})", "🔗 Neighbors of:".cyan().bold(), target.yellow(), title);
                println!("\n  {}:", "Outgoing →".cyan().bold());
                if let Some(row) = outgoing.first() {
                    if let Some(obj) = row.as_object() {
                        for (key, value) in obj {
                            if let Some(arr) = value.as_array() {
                                if !arr.is_empty() {
                                    let items: Vec<String> = arr.iter().filter_map(|v| {
                                        v.get("title").and_then(|t| t.as_str()).map(|s| s.to_string())
                                    }).collect();
                                    if !items.is_empty() {
                                        println!("    {}: {}", key.green(), items.join(", "));
                                    }
                                }
                            }
                        }
                    }
                }
                println!("\n  {}:", "Incoming ←".cyan().bold());
                if let Some(row) = incoming.first() {
                    if let Some(obj) = row.as_object() {
                        for (key, value) in obj {
                            if let Some(arr) = value.as_array() {
                                if !arr.is_empty() {
                                    let items: Vec<String> = arr.iter().filter_map(|v| {
                                        if let Some(t) = v.get("title").and_then(|t| t.as_str()) {
                                            Some(t.to_string())
                                        } else if let Some(r) = v.get("rule").and_then(|t| t.as_str()) {
                                            let sev = v.get("severity").and_then(|s| s.as_str()).unwrap_or("?");
                                            Some(format!("[{}] {}", sev, r))
                                        } else { None }
                                    }).collect();
                                    if !items.is_empty() {
                                        println!("    {}: {}", key.green(), items.join(", "));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
        "rules-for" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("");
            if target.is_empty() || !target.contains(':') {
                bail!("Usage: hidow query rules-for <type:id> (e.g. entity:voucher)");
            }
            header!("{} {}", format, "📋 Business rules for:".cyan().bold(), target.yellow());
            db::queries::rules_for_query(target)
        }
        "path" => {
            let from = args.first().map(|s| s.as_str()).unwrap_or("");
            let to = args.get(1).map(|s| s.as_str()).unwrap_or("");
            if from.is_empty() || to.is_empty() || !from.contains(':') || !to.contains(':') {
                bail!("Usage: hidow query path <from> <to> (e.g. module:claim module:accounting)");
            }

            if format == "json" {
                let direct = db::queries::run_query(&conn, &db::queries::path_direct_query(from, to)).await?;
                let shared = db::queries::run_query(&conn, &db::queries::path_shared_query(from, to)).await?;
                let output = serde_json::json!({
                    "direct_edges": direct,
                    "shared_entities": shared,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }

            println!("{} {} → {}\n", "🔗 Path:".cyan().bold(), from.yellow(), to.yellow());

            // 1. Direct edges
            let direct = db::queries::run_query(&conn, &db::queries::path_direct_query(from, to)).await?;
            println!("  {}:", "Direct edges".cyan().bold());
            if direct.is_empty() {
                println!("    (none)");
            } else {
                for edge in &direct {
                    let edge_type = edge.get("edge_type").and_then(|v| v.as_str()).unwrap_or("?");
                    let from_n = edge.get("from_node").and_then(|v| v.as_str()).unwrap_or("?");
                    let to_n = edge.get("to_node").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("    {} --{}-->{}", from_n, edge_type.green(), to_n);
                }
            }

            // 2. Shared entities
            let shared = db::queries::run_query(&conn, &db::queries::path_shared_query(from, to)).await?;
            println!("\n  {}:", "Shared entities (used by both)".cyan().bold());
            if shared.is_empty() {
                println!("    (none)");
            } else {
                for ent in &shared {
                    let title = ent.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                    let node_id = ent.get("node_id").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("    entity:{} — {}", node_id, title);
                }
            }

            return Ok(()); // Already printed custom format
        }
        "raw" => {
            let raw_query = args.first().map(|s| s.as_str()).unwrap_or("");
            if raw_query.is_empty() {
                bail!("Usage: hidow query raw \"<SurrealQL>\"");
            }
            header!("{} {}", format, "🔧 Raw query:".cyan().bold(), raw_query.dimmed());
            raw_query.to_string()
        }
        _ => {
            bail!(
                "Unknown preset '{}'. Available: list, list-detail, context, search, info, content, neighbors, impact, deps, rules, rules-for, coupling, entity-usage, path, similar, semantic, ask, raw",
                preset
            );
        }
    };

    let results = db::queries::run_query(&conn, &query_str).await?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        _ => {
            // table format (default)
            if results.is_empty() {
                println!("{}", "  (no results)".dimmed());
            } else {
                for (i, row) in results.iter().enumerate() {
                    println!("\n  {}:", format!("Result {}", i + 1).bold());
                    if let Some(obj) = row.as_object() {
                        for (key, value) in obj {
                            if key == "id" {
                                continue;
                            }
                            let display_val = match value {
                                serde_json::Value::Array(arr) if arr.is_empty() => "—".dimmed().to_string(),
                                serde_json::Value::Array(arr) => {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                }
                                serde_json::Value::Null => "—".dimmed().to_string(),
                                other => other.to_string(),
                            };
                            println!("    {}: {}", key.green(), display_val);
                        }
                    } else {
                        println!("  {}", row);
                    }
                }
            }
        }
    }

    Ok(())
}
