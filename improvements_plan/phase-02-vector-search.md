# Phase 2: Vector Search với fastembed-rs

> **Status**: `planned`  
> **Priority**: `high`  
> **Estimated Effort**: 6-8 hours (chia 3 sub-phases)  
> **Created**: 2026-04-22  
> **Completed**: —  

---

## Mục tiêu

Tích hợp semantic vector search vào hidow bằng `fastembed-rs` (local, offline, zero API key). Toàn bộ embedding + search chạy embedded trong SurrealDB, giữ nguyên triết lý **1 binary, 1 DB, zero config**.

## Bối cảnh / Lý do

Keyword search hiện tại (`hidow query search`) chỉ match exact title + tags:
- `search "tính phí"` → 0 results (vì không có node nào title chứa "tính phí")
- `search "premium"` → tìm được (vì match tag)

Semantic search sẽ **hiểu ý nghĩa** → tìm được "premium", "commission", "calculation engine" khi hỏi "tính phí".

---

## Kiến trúc

```
                     hidow CLI
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
         fastembed    SurrealDB   Rust CLI
         (ONNX)      (SurrealKV)  (clap)
              │          │
              │    ┌─────┴─────┐
              │    │  nodes    │ ← title, content, tags, wiki_path
              └───▶│  +embed   │ ← embedding: array<float> [384 dims]
                   │  MTREE    │ ← DEFINE INDEX ... MTREE DIMENSION 384
                   └───────────┘
```

### Tech Stack

| Component | Crate | Version | Vai trò |
|-----------|-------|---------|---------|
| Embedding | `fastembed` | 5.x | Local ONNX inference, no API key |
| Model | `all-MiniLM-L6-v2` | — | 384 dims, ~23MB, multilingual OK |
| Vector Index | SurrealDB MTREE | v2 (đang dùng) | KNN search, cosine distance |
| Feature flag | `vector` | — | `cargo build --features vector` |

### Model cache
```
~/.cache/huggingface/hub/
└── models--Qdrant--all-MiniLM-L6-v2/   (~23MB, download 1 lần)
```

---

## Thay đổi chi tiết

### Phase 2A: Foundation — Embed + Similar (~3 hours)

#### 1. Cargo.toml

**File**: `Cargo.toml`

- [ ] Thêm `fastembed` dependency với feature flag

```toml
[features]
default = []
vector = ["fastembed"]

[dependencies]
fastembed = { version = "5", optional = true }
```

#### 2. Embedding module

**File**: `src/db/embed.rs` [NEW]

- [ ] Tạo module embed.rs wrapper cho fastembed
- [ ] Function `init_model()` → TextEmbedding instance (cached)
- [ ] Function `embed_text(text: &str) -> Vec<f32>` → embedding vector
- [ ] Function `embed_batch(texts: &[String]) -> Vec<Vec<f32>>` → batch embeddings
- [ ] Cfg-guard toàn bộ module: `#[cfg(feature = "vector")]`

```rust
#[cfg(feature = "vector")]
pub mod embed {
    use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
    
    pub fn init_model() -> anyhow::Result<TextEmbedding> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            ..Default::default()
        })?;
        Ok(model)
    }
    
    pub fn embed_text(model: &TextEmbedding, text: &str) -> anyhow::Result<Vec<f32>> {
        let embeddings = model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
    
    pub fn embed_batch(model: &TextEmbedding, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let embeddings = model.embed(refs, None)?;
        Ok(embeddings)
    }
}
```

#### 3. Schema updates

**File**: `src/db/schema.rs`

- [ ] Thêm `embedding` field (optional) vào node tables
- [ ] Thêm MTREE index definitions

```sql
-- Fields (thêm vào mỗi table)
DEFINE FIELD embedding ON TABLE module TYPE option<array<float>>;
DEFINE FIELD embedding ON TABLE entity TYPE option<array<float>>;
DEFINE FIELD embedding ON TABLE concept TYPE option<array<float>>;
DEFINE FIELD embedding ON TABLE flow TYPE option<array<float>>;
DEFINE FIELD embedding ON TABLE question TYPE option<array<float>>;

-- Indexes
DEFINE INDEX idx_module_emb ON TABLE module FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;
DEFINE INDEX idx_entity_emb ON TABLE entity FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;
DEFINE INDEX idx_concept_emb ON TABLE concept FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;
DEFINE INDEX idx_flow_emb ON TABLE flow FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;
DEFINE INDEX idx_question_emb ON TABLE question FIELDS embedding MTREE DIMENSION 384 DIST COSINE TYPE F32;
```

#### 4. Ingest --embed

**File**: `src/commands/ingest.rs`

- [ ] Thêm `--embed` flag vào ingest command
- [ ] Sau khi upsert nodes, generate embeddings cho changed nodes
- [ ] Embedding input = `"{title}\n{tags}\n{content}"` (concat metadata + body)
- [ ] UPDATE node SET embedding = [...] cho mỗi node

```rust
// Pseudo-code
if embed_flag {
    eprintln!("🧠 Generating embeddings...");
    let model = embed::init_model()?;
    for node in changed_nodes {
        let text = format!("{}\n{}\n{}", node.title, node.tags.join(" "), node.content);
        let vector = embed::embed_text(&model, &text)?;
        db.query(format!("UPDATE {} SET embedding = {:?}", node.id, vector)).await?;
    }
    eprintln!("✅ {} embeddings generated", changed_nodes.len());
}
```

#### 5. `similar` query preset

**File**: `src/commands/query.rs`, `src/db/queries.rs`

- [ ] Thêm `similar_query(record_id, k)` vào queries.rs
- [ ] Thêm `"similar"` branch vào query.rs match block
- [ ] Table format: ranked list với score
- [ ] JSON format: array of {node_id, title, wiki_path, score}

```sql
-- Step 1: Lấy embedding của target
LET $target = (SELECT embedding FROM module:claim)[0].embedding;

-- Step 2: KNN search (exclude self)
SELECT 
    meta::id(id) AS node_id,
    meta::tb(id) AS node_type,
    title, wiki_path,
    vector::similarity::cosine(embedding, $target) AS score
FROM module 
WHERE embedding <|6|> $target
AND id != module:claim
ORDER BY score DESC
LIMIT 5;
```

**Usage**:
```bash
hidow query similar module:claim                    # Top 5 similar modules
hidow query similar entity:voucher --format json     # JSON output
```

**Expected output**:
```
🔍 Similar to: module:claim (Module Claim)

  1. Module Policy & Endorsement      score: 0.87
  2. Module Retain Engine              score: 0.82
  3. Module Technical Account          score: 0.79
  4. Module Reinsurance Contract       score: 0.74
  5. Module Report                     score: 0.68
```

---

### Phase 2B: Semantic Search (~2 hours)

#### 6. `semantic` query preset

**File**: `src/commands/query.rs`, `src/db/queries.rs`

- [ ] Embed query text tại runtime: `embed_text(model, question)`
- [ ] KNN search across ALL tables (module + entity + concept + flow)
- [ ] Merge + sort by score
- [ ] Thêm `"semantic"` branch vào match block

```bash
hidow query semantic "tính phí bảo hiểm"            # Tìm theo ý nghĩa
hidow query semantic "cách xử lý bồi thường"        # Tiếng Việt OK
```

**Expected output**:
```
🧠 Semantic search: "tính phí bảo hiểm"

  1. [entity]  Premium                          score: 0.89
  2. [module]  Module Calculation Engine Setting score: 0.84
  3. [entity]  Commission                       score: 0.81
  4. [concept] Proportional Treaty               score: 0.78
  5. [module]  Module Retain Engine              score: 0.75
```

#### 7. Hybrid search (keyword + semantic)

**File**: `src/commands/query.rs`

- [ ] Cập nhật `search` preset: nếu có embeddings, combine keyword score + vector score
- [ ] Sử dụng Reciprocal Rank Fusion (RRF) để merge 2 ranking lists
- [ ] Fallback: nếu không có embeddings → keyword search như cũ

---

### Phase 2C: RAG Context Retrieval (~2 hours)

#### 8. `ask` query preset

**File**: `src/commands/query.rs`, `src/db/queries.rs`

- [ ] Embed question → KNN → trả top-k relevant **content chunks**
- [ ] Output bao gồm: title, wiki_path, relevant_excerpt, score
- [ ] JSON output tối ưu cho LLM system prompt

```bash
hidow query ask "làm sao tính retro cho XOL treaty?" --top 3 --format json
```

**Expected JSON**:
```json
{
  "question": "làm sao tính retro cho XOL treaty?",
  "context": [
    {
      "node": "module:retain_engine",
      "title": "Module Retain Engine",
      "wiki_path": "wiki/modules/retain-engine",
      "score": 0.91,
      "content": "# Module Retain Engine\n\n## XOL Calculation\n..."
    },
    {
      "node": "concept:non_proportional_treaty",
      "title": "Non-Proportional Treaty",
      "score": 0.85,
      "content": "..."
    }
  ]
}
```

---

## CLI Changes

**File**: `src/main.rs`

- [ ] Thêm `--embed` flag vào Ingest command
- [ ] Cập nhật preset help text
- [ ] Thêm `embed` subcommand riêng (optional): `hidow embed` — chạy embedding không cần ingest lại

```rust
Ingest {
    #[arg(long)]
    full: bool,
    #[arg(long)]
    dry_run: bool,
    /// Generate vector embeddings (requires 'vector' feature)
    #[arg(long)]
    embed: bool,
},
```

---

## Verification

### Phase 2A
```bash
# Build với feature flag
cargo build --features vector

# Ingest + embed
hidow --wiki-path ./wiki ingest --embed

# Test similar
hidow query similar module:claim --format json | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'{len(d)} results, top={d[0][\"title\"]} score={d[0][\"score\"]:.2f}')"
```

### Phase 2B
```bash
# Semantic search tiếng Việt
hidow query semantic "tính phí" --format json
hidow query semantic "bồi thường" --format json

# So sánh keyword vs semantic
hidow query search "tính phí"           # → 0 results
hidow query semantic "tính phí"          # → Premium, Commission, ...
```

### Phase 2C
```bash
# RAG context
hidow query ask "XOL calculation" --top 3 --format json | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'{len(d[\"context\"])} chunks, top={d[\"context\"][0][\"title\"]}')"
```

### Regression
```bash
# Build KHÔNG có feature flag → tool hoạt động như cũ
cargo build
hidow query search claim --format json   # vẫn hoạt động
hidow query similar module:claim          # Error: "Requires --features vector"
```

---

## Ghi chú / Quyết định thiết kế

1. **Feature flag `vector`**: Build mặc định KHÔNG có vector → binary nhỏ, compile nhanh. Chỉ ai cần vector search mới build `--features vector`.
2. **Model choice `all-MiniLM-L6-v2`**: 384 dims, 23MB, multilingual acceptable. Nếu tiếng Việt kém → upgrade `bge-m3` (1024 dims, ~200MB).
3. **Embedding input**: concat `title + tags + content` để embedding capture cả metadata lẫn body.
4. **Smart embed**: Chỉ re-embed nodes có content thay đổi (dùng SHA-256 hash giống smart sync).
5. **Graceful degradation**: Nếu node chưa có embedding → `similar`/`semantic` trả lời "Run `hidow ingest --embed` first".

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 1 ✅
- External: `fastembed` crate v5 + ONNX Runtime (bundled)
- Model download: `~/.cache/huggingface/hub/` (~23MB, 1 lần)
