use anyhow::Result;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

/// Define the SurrealDB schema: node tables, edge tables, fields, and indexes.
pub async fn define_schema(db: &Surreal<Client>) -> Result<()> {
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

        -- Question nodes
        DEFINE TABLE question SCHEMALESS;
        DEFINE FIELD title ON question TYPE string;
        DEFINE FIELD status ON question TYPE string;
        DEFINE FIELD tags ON question TYPE array;
        DEFINE FIELD sources ON question TYPE array;
        DEFINE FIELD content ON question TYPE string;
        DEFINE FIELD content_hash ON question TYPE string;
        DEFINE FIELD wiki_path ON question TYPE string;

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
