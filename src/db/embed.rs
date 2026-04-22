//! Vector embedding module using fastembed (local ONNX inference).

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

/// Initialize the embedding model (downloads on first run, ~23MB).
pub fn init_model() -> Result<TextEmbedding> {
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true),
    )
    .context("Failed to initialize embedding model")?;
    Ok(model)
}

/// Generate embedding for a single text.
pub fn embed_text(model: &TextEmbedding, text: &str) -> Result<Vec<f32>> {
    let embeddings = model
        .embed(vec![text], None)
        .context("Failed to generate embedding")?;
    Ok(embeddings.into_iter().next().unwrap())
}

/// Prepare embedding input text from node fields.
/// Combines title + tags + content for richer semantic representation.
pub fn prepare_embed_text(title: &str, tags: &[String], content: &str) -> String {
    let tag_str = tags.join(" ");
    // Truncate content safely at char boundary (max ~500 chars)
    let content_preview: String = content.chars().take(500).collect();
    format!("{}\n{}\n{}", title, tag_str, content_preview)
}

/// Generate embeddings for multiple texts in a single batch (more efficient than calling embed_text in a loop).
pub fn embed_batch(model: &TextEmbedding, texts: &[String]) -> Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let embeddings = model
        .embed(refs, None)
        .context("Failed to generate batch embeddings")?;
    Ok(embeddings)
}
