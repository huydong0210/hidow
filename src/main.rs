mod commands;
mod db;
mod parser;

use clap::{Parser, Subcommand};

const DEFAULT_DB_URL: &str = "127.0.0.1:8123";
const DEFAULT_WIKI_PATH: &str = "./wiki";

#[derive(Parser)]
#[command(
    name = "hidow",
    about = "CLI tool to manage NIMP wiki knowledge graph in SurrealDB",
    version
)]
struct Cli {
    /// SurrealDB URL (without ws:// prefix)
    #[arg(long, global = true, default_value = DEFAULT_DB_URL)]
    db_url: String,

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
        /// Preset name: impact, deps, rules, coupling, entity-usage, raw
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init::run(&cli.db_url).await?;
        }
        Commands::Ingest { full, dry_run, file } => {
            commands::ingest::run(
                &cli.db_url,
                &cli.wiki_path,
                full,
                dry_run,
                file.as_deref(),
            )
            .await?;
        }
        Commands::Lint { check } => {
            commands::lint::run(&cli.db_url, &cli.wiki_path, check.as_deref()).await?;
        }
        Commands::Query { preset, args, format } => {
            commands::query::run(&cli.db_url, &preset, args, &format).await?;
        }
        Commands::Export { format, node_type } => {
            commands::export::run(&cli.db_url, &format, node_type.as_deref()).await?;
        }
        Commands::Status => {
            commands::status::run(&cli.db_url).await?;
        }
    }

    Ok(())
}
