# Phase 6: Dynamic Page Types — Loại bỏ hardcode node tables

> **Status**: `completed`  
> **Priority**: `high`  
> **Estimated Effort**: 4-6 hours  
> **Created**: 2026-05-14  
> **Completed**: 2026-05-14  

---

## Mục tiêu

Refactor hidow để **tự động phát hiện và tạo node types** khi ingest, thay vì hardcode 6 types cố định. Wiki page với bất kỳ `type: workflow` hay `type: policy` sẽ tự động trở thành first-class citizen — có schema, embedding index, và xuất hiện trong tất cả queries.

## Bối cảnh / Lý do

Hiện tại hidow hardcode 6 node types (`module`, `entity`, `concept`, `flow`, `question`, `overview`) tại **12 vị trí** trong 7 files. Thêm 1 type mới = sửa 7 files. Không scale khi NIMP wiki mở rộng thêm types mới (workflow, policy, integration, ...).

---

## Audit: 12 vị trí hardcode

| # | File | Line | Hardcode | Tác dụng |
|---|------|------|----------|----------|
| 1 | `db/schema.rs` | 11-89 | 6 blocks `DEFINE TABLE` | Schema definition |
| 2 | `commands/ingest.rs` | 25-28 | `.page_type == "module"` x4 | Count log display |
| 3 | `commands/ingest.rs` | 94 | `tables = ["module", ...]` | Embedding generation |
| 4 | `commands/query.rs` | 72,82,94 | `valid = ["module", ...]` | list/list-detail/context validation |
| 5 | `commands/query.rs` | 115 | `tables = ["module", ...]` | Hybrid search |
| 6 | `commands/query.rs` | 331 | `tables = ["module", ...]` | Semantic search |
| 7 | `commands/query.rs` | 377 | `tables = ["module", ...]` | Ask/RAG query |
| 8 | `commands/status.rs` | 16 | `tables = ["module", ...]` | Status count |
| 9 | `commands/export.rs` | 13 | `vec!["module", ...]` | Export default tables |
| 10 | `commands/init.rs` | 14 | Print message hardcode | Init log |
| 11 | `db/queries.rs` | 80,96,122,297 | `FROM module, entity, ...` | list, list-detail, search, hybrid queries |
| 12 | `commands/export.rs` | 91-101 | DOT color mapping | Export DOT colors |

---

## Thay đổi chi tiết

### 1. Centralized helper: `db::node_tables()`

**File**: `src/db/mod.rs` [MODIFY]

Thêm function query SurrealDB metadata để lấy danh sách node tables hiện có:

```rust
/// Edge tables — these are fixed (TYPE RELATION).
const EDGE_TABLES: &[&str] = &[
    "depends_on", "produces", "consumes", "contains",
    "part_of", "implements", "uses", "triggers", "affects",
];

/// Get all node table names from DB (excludes edge tables and business_rule).
pub async fn node_tables(db: &DbConn) -> Result<Vec<String>> {
    let result: Vec<serde_json::Value> = db
        .query("INFO FOR DB;")
        .await?
        .take(0)?;
    
    let mut tables = Vec::new();
    if let Some(info) = result.first() {
        if let Some(tb) = info.get("tables").and_then(|v| v.as_object()) {
            for name in tb.keys() {
                // Skip edge tables and business_rule
                if !EDGE_TABLES.contains(&name.as_str()) && name != "business_rule" {
                    tables.push(name.clone());
                }
            }
        }
    }
    tables.sort();
    Ok(tables)
}

/// Build a FROM clause string: "module, entity, concept, ..."
pub async fn node_tables_clause(db: &DbConn) -> Result<String> {
    let tables = node_tables(db).await?;
    Ok(tables.join(", "))
}
```

- [ ] Add `EDGE_TABLES` constant
- [ ] Add `node_tables()` function
- [ ] Add `node_tables_clause()` helper
- [ ] Add `define_node_table()` function (see section 2)

### 2. Dynamic schema creation at ingest time

**File**: `src/db/schema.rs` [MODIFY]

Thay 6 blocks `DEFINE TABLE` hardcode bằng 1 hàm dynamic:

```rust
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
```

`define_schema()` sẽ giữ lại nhưng chỉ define:
- Edge tables (cố định — TYPE RELATION)  
- `business_rule` table (cố định — special structure)
- Default node tables (`module`, `entity`, `concept`, `flow`, `question`, `overview`) qua loop gọi `define_node_table()`

- [ ] Extract `define_node_table()` function
- [ ] Refactor `define_schema()` to loop qua default types
- [ ] Giữ edge tables hardcode (chúng là cố định, khác structure)

### 3. Auto-create type on ingest

**File**: `src/db/loader.rs` [MODIFY]

Trước khi `CREATE` node, check xem table đã tồn tại chưa. Nếu chưa → gọi `define_node_table()`:

```rust
// Phase 1: Create nodes
let known_tables = db::node_tables(db).await?;
for page in pages {
    let table = &page.frontmatter.page_type;
    
    // Auto-create schema for new types
    if !known_tables.contains(&table.to_string()) {
        eprintln!("  🆕 New type '{}' — creating schema...", table);
        db::schema::define_node_table(db, table).await?;
        known_tables.push(table.to_string());
    }
    
    // CREATE node as usual...
}
```

- [ ] Add auto-detect new type logic
- [ ] Call `define_node_table()` for unknown types
- [ ] Track discovered types in memory to avoid repeated define calls

### 4. Dynamic ingest stats

**File**: `src/commands/ingest.rs` [MODIFY]

Thay hardcode count 4 types bằng dynamic grouping:

```rust
// Before (hardcode):
// pages.iter().filter(|p| p.frontmatter.page_type == "module").count()

// After (dynamic):
let mut type_counts: HashMap<&str, usize> = HashMap::new();
for page in &pages {
    *type_counts.entry(&page.frontmatter.page_type).or_insert(0) += 1;
}
println!("  Found {} pages:", pages.len());
for (ptype, count) in &type_counts {
    println!("    {}: {}", ptype, count);
}
```

- [ ] Replace 4 hardcoded `.filter()` with dynamic HashMap grouping
- [ ] Replace hardcoded `tables` in `generate_embeddings()` with `db::node_tables()`

### 5. Dynamic queries

**File**: `src/db/queries.rs` [MODIFY]

Thay hardcoded `FROM module, entity, ...` bằng parameter:

```rust
// Before:
pub fn list_query(node_type: &str) -> String {
    if node_type == "all" {
        "SELECT ... FROM module, entity, concept, flow, question, overview ..."
    }
}

// After — accept from_clause parameter:
pub fn list_query(node_type: &str, all_tables: &str) -> String {
    if node_type == "all" {
        format!("SELECT ... FROM {} ...", all_tables)
    }
}
```

Affected queries (4):
- `list_query()` — line 80
- `list_detail_query()` — line 96
- `search_query()` — line 122
- `keyword_search_for_hybrid()` — line 297

- [ ] Add `all_tables: &str` parameter to 4 query functions
- [ ] Remove hardcoded FROM clauses

### 6. Dynamic query command

**File**: `src/commands/query.rs` [MODIFY]

- [ ] Fetch `node_tables()` at start of `run()` → build `all_tables` clause
- [ ] Replace `valid = ["module", ...]` lists with dynamic tables from DB
- [ ] Replace `tables = ["module", ...]` in search/semantic/ask with dynamic list
- [ ] Pass `all_tables` to query functions

```rust
pub async fn run(...) -> Result<()> {
    let conn = db::connect(data_dir, "nimp", "wiki").await?;
    
    // Dynamic: get all node tables from DB
    let all_tables = db::node_tables(&conn).await?;
    let all_tables_clause = all_tables.join(", ");
    
    match preset {
        "list" => {
            let node_type = args.first().map(|s| s.as_str()).unwrap_or("all");
            // Dynamic validation
            if node_type != "all" && !all_tables.contains(&node_type.to_string()) {
                bail!("Invalid type '{}'. Available: {}, all", node_type, all_tables_clause);
            }
            db::queries::list_query(node_type, &all_tables_clause)
        }
        "search" => {
            // Hybrid search loops through dynamic tables
            for table in &all_tables { ... }
        }
        // ...
    }
}
```

### 7. Dynamic status & export

**File**: `src/commands/status.rs` [MODIFY]

- [ ] Replace `tables = ["module", ...]` with `db::node_tables()` call

**File**: `src/commands/export.rs` [MODIFY]

- [ ] Replace `vec!["module", ...]` with `db::node_tables()` call
- [ ] DOT color mapping: assign colors dynamically (use a palette array)

**File**: `src/commands/init.rs` [MODIFY]

- [ ] Update print message to be dynamic or generic

---

## Migration / Backward Compatibility

> [!IMPORTANT]
> **Không breaking change.** Existing databases vẫn hoạt động bình thường:
> - `define_schema()` vẫn tạo 6 default types
> - `node_tables()` sẽ discover chúng từ DB metadata
> - Chỉ khi wiki có `type: workflow` → tự tạo thêm bảng mới

---

## Verification

### Automated
```bash
# 1. Build
cargo build

# 2. Existing types still work
hidow ingest --dry-run
hidow query list all --format json | jq length
hidow query search "claim" --format json

# 3. Add a test wiki page with custom type
cat > /tmp/test-workflow.md << 'EOF'
---
title: "Test Workflow"
type: workflow
status: active
tags: [test]
---
# Test Workflow
This is a custom type test.
EOF

# 4. Ingest custom type
hidow ingest --file /tmp/test-workflow.md

# 5. Verify custom type is queryable
hidow query list workflow              # Should show "Test Workflow"
hidow query list all --format json     # Should include workflow type
hidow query info workflow:test_workflow
hidow query search "workflow"          # Should find it
hidow query semantic "test"            # Should find it (has embedding)
hidow status                           # Should show workflow count
```

### Manual
- [ ] Verify `hidow status` shows custom type count
- [ ] Verify `hidow query list all` includes custom type nodes
- [ ] Verify semantic search finds custom type nodes
- [ ] Verify `hidow export --format json` includes custom type
- [ ] Verify re-ingest doesn't duplicate schema definitions (idempotent)

---

## Ghi chú / Quyết định thiết kế

1. **Edge tables stay hardcoded**: Edge types (`depends_on`, `produces`, ...) represent relationship semantics. Unlike node types, they have fixed structure (`TYPE RELATION`) and adding new edge types requires understanding graph semantics. Not worth making dynamic.
2. **`INFO FOR DB` as source of truth**: SurrealDB's metadata query lists all tables. Filter out edge tables → remaining = node tables. Simple, no extra tracking needed.
3. **Idempotent `DEFINE TABLE`**: SurrealDB `DEFINE TABLE ... SCHEMALESS` is safe to call multiple times — won't error or reset data. So `define_node_table()` can be called freely.
4. **Default types preserved**: `define_schema()` still creates 6 default types on first init. This ensures backward compatibility and a clean starting point.
5. **Performance**: `node_tables()` queries `INFO FOR DB` once per command invocation (~1ms). Negligible overhead.

---

## Dependencies

- Phase(s) phải hoàn thành trước: None (independent of Phase 3-5)
- External: None
