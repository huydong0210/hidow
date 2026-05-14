mod commands;
mod db;
mod parser;

use clap::{Parser, Subcommand};
use colored::Colorize;

const DEFAULT_WIKI_PATH: &str = "./wiki";

#[derive(Parser)]
#[command(
    name = "hidow",
    about = "CLI tool to manage knowledge graph instances in SurrealDB",
    version
)]
struct Cli {
    /// Instance name (each project = separate instance)
    #[arg(short = 'i', long, global = true)]
    instance: Option<String>,

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

        /// Filter by node type (e.g. module, entity, concept, flow, or any custom type)
        #[arg(long)]
        node_type: Option<String>,
    },

    /// Show graph status overview
    Status,

    /// Uninstall hidow — remove database, ORT runtime, model cache, and binary
    Uninstall {
        /// Skip confirmation prompt and proceed with removal
        #[arg(long)]
        confirm: bool,
    },

    /// Manage hidow instances
    Instance {
        /// Command: list
        preset: String,
    },
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

/// Auto-detect and set ORT_DYLIB_PATH for ONNX Runtime (used by fastembed).
/// Searches ~/.hidow/ort/lib/ for the shared library. No-op if already set.
fn auto_detect_ort_dylib() {
    if std::env::var("ORT_DYLIB_PATH").is_ok() {
        return; // User already set it, respect their choice
    }
    if let Some(home) = dirs::home_dir() {
        let ort_lib_dir = home.join(".hidow").join("ort").join("lib");
        if ort_lib_dir.exists() {
            // Find libonnxruntime.so.* (versioned) or libonnxruntime.so
            if let Ok(entries) = std::fs::read_dir(&ort_lib_dir) {
                let mut best: Option<std::path::PathBuf> = None;
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("libonnxruntime.so") && !name_str.contains("providers") {
                        // Prefer versioned .so.X.Y.Z over plain .so (which is usually a symlink)
                        if name_str.chars().filter(|c| *c == '.').count() > 1 {
                            best = Some(entry.path());
                        } else if best.is_none() {
                            best = Some(entry.path());
                        }
                    }
                }
                if let Some(lib_path) = best {
                    std::env::set_var("ORT_DYLIB_PATH", &lib_path);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    auto_detect_ort_dylib();

    let cli = Cli::parse();
    let data_dir = resolve_data_dir(&cli.data_dir);

    // Instance commands don't need an instance name
    if let Commands::Instance { ref preset } = cli.command {
        commands::instance::run(&data_dir, preset).await?;
        return Ok(());
    }
    if let Commands::Uninstall { confirm } = cli.command {
        commands::uninstall::run(confirm)?;
        return Ok(());
    }

    // Resolve instance (default + warning)
    let instance = resolve_instance(&cli.instance);

    match cli.command {
        Commands::Init => {
            commands::init::run(&data_dir, &instance).await?;
        }
        Commands::Ingest { full, dry_run, file } => {
            commands::ingest::run(
                &data_dir,
                &instance,
                &cli.wiki_path,
                full,
                dry_run,
                file.as_deref(),
            )
            .await?;
        }
        Commands::Lint { check } => {
            commands::lint::run(&data_dir, &instance, &cli.wiki_path, check.as_deref()).await?;
        }
        Commands::Query { preset, args, format } => {
            commands::query::run(&data_dir, &instance, &preset, args, &format).await?;
        }
        Commands::Export { format, node_type } => {
            commands::export::run(&data_dir, &instance, &format, node_type.as_deref()).await?;
        }
        Commands::Status => {
            commands::status::run(&data_dir, &instance).await?;
        }
        // Instance and Uninstall handled above
        Commands::Instance { .. } | Commands::Uninstall { .. } => unreachable!(),
    }

    Ok(())
}

/// Resolve instance name: use provided name or default with warning.
fn resolve_instance(instance: &Option<String>) -> String {
    match instance {
        Some(name) => name.clone(),
        None => {
            eprintln!(
                "{} No instance specified, using '{}'. Use {} to specify.",
                "⚠️ ".yellow(),
                "default".yellow().bold(),
                "-i <name>".cyan()
            );
            "default".to_string()
        }
    }
}
