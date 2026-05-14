# Hidow

`hidow` là một công cụ CLI hiệu năng cao được viết bằng Rust, dùng để phân tích tài liệu Wiki (kiến trúc Markdown + YAML Frontmatter) và đồng bộ hóa thành một **Knowledge Graph** trên SurrealDB (embedded).

Hỗ trợ **multi-instance** — mỗi project có knowledge graph riêng biệt, isolated hoàn toàn trong cùng 1 database storage.

Công cụ này giúp developer và BA dễ dàng truy vấn mối quan hệ giữa các Modules, Entities, Concepts, theo dõi Business Rules, phân tích mức độ Coupling (phụ thuộc) và Impact (tác động) khi thay đổi hệ thống.

---

## Tính năng cốt lõi

- ⚡️ **Smart Sync**: Quét toàn bộ wiki directory, sử dụng mã băm `SHA-256` để chỉ cập nhật những file markdown có thay đổi, tối ưu hóa tốc độ Ingest.
- 🕸 **Embedded Graph Database**: SurrealDB chạy trực tiếp trong process (SurrealKV engine), không cần Docker hay service ngoài.
- 📦 **Multi-Instance**: Mỗi project = 1 instance riêng biệt (`-i <name>`). Quản lý bằng `hidow instance list`.
- 🔀 **Dynamic Page Types**: Tự động tạo schema cho bất kỳ page type mới khi ingest (không giới hạn 6 types cố định).
- 🔍 **16 Query Presets**: Hỗ trợ các query lập trình sẵn để phân tích kiến trúc: Discovery, Impact Analysis, Coupling, Content retrieval.
- 🧠 **Vector Search**: Semantic search, similarity ranking, RAG context retrieval bằng `fastembed-rs` + ONNX Runtime (local, offline, zero API key). Embeddings được tự động generate khi ingest.
- 🛠 **Linter**: Kiểm tra sức khỏe của Wiki, phát hiện Orphan nodes, đảm bảo Graph Database và Wiki luôn đồng bộ (100% in-sync).
- 📤 **Export linh hoạt**: Xuất Graph ra định dạng `JSON` (bao gồm Business Rules), `CSV` và `DOT` (tương thích Graphviz).

---

## Cài đặt

### Yêu cầu hệ thống
- **Rust Toolchain** (v1.75+)

### Build & Install CLI

**Cách nhanh nhất** — Chạy script setup (download ONNX Runtime + build + install global):
```bash
./scripts/setup.sh
```

Script sẽ tự động:
1. Detect ONNX Runtime — download nếu chưa có (~16MB, 1 lần)
2. Build release
3. Copy binary vào `~/.cargo/bin/hidow`

Sau khi install, gọi `hidow` ở bất kỳ đâu trên terminal.

Database tự động lưu ở `~/.hidow/data/`. Có thể thay đổi bằng flag `--data-dir <PATH>`.

---

## Hướng dẫn sử dụng

> **⚠️ Tất cả lệnh yêu cầu flag `-i <instance>`**. Nếu không truyền, sẽ dùng instance `default` kèm cảnh báo.

### 1. Đồng bộ Wiki vào Graph (Ingest)
Quét toàn bộ thư mục Wiki và đẩy dữ liệu/quan hệ vào Database.
Mặc định tool sẽ lấy Wiki ở `./wiki` và Database ở `~/.hidow/data`.

> **Lần đầu chạy**: Schema được tự động khởi tạo, không cần chạy `hidow init` riêng.

```bash
# Đồng bộ thông minh (chỉ đẩy các file có thay đổi + tự động generate embeddings)
hidow -i nimp --wiki-path /path/to/wiki ingest

# Reload lại toàn bộ (xóa sạch DB cũ, đẩy mới hoàn toàn)
hidow -i nimp --wiki-path /path/to/wiki ingest --full

# Dry-run (Chỉ xem trước số lượng Nodes/Edges được tạo, không ghi vào DB)
hidow -i nimp --wiki-path /path/to/wiki ingest --dry-run
```

### 2. Kiểm tra tính toàn vẹn (Lint & Status)
Xem báo cáo tổng quan về Graph và tìm các lỗi (Orphan nodes, Missing links...).

```bash
# Xem thống kê tổng số Node / Edge hiện có
hidow -i nimp status

# Chạy full Health Check
hidow -i nimp --wiki-path /path/to/wiki lint

# Liệt kê tất cả instances
hidow instance list
```

### 3. Truy vấn Graph (Query)
`hidow` cung cấp **16 preset queries** để phân tích hệ thống:

#### 🔎 Khám phá hệ thống (Discovery)
```bash
# Liệt kê toàn bộ nodes theo loại
hidow -i nimp query list module          # 14 modules
hidow -i nimp query list entity          # 22 entities
hidow -i nimp query list all             # Tất cả nodes

# Tìm kiếm theo từ khóa (tìm trong title + tags)
hidow -i nimp query search premium
hidow -i nimp query search "technical account"

# Xem chi tiết metadata của 1 node (tags, sources, relationship counts, BR counts)
hidow -i nimp query info module:accounting
hidow -i nimp query info entity:voucher

# Đọc toàn bộ nội dung wiki page (markdown body) trực tiếp từ DB
hidow -i nimp query content module:accounting

# Xem TẤT CẢ nodes liên quan (tất cả edge types, cả 2 chiều) trong 1 lệnh
hidow -i nimp query neighbors module:claim
```

#### 📊 Phân tích kiến trúc (Analysis)
```bash
# Phân tích tác động: Module này ảnh hưởng đến những Module / Entity nào?
hidow -i nimp query impact module:technical_account

# Xem toàn bộ Dependency của 1 node
hidow -i nimp query deps entity:voucher

# Business Rules liên quan đến 1 node cụ thể
hidow -i nimp query rules-for module:accounting

# Liệt kê Business Rules theo severity
hidow -i nimp query rules critical
```

#### 🧭 Phân tích nâng cao (Advanced)
```bash
# Tìm đường đi & shared entities giữa 2 node
hidow -i nimp query path module:claim module:accounting

# Tính toán mức độ Coupling: Xếp hạng module phức tạp nhất
hidow -i nimp query coupling

# Entity Usage: Entity nào được Read/Write bởi nhiều Module nhất
hidow -i nimp query entity-usage

# Chạy truy vấn SurrealQL tùy chỉnh
hidow -i nimp query raw "SELECT title FROM module WHERE count(->depends_on) > 5"
```
*(Thêm cờ `--format json` ở cuối nếu bạn muốn output format JSON thay vì Table)*

#### 🧠 Vector Search
```bash
# Tìm modules tương tự (KNN cosine similarity)
hidow -i nimp query similar module:claim

# Semantic search — tìm theo ý nghĩa (hỗ trợ tiếng Việt)
hidow -i nimp query semantic "tính phí bảo hiểm"

# RAG context retrieval — trả về full content cho LLM
hidow -i nimp query ask "XOL calculation" --format json

# Hybrid search (keyword + vector, Reciprocal Rank Fusion)
hidow -i nimp query search claim
```

### 4. Xuất dữ liệu và Vẽ sơ đồ (Export)
```bash
# Xuất toàn bộ Database ra file JSON (bao gồm nodes, edges, business_rules)
hidow -i nimp export --format json > dump.json

# Xuất ra định dạng DOT (Graphviz) để vẽ sơ đồ trực quan (toàn bộ)
hidow -i nimp export --format dot > hidow_graph.dot

# Chỉ xuất Graph của các Modules (bỏ qua Entity/Concept để đỡ rối)
hidow -i nimp export --format dot --node-type module > modules.dot
```

**Mẹo vẽ sơ đồ:** Copy nội dung file `.dot` và dán vào [Edotor.net](https://edotor.net/) hoặc [Graphviz Online](https://dreampuf.github.io/GraphvizOnline/) để xem mô hình đồ thị tương tác.

---

## Query Presets Reference

| # | Preset | Arguments | Mô tả |
|---|--------|-----------|-------|
| 1 | `list` | `<type>` | Liệt kê nodes: module, entity, concept, flow, question, all |
| 2 | `search` | `<keyword>` | Tìm kiếm hybrid (keyword + vector RRF) |
| 3 | `info` | `<type:id>` | Metadata + relationship counts + BR counts |
| 4 | `content` | `<type:id>` | Full markdown body của wiki page |
| 5 | `neighbors` | `<type:id>` | Tất cả nodes liên quan (in/out, all edge types) |
| 6 | `impact` | `<type:id>` | Ai phụ thuộc vào node này? (downstream) |
| 7 | `deps` | `<type:id>` | Node này phụ thuộc ai? (upstream) |
| 8 | `rules` | `[severity]` | Business rules (filter: critical/warning/info) |
| 9 | `rules-for` | `<type:id>` | Business rules ràng buộc 1 node cụ thể |
| 10 | `coupling` | — | Ranking module phức tạp nhất |
| 11 | `entity-usage` | — | Ranking entity được dùng nhiều nhất |
| 12 | `path` | `<from> <to>` | Đường đi + shared entities giữa 2 nodes |
| 13 | `similar` | `<type:id>` | Top-K nodes tương tự (KNN cosine similarity) |
| 14 | `semantic` | `<question>` | Tìm kiếm theo ý nghĩa across all tables |
| 15 | `ask` | `<question>` | RAG context retrieval với full content |
| 16 | `raw` | `"<SurrealQL>"` | Query SurrealQL tự do |



---

## Data Model Architecture

Hệ thống Graph sử dụng các models sau:

**Nodes (Đỉnh):**
- Default types: `module`, `entity`, `concept`, `flow`, `question`, `overview`
- Custom types: Tự động tạo khi ingest (bất kỳ `type` nào trong YAML frontmatter)
- `business_rule` (auto-extracted từ frontmatter)

**Edges (Cạnh chỉ hướng):**
- `depends_on`: Module A phụ thuộc vào Module B
- `produces`: Module/Entity A tạo ra Entity B
- `consumes`: Module/Entity A sử dụng Entity B (Read)
- `contains`: Entity A chứa Entity B
- `part_of`: Entity A là một phần của Entity B
- `implements`: Module implement Concept
- `uses`: Quan hệ sử dụng tổng quát
- `triggers`: Module A kích hoạt Module B
- `affects`: Business Rule ảnh hưởng đến Entities nào

---
*Built with Rust & SurrealDB v2 (Embedded SurrealKV). Vector search powered by fastembed-rs + ONNX Runtime.*
