use anyhow::Result;

use super::DbConn;

/// Default node types created on first init.
const DEFAULT_NODE_TABLES: &[&str] = &[
    "module", "entity", "concept", "flow", "question", "overview",
];

/// Define schema for a single node table (idempotent — safe to call multiple times).
pub async fn define_node_table(db: &DbConn, table: &str) -> Result<()> {
    let q = format!(
        "DEFINE TABLE {t} SCHEMALESS;
         DEFINE FIELD title ON {t} TYPE string;
         DEFINE FIELD status ON {t} TYPE string;
         DEFINE FIELD tags ON {t} TYPE array;
         DEFINE FIELD sources ON {t} TYPE array;
         DEFINE FIELD content ON {t} TYPE string;
         DEFINE FIELD content_hash ON {t} TYPE string;
         DEFINE FIELD wiki_path ON {t} TYPE string;
         DEFINE FIELD embedding ON {t} TYPE option<array<float>>;
         DEFINE INDEX idx_{t}_hash ON {t} FIELDS content_hash;
         DEFINE INDEX idx_{t}_emb ON {t} FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;",
        t = table
    );
    db.query(&q).await?;
    Ok(())
}

/// Define the SurrealDB schema: node tables, edge tables, fields, and indexes.
pub async fn define_schema(db: &DbConn) -> Result<()> {
    // ──────────────────────────────────────────────
    // Node tables (default types)
    // ──────────────────────────────────────────────

    for table in DEFAULT_NODE_TABLES {
        define_node_table(db, table).await?;
    }

    // ── Entity-specific fields ──
    db.query("DEFINE FIELD attributes ON entity TYPE array;")
        .await?;

    // ── Flow-specific fields ──
    db.query("DEFINE FIELD data_flow ON flow TYPE array;")
        .await?;

    // ── Business Rule nodes ──
    db.query(
        "DEFINE TABLE business_rule SCHEMALESS;
         DEFINE FIELD rule ON business_rule TYPE string;
         DEFINE FIELD severity ON business_rule TYPE string;
         DEFINE FIELD module ON business_rule TYPE string;
         DEFINE INDEX idx_br_severity ON business_rule FIELDS severity;",
    )
    .await?;

    // ──────────────────────────────────────────────
    // Edge tables (TYPE RELATION)
    // ──────────────────────────────────────────────

    db.query(
        "DEFINE TABLE depends_on TYPE RELATION;
        DEFINE FIELD label ON depends_on TYPE string;

        DEFINE TABLE produces TYPE RELATION;
        DEFINE FIELD label ON produces TYPE string;

        DEFINE TABLE consumes TYPE RELATION;
        DEFINE FIELD label ON consumes TYPE string;

        DEFINE TABLE contains TYPE RELATION;
        DEFINE FIELD label ON contains TYPE string;

        DEFINE TABLE part_of TYPE RELATION;
        DEFINE FIELD label ON part_of TYPE string;

        DEFINE TABLE implements TYPE RELATION;
        DEFINE FIELD label ON implements TYPE string;

        DEFINE TABLE uses TYPE RELATION;
        DEFINE FIELD label ON uses TYPE string;

        DEFINE TABLE triggers TYPE RELATION;
        DEFINE FIELD label ON triggers TYPE string;

        DEFINE TABLE affects TYPE RELATION;
        DEFINE FIELD label ON affects TYPE string;
        ",
    )
    .await?;

    Ok(())
}
