# Phase 1: LLM Query Enhancements

> **Status**: `completed`  
> **Priority**: `high`  
> **Estimated Effort**: 2-3 hours  
> **Created**: 2026-04-22  
> **Completed**: 2026-04-22  

---

## Mục tiêu

Bổ sung 2 query presets mới (`content`, `neighbors`) và fix export JSON thiếu business_rule nodes. Cả 3 thay đổi đều hướng đến tối ưu trải nghiệm cho LLM agent khi dùng hidow.

## Bối cảnh / Lý do

1. **`content`**: LLM hiện phải gọi `hidow query info <id>` lấy `wiki_path`, rồi tự đọc file markdown. Nếu có `content` preset, LLM chỉ cần 1 lệnh để lấy toàn bộ nội dung — giảm số bước trong pipeline.
2. **`neighbors`**: Để biết "tất cả nodes liên quan đến X", LLM phải gọi `impact` + `deps` + `rules-for` (3 lệnh). Preset `neighbors` gộp tất cả vào 1 query duy nhất.
3. **Export JSON**: Hiện `export --format json` chỉ xuất nodes (module, entity, concept, flow, question) + edges. Business rule nodes (82 records) hoàn toàn bị bỏ sót → mất data khi backup/import.

---

## Thay đổi chi tiết

### 1. `content` query preset

**File(s)**: `src/commands/query.rs`, `src/db/queries.rs`

Query trả về metadata + markdown body content của 1 node.

- [ ] Thêm `content_query(record_id)` vào `queries.rs` — SELECT title, wiki_path, content FROM {record_id}
- [ ] Thêm `"content"` branch vào match block trong `query.rs`
- [ ] Table format: in header + raw markdown content
- [ ] JSON format: `{ "title": "...", "wiki_path": "...", "content": "..." }`
- [ ] Error handling: bail nếu thiếu `<type:id>` argument
- [ ] Cập nhật help text trong `main.rs` (preset list)

**Usage**:
```bash
hidow query content module:accounting                  # Table: in markdown body
hidow query content entity:voucher --format json       # JSON: structured output
```

**Expected JSON output**:
```json
[{
  "title": "Module Accounting",
  "wiki_path": "wiki/modules/accounting",
  "content": "# Module Accounting\n\n## Mục đích\n..."
}]
```

---

### 2. `neighbors` query preset

**File(s)**: `src/commands/query.rs`, `src/db/queries.rs`

Trả về toàn bộ nodes có relationship trực tiếp với node target (bất kể edge type/direction).

- [ ] Thêm `neighbors_query(record_id)` vào `queries.rs` — query tất cả edge tables (cả in/out)
- [ ] Thêm `"neighbors"` branch vào match block trong `query.rs`
- [ ] Table format: group theo edge type, hiển thị direction (→/←)
- [ ] JSON format: object với key = edge_type, value = array of connected nodes
- [ ] Error handling: bail nếu thiếu `<type:id>` argument
- [ ] Cập nhật help text trong `main.rs` (preset list)

**Usage**:
```bash
hidow query neighbors module:claim                     # Table: grouped by edge type
hidow query neighbors entity:voucher --format json     # JSON: structured
```

**Expected JSON output**:
```json
{
  "node": "module:claim",
  "title": "Module Claim",
  "outgoing": {
    "depends_on": ["module:reinsurance_contract", "module:import_function", ...],
    "consumes": ["entity:contract", "entity:section", ...],
    "produces": ["entity:event", "entity:claim"],
    "implements": ["concept:coinsurance"]
  },
  "incoming": {
    "depends_on": ["module:technical_account", "module:retain_engine", ...],
    "triggers": []
  },
  "business_rules": {
    "critical": 4,
    "warning": 2,
    "info": 1,
    "total": 7
  }
}
```

**SurrealQL approach**: Dùng nhiều sub-queries trong 1 SELECT statement:
```sql
SELECT
    title,
    ->depends_on->module.{title, wiki_path} AS out_depends_on,
    <-depends_on<-module.{title, wiki_path} AS in_depends_on,
    ->produces->entity.{title, wiki_path} AS out_produces,
    ->consumes->entity.{title, wiki_path} AS out_consumes,
    <-consumes<-module.{title, wiki_path} AS in_consumes,
    ->implements->concept.{title, wiki_path} AS out_implements,
    ->uses.{title, wiki_path} AS out_uses,
    ->contains->entity.{title, wiki_path} AS out_contains,
    <-contains.{title, wiki_path} AS in_contains,
    ->part_of.{title, wiki_path} AS out_part_of,
    <-part_of.{title, wiki_path} AS in_part_of,
    ->triggers.{title, wiki_path} AS out_triggers,
    <-triggers.{title, wiki_path} AS in_triggers
FROM module:claim;
```

---

### 3. Export JSON bao gồm business_rule nodes

**File(s)**: `src/commands/export.rs`

- [ ] Thêm `"business_rule"` vào default tables list (line 13)
- [ ] Thêm `"business_rules"` key riêng trong JSON output (tách biệt nodes thường)
- [ ] Hoặc: gộp business_rule vào nodes array nhưng có `node_type: "business_rule"` để phân biệt
- [ ] Cập nhật DOT export: thêm business_rule nodes với color riêng (VD: `#ffcdd2` đỏ nhạt)
- [ ] Cập nhật CSV export: thêm business_rule rows

**Quyết định thiết kế cần chọn**:

Option A — Gộp vào `nodes` array:
```json
{
  "nodes": [...existing..., {"node_type": "business_rule", "id": "BR_ACC_001", ...}],
  "edges": [...]
}
```

Option B — Tách riêng `business_rules` key:
```json
{
  "nodes": [...],
  "edges": [...],
  "business_rules": [{"br_id": "BR_ACC_001", "rule": "...", "severity": "critical", ...}]
}
```

**Đề xuất**: Option B — tách riêng vì business_rule có schema khác (rule, severity, module thay vì title, status, tags).

---

## Verification

### Automated
```bash
cargo test
cargo build --release
```

### Manual
- [ ] `hidow query content module:accounting` → in ra markdown body
- [ ] `hidow query content module:accounting --format json` → valid JSON với "content" field
- [ ] `hidow query neighbors module:claim` → list tất cả related nodes, grouped by edge type
- [ ] `hidow query neighbors module:claim --format json` → valid JSON parseable bởi python
- [ ] `hidow export --format json | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d['business_rules']))"` → 82
- [ ] `hidow export --format dot` → business_rule nodes xuất hiện với color đỏ nhạt
- [ ] Tất cả 11 presets cũ vẫn hoạt động bình thường (regression test)

---

## Ghi chú / Quyết định thiết kế

- `content` preset trả về raw markdown body từ DB (field `content` đã được store khi ingest). Không cần đọc file hệ thống.
- `neighbors` preset phải handle cả module, entity, concept — không chỉ module. Edge types khác nhau tùy node type.
- Export business_rules dùng Option B (tách key riêng) vì schema khác biệt.
- Tất cả new presets phải support `--format json` với clean stdout (dùng `header!` macro).

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 0 ✅ (embedded SurrealKV + clean JSON)
- External dependencies: Không có
