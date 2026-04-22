use anyhow::Result;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

/// Run a predefined query and return JSON results.
pub async fn run_query(db: &Surreal<Client>, query: &str) -> Result<Vec<serde_json::Value>> {
    let mut response = db.query(query).await?;
    let results: Vec<serde_json::Value> = response.take(0)?;
    Ok(results)
}

/// Impact analysis: what depends on a given node?
pub fn impact_query(record_id: &str) -> String {
    format!(
        "SELECT \
            <-depends_on<-module.title AS dependent_modules, \
            <-depends_on<-module.wiki_path AS dependent_paths, \
            ->produces->entity.title AS produced_entities, \
            ->consumes->entity.title AS consumed_entities, \
            ->implements->concept.title AS implemented_concepts \
        FROM {};",
        record_id
    )
}

/// Dependencies: what does a node depend on?
pub fn deps_query(record_id: &str) -> String {
    format!(
        "SELECT \
            ->depends_on->module.title AS depends_on_modules, \
            ->uses->module.title AS uses_modules, \
            ->uses->entity.title AS uses_entities, \
            ->part_of->entity.title AS part_of_entities, \
            ->contains->entity.title AS contains_entities \
        FROM {};",
        record_id
    )
}

/// List business rules by severity.
pub fn rules_query(severity: Option<&str>) -> String {
    match severity {
        Some(sev) => format!(
            "SELECT meta::id(id) AS br_id, rule, severity, module, ->affects->entity.title AS affected_entities \
             FROM business_rule WHERE severity = '{}' ORDER BY br_id;",
            sev
        ),
        None => "SELECT meta::id(id) AS br_id, rule, severity, module, ->affects->entity.title AS affected_entities \
                 FROM business_rule ORDER BY severity, br_id;"
            .to_string(),
    }
}

/// Module coupling ranking.
pub fn coupling_query() -> String {
    "SELECT title, wiki_path, \
        count(->depends_on) AS outgoing_deps, \
        count(<-depends_on) AS incoming_deps, \
        count(->produces) AS produces_count, \
        count(->consumes) AS consumes_count \
    FROM module ORDER BY outgoing_deps DESC;"
        .to_string()
}

/// Entity usage across modules.
pub fn entity_usage_query() -> String {
    "SELECT title, wiki_path, \
        count(<-produces<-module) AS produced_by_count, \
        count(<-consumes<-module) AS consumed_by_count, \
        <-produces<-module.title AS produced_by, \
        <-consumes<-module.title AS consumed_by \
    FROM entity ORDER BY consumed_by_count DESC;"
        .to_string()
}

/// Status overview query.
pub fn status_query() -> String {
    "SELECT \
        (SELECT count() FROM module GROUP ALL)[0].count AS modules, \
        (SELECT count() FROM entity GROUP ALL)[0].count AS entities, \
        (SELECT count() FROM concept GROUP ALL)[0].count AS concepts, \
        (SELECT count() FROM flow GROUP ALL)[0].count AS flows, \
        (SELECT count() FROM question GROUP ALL)[0].count AS questions, \
        (SELECT count() FROM business_rule GROUP ALL)[0].count AS business_rules, \
        (SELECT count() FROM depends_on GROUP ALL)[0].count AS edge_depends_on, \
        (SELECT count() FROM produces GROUP ALL)[0].count AS edge_produces, \
        (SELECT count() FROM consumes GROUP ALL)[0].count AS edge_consumes, \
        (SELECT count() FROM contains GROUP ALL)[0].count AS edge_contains, \
        (SELECT count() FROM part_of GROUP ALL)[0].count AS edge_part_of, \
        (SELECT count() FROM implements GROUP ALL)[0].count AS edge_implements, \
        (SELECT count() FROM uses GROUP ALL)[0].count AS edge_uses, \
        (SELECT count() FROM triggers GROUP ALL)[0].count AS edge_triggers, \
        (SELECT count() FROM affects GROUP ALL)[0].count AS edge_affects \
    ;"
    .to_string()
}

/// List all nodes of a given type.
pub fn list_query(node_type: &str) -> String {
    if node_type == "all" {
        "SELECT meta::id(id) AS node_id, meta::tb(id) AS node_type, title, status, wiki_path \
         FROM module, entity, concept, flow, question ORDER BY node_type, title;"
            .to_string()
    } else {
        format!(
            "SELECT meta::id(id) AS node_id, title, status, wiki_path \
             FROM {} ORDER BY title;",
            node_type
        )
    }
}

/// Search nodes by keyword (matches title and tags).
pub fn search_query(keyword: &str) -> String {
    let kw_lower = keyword.to_lowercase();
    format!(
        "SELECT meta::id(id) AS node_id, meta::tb(id) AS node_type, title, tags, wiki_path \
         FROM module, entity, concept, flow, question \
         WHERE string::lowercase(title) CONTAINS '{}' \
            OR tags CONTAINS '{}' \
         ORDER BY node_type, title;",
        kw_lower, kw_lower
    )
}

/// Get detailed info for a single node.
pub fn info_query(record_id: &str) -> String {
    format!(
        "SELECT \
            meta::id(id) AS node_id, \
            meta::tb(id) AS node_type, \
            title, status, tags, sources, wiki_path, \
            count(->depends_on) AS out_depends_on, \
            count(<-depends_on) AS in_depends_on, \
            count(->produces) AS out_produces, \
            count(<-produces) AS in_produces, \
            count(->consumes) AS out_consumes, \
            count(<-consumes) AS in_consumes, \
            count(->implements) AS out_implements, \
            count(->uses) AS out_uses, \
            count(->contains) AS out_contains, \
            count(<-contains) AS in_contains, \
            count(->part_of) AS out_part_of, \
            count(<-part_of) AS in_part_of, \
            count(->triggers) AS out_triggers, \
            count(<-triggers) AS in_triggers \
        FROM {};",
        record_id
    )
}

/// Business rules info for a module (count by severity).
pub fn info_rules_count(module_slug: &str) -> String {
    format!(
        "SELECT severity, count() AS cnt \
         FROM business_rule WHERE module = '{}' GROUP BY severity;",
        module_slug
    )
}

/// Business rules related to a specific node.
pub fn rules_for_query(record_id: &str) -> String {
    let parts: Vec<&str> = record_id.splitn(2, ':').collect();
    let node_type = parts.first().copied().unwrap_or("");
    let slug = parts.get(1).copied().unwrap_or("");

    if node_type == "module" {
        format!(
            "SELECT meta::id(id) AS br_id, rule, severity, module, \
                    ->affects->entity.title AS affected_entities \
             FROM business_rule WHERE module = '{}' ORDER BY severity, br_id;",
            slug
        )
    } else {
        // For entities: find rules that affect this entity
        format!(
            "SELECT meta::id(id) AS br_id, rule, severity, module \
             FROM business_rule WHERE ->affects CONTAINS {} ORDER BY severity, br_id;",
            record_id
        )
    }
}

/// Path: check direct edges between two nodes.
pub fn path_direct_query(from: &str, to: &str) -> String {
    format!(
        "SELECT meta::tb(id) AS edge_type, \
                string::concat(meta::tb(in), ':', meta::id(in)) AS from_node, \
                string::concat(meta::tb(out), ':', meta::id(out)) AS to_node, \
                label \
         FROM depends_on, produces, consumes, contains, part_of, implements, uses, triggers \
         WHERE (in = {} AND out = {}) OR (in = {} AND out = {});",
        from, to, to, from
    )
}

/// Path: find shared neighbors (entities consumed/produced by both modules).
pub fn path_shared_query(from: &str, to: &str) -> String {
    format!(
        "SELECT meta::id(id) AS node_id, title, wiki_path \
         FROM entity \
         WHERE (<-consumes<-module CONTAINS {} OR <-produces<-module CONTAINS {}) \
           AND (<-consumes<-module CONTAINS {} OR <-produces<-module CONTAINS {});",
        from, from, to, to
    )
}
