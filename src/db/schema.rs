use anyhow::Result;

use super::DbConn;

/// Define the SurrealDB schema: node tables, edge tables, fields, and indexes.
pub async fn define_schema(db: &DbConn) -> Result<()> {
    // ──────────────────────────────────────────────
    // Node tables
    // ──────────────────────────────────────────────

    db.query(
        "
        -- Module nodes
        DEFINE TABLE module SCHEMALESS;
        DEFINE FIELD title ON module TYPE string;
        DEFINE FIELD status ON module TYPE string;
        DEFINE FIELD tags ON module TYPE array;
        DEFINE FIELD sources ON module TYPE array;
        DEFINE FIELD content ON module TYPE string;
        DEFINE FIELD content_hash ON module TYPE string;
        DEFINE FIELD wiki_path ON module TYPE string;
        DEFINE INDEX idx_module_hash ON module FIELDS content_hash;
        DEFINE FIELD embedding ON module TYPE option<array<float>>;
        DEFINE INDEX idx_module_emb ON module FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Entity nodes
        DEFINE TABLE entity SCHEMALESS;
        DEFINE FIELD title ON entity TYPE string;
        DEFINE FIELD status ON entity TYPE string;
        DEFINE FIELD tags ON entity TYPE array;
        DEFINE FIELD sources ON entity TYPE array;
        DEFINE FIELD content ON entity TYPE string;
        DEFINE FIELD content_hash ON entity TYPE string;
        DEFINE FIELD wiki_path ON entity TYPE string;
        DEFINE FIELD attributes ON entity TYPE array;
        DEFINE INDEX idx_entity_hash ON entity FIELDS content_hash;
        DEFINE FIELD embedding ON entity TYPE option<array<float>>;
        DEFINE INDEX idx_entity_emb ON entity FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Concept nodes
        DEFINE TABLE concept SCHEMALESS;
        DEFINE FIELD title ON concept TYPE string;
        DEFINE FIELD status ON concept TYPE string;
        DEFINE FIELD tags ON concept TYPE array;
        DEFINE FIELD sources ON concept TYPE array;
        DEFINE FIELD content ON concept TYPE string;
        DEFINE FIELD content_hash ON concept TYPE string;
        DEFINE FIELD wiki_path ON concept TYPE string;
        DEFINE INDEX idx_concept_hash ON concept FIELDS content_hash;
        DEFINE FIELD embedding ON concept TYPE option<array<float>>;
        DEFINE INDEX idx_concept_emb ON concept FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Flow nodes
        DEFINE TABLE flow SCHEMALESS;
        DEFINE FIELD title ON flow TYPE string;
        DEFINE FIELD status ON flow TYPE string;
        DEFINE FIELD tags ON flow TYPE array;
        DEFINE FIELD sources ON flow TYPE array;
        DEFINE FIELD content ON flow TYPE string;
        DEFINE FIELD content_hash ON flow TYPE string;
        DEFINE FIELD wiki_path ON flow TYPE string;
        DEFINE FIELD data_flow ON flow TYPE array;
        DEFINE INDEX idx_flow_hash ON flow FIELDS content_hash;
        DEFINE FIELD embedding ON flow TYPE option<array<float>>;
        DEFINE INDEX idx_flow_emb ON flow FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Question nodes
        DEFINE TABLE question SCHEMALESS;
        DEFINE FIELD title ON question TYPE string;
        DEFINE FIELD status ON question TYPE string;
        DEFINE FIELD tags ON question TYPE array;
        DEFINE FIELD sources ON question TYPE array;
        DEFINE FIELD content ON question TYPE string;
        DEFINE FIELD content_hash ON question TYPE string;
        DEFINE FIELD wiki_path ON question TYPE string;
        DEFINE FIELD embedding ON question TYPE option<array<float>>;
        DEFINE INDEX idx_question_emb ON question FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Overview nodes (system architecture, meta-documentation)
        DEFINE TABLE overview SCHEMALESS;
        DEFINE FIELD title ON overview TYPE string;
        DEFINE FIELD status ON overview TYPE string;
        DEFINE FIELD tags ON overview TYPE array;
        DEFINE FIELD sources ON overview TYPE array;
        DEFINE FIELD content ON overview TYPE string;
        DEFINE FIELD content_hash ON overview TYPE string;
        DEFINE FIELD wiki_path ON overview TYPE string;
        DEFINE FIELD embedding ON overview TYPE option<array<float>>;
        DEFINE INDEX idx_overview_emb ON overview FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;

        -- Business Rule nodes
        DEFINE TABLE business_rule SCHEMALESS;
        DEFINE FIELD rule ON business_rule TYPE string;
        DEFINE FIELD severity ON business_rule TYPE string;
        DEFINE FIELD module ON business_rule TYPE string;
        DEFINE INDEX idx_br_severity ON business_rule FIELDS severity;
        ",
    )
    .await?;

    // ──────────────────────────────────────────────
    // Edge tables (TYPE RELATION)
    // ──────────────────────────────────────────────

    db.query(
        "
        DEFINE TABLE depends_on TYPE RELATION;
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
