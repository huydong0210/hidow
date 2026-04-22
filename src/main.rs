mod commands;
mod db;
mod parser;

use clap::{Parser, Subcommand};

const DEFAULT_WIKI_PATH: &str = "./wiki";

#[derive(Parser)]
#[command(
    name = "hidow",
    about = "CLI tool to manage NIMP wiki knowledge graph in SurrealDB",
    version
)]
struct Cli {
    /// Path to embedded database directory (default: ~/.hidow/data)
    #[arg(long, global = true)]
    data_dir: Option<String>,

    /// Path to wiki directory
    #[arg(long, global = true, default_value = DEFAULT_WIKI_PATH)]
    wiki_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize SurrealDB schema (run once)
    Init,

    /// Sync wiki pages into SurrealDB graph
    Ingest {
        /// Full reload (delete all, re-import everything)
        #[arg(long)]
        full: bool,

        /// Dry run — show what would change without writing
        #[arg(long)]
        dry_run: bool,

        /// Ingest a single file only
        #[arg(long)]
        file: Option<String>,
    },

    /// Validate graph integrity against wiki
    Lint {
        /// Run specific check only: sync, orphans, edges, rules
        #[arg(long)]
        check: Option<String>,
    },

    /// Query the graph (predefined presets or raw SurrealQL)
    Query {
        /// Preset name: list, search, info, content, neighbors, impact, deps, rules, rules-for, coupling, entity-usage, path, raw
        preset: String,

        /// Arguments for the preset (e.g. record ID, severity, raw query)
        args: Vec<String>,

        /// Output format: table (default) or json
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Export graph data to DOT, JSON, or CSV
    Export {
        /// Output format: dot, json, csv
        #[arg(long)]
        format: String,

        /// Filter by node type: module, entity, concept, flow
        #[arg(long)]
        node_type: Option<String>,
    },

    /// Show graph status overview
    Status,
}

/// Resolve the data directory path.
fn resolve_data_dir(data_dir: &Option<String>) -> String {
    if let Some(dir) = data_dir {
        return dir.clone();
    }
    // Default: ~/.hidow/data
    if let Some(home) = dirs::home_dir() {
        return home.join(".hidow").join("data").to_string_lossy().to_string();
    }
    // Fallback
    ".hidow/data".to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let data_dir = resolve_data_dir(&cli.data_dir);

    match cli.command {
        Commands::Init => {
            commands::init::run(&data_dir).await?;
        }
        Commands::Ingest { full, dry_run, file } => {
            commands::ingest::run(
                &data_dir,
                &cli.wiki_path,
                full,
                dry_run,
                file.as_deref(),
            )
            .await?;
        }
        Commands::Lint { check } => {
            commands::lint::run(&data_dir, &cli.wiki_path, check.as_deref()).await?;
        }
        Commands::Query { preset, args, format } => {
            commands::query::run(&data_dir, &preset, args, &format).await?;
        }
        Commands::Export { format, node_type } => {
            commands::export::run(&data_dir, &format, node_type.as_deref()).await?;
        }
        Commands::Status => {
            commands::status::run(&data_dir).await?;
        }
    }

    Ok(())
}
