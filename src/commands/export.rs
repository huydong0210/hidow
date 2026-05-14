use anyhow::{bail, Result};
use colored::Colorize;

use crate::db;

/// Export graph data to various formats.
pub async fn run(data_dir: &str, instance: &str, format: &str, node_type: Option<&str>) -> Result<()> {
    let conn = db::connect(data_dir, instance).await?;

    // Fetch all nodes (dynamic — includes custom types)
    let all_node_tables = db::node_tables(&conn).await.unwrap_or_default();
    let tables: Vec<&str> = match node_type {
        Some(t) => vec![t],
        None => all_node_tables.iter().map(|s| s.as_str()).collect(),
    };

    let mut all_nodes: Vec<serde_json::Value> = Vec::new();
    for table in &tables {
        let q = format!(
            "SELECT meta::id(id) AS id, meta::tb(id) AS node_type, title, status, tags, sources, wiki_path FROM {};",
            table
        );
        let results = db::queries::run_query(&conn, &q).await?;
        all_nodes.extend(results);
    }

    // Fetch all edges
    let edge_tables = db::EDGE_TABLES;
    let mut all_edges: Vec<serde_json::Value> = Vec::new();
    for table in edge_tables {
        let q = format!(
            "SELECT string::concat(meta::tb(in), ':', meta::id(in)) AS from_id, \
                    string::concat(meta::tb(out), ':', meta::id(out)) AS to_id, \
                    label FROM {};",
            table
        );
        let results = db::queries::run_query(&conn, &q).await?;
        for mut edge in results {
            if let Some(obj) = edge.as_object_mut() {
                obj.insert("edge_type".to_string(), serde_json::json!(table));
            }
            all_edges.push(edge);
        }
    }

    // Fetch business rules
    let br_q = "SELECT meta::id(id) AS br_id, rule, severity, module, ->affects->entity.title AS affected_entities FROM business_rule ORDER BY severity, br_id;";
    let all_brs = db::queries::run_query(&conn, br_q).await?;
    match format {
        "json" => export_json(&all_nodes, &all_edges, &all_brs)?,
        "dot" => export_dot(&all_nodes, &all_edges)?,
        "csv" => export_csv(&all_nodes, &all_edges)?,
        _ => bail!("Unknown format '{}'. Available: json, dot, csv", format),
    }

    Ok(())
}

fn export_json(nodes: &[serde_json::Value], edges: &[serde_json::Value], business_rules: &[serde_json::Value]) -> Result<()> {
    let output = serde_json::json!({
        "nodes": nodes,
        "business_rules": business_rules,
        "edges": edges,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    eprintln!(
        "\n{} {} nodes, {} edges",
        "Exported:".green(),
        nodes.len(),
        edges.len()
    );
    Ok(())
}

fn export_dot(nodes: &[serde_json::Value], edges: &[serde_json::Value]) -> Result<()> {
    println!("digraph hidow {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box, style=filled, fontname=\"Inter\"];");
    println!();

    // Color mapping for node types
    println!("  // Node type styles");
    println!("  node [fillcolor=\"#e3f2fd\"]; // default");

    for node in nodes {
        let node_type = node.get("node_type").and_then(|v| v.as_str()).unwrap_or("?");
        let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        let full_id = format!("{}:{}", node_type, node_id);
        let title = node.get("title").and_then(|v| v.as_str()).unwrap_or(&full_id);
        let safe_id = full_id.replace(':', "_").replace('-', "_");

        // Dynamic color palette for any node type
        let colors = ["#bbdefb", "#c8e6c9", "#fff9c4", "#f8bbd0", "#d1c4e9", "#b2dfdb", "#ffe0b2", "#f0f4c3"];
        let type_index = {
            let known = ["module", "entity", "concept", "flow", "question", "overview"];
            known.iter().position(|&t| t == node_type)
                .unwrap_or_else(|| {
                    // Hash-based index for unknown types
                    node_type.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize)) % colors.len()
                })
        };
        let color = colors.get(type_index % colors.len()).unwrap_or(&"#e0e0e0");

        println!("  {} [label=\"{}\", fillcolor=\"{}\"];", safe_id, title, color);
    }

    println!();
    for edge in edges {
        let from = edge.get("from_id").and_then(|v| v.as_str()).unwrap_or("?");
        let to = edge.get("to_id").and_then(|v| v.as_str()).unwrap_or("?");
        let edge_type = edge.get("edge_type").and_then(|v| v.as_str()).unwrap_or("?");

        let safe_from = from.replace(':', "_").replace('-', "_");
        let safe_to = to.replace(':', "_").replace('-', "_");

        println!("  {} -> {} [label=\"{}\"];", safe_from, safe_to, edge_type);
    }

    println!("}}");
    eprintln!(
        "\n{} {} nodes, {} edges → pipe to `dot -Tpng > graph.png`",
        "DOT exported:".green(),
        nodes.len(),
        edges.len()
    );
    Ok(())
}

fn export_csv(nodes: &[serde_json::Value], edges: &[serde_json::Value]) -> Result<()> {
    // Nodes CSV
    eprintln!("{}", "--- nodes.csv ---".bold());
    println!("id,type,title,status,tags");
    for node in nodes {
        let node_type = node.get("node_type").and_then(|v| v.as_str()).unwrap_or("");
        let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let full_id = format!("{}:{}", node_type, node_id);
        let title = node.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let status = node.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let tags = node.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(";"))
            .unwrap_or_default();
        let title_escaped = title.replace('"', "\"\"");
        println!("{},{},\"{}\",{},\"{}\"", full_id, node_type, title_escaped, status, tags);
    }

    eprintln!("\n{}", "--- edges.csv ---".bold());
    eprintln!("from,to,type,label");
    for edge in edges {
        let from = edge.get("from_id").and_then(|v| v.as_str()).unwrap_or("");
        let to = edge.get("to_id").and_then(|v| v.as_str()).unwrap_or("");
        let edge_type = edge.get("edge_type").and_then(|v| v.as_str()).unwrap_or("");
        let label = edge.get("label").and_then(|v| v.as_str()).unwrap_or("");
        let label_escaped = label.replace('"', "\"\"");
        eprintln!("{},{},{},\"{}\"", from, to, edge_type, label_escaped);
    }

    eprintln!(
        "\n{} {} nodes, {} edges",
        "CSV exported:".green(),
        nodes.len(),
        edges.len()
    );
    Ok(())
}
