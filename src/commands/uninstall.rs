use std::path::PathBuf;
use anyhow::Result;
use colored::Colorize;

/// Paths that hidow creates and manages.
fn get_hidow_paths() -> Vec<(PathBuf, &'static str)> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Database
        paths.push((home.join(".hidow").join("data"), "Database (SurrealKV)"));
        // ONNX Runtime
        paths.push((home.join(".hidow").join("ort"), "ONNX Runtime library"));
        // Parent .hidow dir (if empty after removing above)
        paths.push((home.join(".hidow"), "Config directory"));
        // Global binary
        paths.push((home.join(".cargo").join("bin").join("hidow"), "Installed binary"));
    }

    // Fastembed model cache (project-local)
    paths.push((PathBuf::from(".fastembed_cache"), "Fastembed model cache (local)"));

    paths
}

/// Run the uninstall command.
pub fn run(confirm: bool) -> Result<()> {
    println!("{}", "🗑  Hidow Uninstall".red().bold());
    println!();

    let paths = get_hidow_paths();

    // Show what will be removed
    println!("{}", "The following will be removed:".bold());
    let mut has_anything = false;
    for (path, label) in &paths {
        if path.exists() {
            let size = dir_size(path);
            println!(
                "  {} {} {}",
                "✕".red(),
                label.yellow(),
                format!("({})", format_size(size)).dimmed()
            );
            println!("    {}", path.display().to_string().dimmed());
            has_anything = true;
        }
    }

    if !has_anything {
        println!("\n{}", "Nothing to remove — hidow is already clean.".green());
        return Ok(());
    }

    // Confirm
    if !confirm {
        println!();
        println!(
            "{}",
            "Run with --confirm to proceed, or --dry-run to preview only.".yellow()
        );
        return Ok(());
    }

    println!();

    // Remove paths (in order: files first, then parent dirs)
    for (path, label) in &paths {
        if path.exists() {
            if path.is_dir() {
                // For .hidow parent: only remove if empty
                if *label == "Config directory" {
                    if is_dir_empty(path) {
                        std::fs::remove_dir(path)?;
                        println!("  {} {} {}", "✓".green(), "Removed".green(), label);
                    } else {
                        println!("  {} {} (not empty, skipped)", "⏭".dimmed(), label.dimmed());
                    }
                } else {
                    std::fs::remove_dir_all(path)?;
                    println!("  {} {} {}", "✓".green(), "Removed".green(), label);
                }
            } else {
                std::fs::remove_file(path)?;
                println!("  {} {} {}", "✓".green(), "Removed".green(), label);
            }
        }
    }

    println!("\n{}", "✅ Hidow has been uninstalled.".green().bold());
    println!(
        "{}",
        "   To reinstall: ./scripts/setup.sh".dimmed()
    );

    Ok(())
}

/// Calculate total size of a path (file or directory).
fn dir_size(path: &PathBuf) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Format bytes as human-readable size.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.0}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Check if a directory is empty.
fn is_dir_empty(path: &PathBuf) -> bool {
    path.read_dir()
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true)
}
