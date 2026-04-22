use serde::{Deserialize, Serialize};

/// Represents the full YAML frontmatter of a wiki page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WikiFrontmatter {
    pub title: String,
    #[serde(rename = "type")]
    pub page_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub business_rules: Vec<BusinessRule>,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(default)]
    pub data_flow: Vec<DataFlowStep>,
}

/// A typed relationship to another wiki page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relationship {
    pub target: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(default)]
    pub label: String,
}

/// A business rule defined in a module page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BusinessRule {
    pub id: String,
    pub rule: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub affects: Vec<String>,
}

/// A structured attribute on an entity page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub reference: Option<String>,
}

/// A step in a data flow (for flow pages).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataFlowStep {
    pub step: u32,
    pub name: String,
    #[serde(default)]
    pub module: String,
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

/// A fully parsed wiki page with metadata and content.
#[derive(Debug, Clone)]
pub struct WikiPage {
    /// Relative path without .md, e.g. "wiki/modules/accounting"
    pub path: String,
    /// Slug for SurrealDB record ID, e.g. "accounting"
    pub slug: String,
    /// Parsed frontmatter
    pub frontmatter: WikiFrontmatter,
    /// Markdown body content (after frontmatter)
    pub content: String,
    /// SHA256 hash of the full file content
    pub content_hash: String,
}
