pub mod models;

use std::path::Path;
use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use models::{WikiFrontmatter, WikiPage};

/// List of files to skip during parsing (meta pages without graph-ready data).
const SKIP_FILES: &[&str] = &["index.md", "log.md", "overview.md"];

/// Parse all wiki markdown files from the given directory.
pub fn parse_wiki_dir(wiki_path: &Path) -> Result<Vec<WikiPage>> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(wiki_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext == "md")
        })
    {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();

        // Skip meta files
        if SKIP_FILES.iter().any(|s| *s == filename.as_ref()) {
            continue;
        }

        match parse_wiki_file(path) {
            Ok(page) => pages.push(page),
            Err(e) => {
                eprintln!("⚠️  Failed to parse {}: {}", path.display(), e);
            }
        }
    }

    pages.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(pages)
}

/// Parse a single wiki markdown file.
pub fn parse_wiki_file(file_path: &Path) -> Result<WikiPage> {
    let raw = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    // Compute SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    // Extract frontmatter between --- delimiters
    let re = Regex::new(r"(?s)^---\n(.*?)\n---")?;
    let caps = re
        .captures(&raw)
        .with_context(|| format!("No frontmatter found in {}", file_path.display()))?;

    let yaml_str = &caps[1];
    let frontmatter: WikiFrontmatter = serde_yaml::from_str(yaml_str)
        .with_context(|| format!("Failed to parse YAML in {}", file_path.display()))?;

    // Extract body content (after second ---)
    let body_start = caps.get(0).unwrap().end();
    let content = raw[body_start..].trim().to_string();

    // Derive path and slug
    // file_path: /abs/path/wiki/modules/accounting.md
    // We want: wiki/modules/accounting (relative, no .md)
    let rel_path = derive_wiki_path(file_path);
    let slug = derive_slug(file_path);

    Ok(WikiPage {
        path: rel_path,
        slug,
        frontmatter,
        content,
        content_hash,
    })
}

/// Derive the wiki-relative path (e.g. "wiki/modules/accounting") from a file path.
fn derive_wiki_path(file_path: &Path) -> String {
    let path_str = file_path.to_string_lossy();
    // Find "wiki/" in the path and take everything after it, removing .md
    if let Some(idx) = path_str.find("wiki/") {
        let rel = &path_str[idx..];
        rel.trim_end_matches(".md").to_string()
    } else {
        file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

/// Derive a slug suitable for SurrealDB record ID (e.g. "accounting").
fn derive_slug(file_path: &Path) -> String {
    file_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_derive_slug() {
        let p = PathBuf::from("wiki/modules/technical-account.md");
        assert_eq!(derive_slug(&p), "technical_account");
    }

    #[test]
    fn test_derive_wiki_path() {
        let p = PathBuf::from("/home/user/docs/wiki/modules/accounting.md");
        assert_eq!(derive_wiki_path(&p), "wiki/modules/accounting");
    }
}
