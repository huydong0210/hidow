//! Vector embedding module using fastembed (local ONNX inference).
//! Only compiled when the `vector` feature is enabled.

#[cfg(feature = "vector")]
use anyhow::{Context, Result};
#[cfg(feature = "vector")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

#[cfg(feature = "vector")]
/// Initialize the embedding model (downloads on first run, ~23MB).
pub fn init_model() -> Result<TextEmbedding> {
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true),
    )
    .context("Failed to initialize embedding model")?;
    Ok(model)
}

#[cfg(feature = "vector")]
/// Generate embedding for a single text.
pub fn embed_text(model: &TextEmbedding, text: &str) -> Result<Vec<f32>> {
    let embeddings = model
        .embed(vec![text], None)
        .context("Failed to generate embedding")?;
    Ok(embeddings.into_iter().next().unwrap())
}

#[cfg(feature = "vector")]
/// Prepare embedding input text from node fields.
/// Combines title + tags + content for richer semantic representation.
pub fn prepare_embed_text(title: &str, tags: &[String], content: &str) -> String {
    let tag_str = tags.join(" ");
    // Truncate content safely at char boundary (max ~500 chars)
    let content_preview: String = content.chars().take(500).collect();
    format!("{}\n{}\n{}", title, tag_str, content_preview)
}
