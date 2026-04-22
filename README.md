# Hidow

`hidow` là một công cụ CLI hiệu năng cao được viết bằng Rust, dùng để phân tích tài liệu Wiki hệ thống NIMP (kiến trúc Markdown + YAML Frontmatter) và đồng bộ hóa thành một **Knowledge Graph** trên [SurrealDB](https://surrealdb.com/).

Công cụ này giúp developer và BA dễ dàng truy vấn mối quan hệ giữa các Modules, Entities, Concepts, theo dõi Business Rules, phân tích mức độ Coupling (phụ thuộc) và Impact (tác động) khi thay đổi hệ thống.

---

## Tính năng cốt lõi

- ⚡️ **Smart Sync**: Quét toàn bộ wiki directory, sử dụng mã băm `SHA-256` để chỉ cập nhật những file markdown có thay đổi, tối ưu hóa tốc độ Ingest.
- 🕸 **Native Graph Database**: Mapping toàn bộ kiến trúc tài liệu thành Nodes (Module, Entity, Concept...) và Edges (depends_on, consumes, produces...) trên SurrealDB.
- 🔍 **Graph Queries**: Hỗ trợ các query lập trình sẵn để phân tích kiến trúc: tính toán Dependency, Impact Analysis, System Coupling.
- 🛠 **Linter**: Kiểm tra sức khỏe của Wiki, phát hiện Orphan nodes, đảm bảo Graph Database và Wiki luôn đồng bộ (100% in-sync).
- 📤 **Export linh hoạt**: Xuất Graph ra định dạng `JSON`, `CSV` và đặc biệt là `DOT` (tương thích Graphviz) để vẽ sơ đồ trực quan.

---

## Cài đặt

### Yêu cầu hệ thống
- **Rust Toolchain** (v1.75+)
- **Docker & Docker Compose** (Để chạy SurrealDB)

### 1. Khởi động SurrealDB & Surrealist (GUI)
Công cụ đi kèm file `docker-compose.yml` để chạy SurrealDB (lưu trữ in-memory) và giao diện quản trị Surrealist:

```bash
cd hidow
docker compose up -d
```
- **SurrealDB** chạy ở cổng: `localhost:8123`
- **Surrealist GUI** chạy ở cổng: `http://localhost:8124`

### 2. Build & Install CLI
Build project với profile release để tối ưu hiệu năng và copy vào global path:

```bash
cargo build --release
cp target/release/hidow ~/.cargo/bin/
```

Sau khi copy, bạn có thể gọi lệnh `hidow` ở bất kỳ đâu trên terminal.

---

## Hướng dẫn sử dụng

### 1. Khởi tạo Database (Chạy lần đầu)
Tạo các Node schemas, Edge schemas và index cần thiết trên SurrealDB.

```bash
hidow init
```

### 2. Đồng bộ Wiki vào Graph (Ingest)
Quét toàn bộ thư mục Wiki và đẩy dữ liệu/quan hệ vào Database. 
Mặc định tool sẽ lấy Wiki ở `./wiki` và Database ở `127.0.0.1:8123`.

```bash
# Đồng bộ thông minh (chỉ đẩy các file có thay đổi)
hidow --wiki-path /path/to/wiki ingest

# Reload lại toàn bộ (xóa sạch DB cũ, đẩy mới hoàn toàn)
hidow --wiki-path /path/to/wiki ingest --full

# Dry-run (Chỉ xem trước số lượng Nodes/Edges được tạo, không ghi vào DB)
hidow --wiki-path /path/to/wiki ingest --dry-run
```

### 3. Kiểm tra tính toàn vẹn (Lint & Status)
Xem báo cáo tổng quan về Graph và tìm các lỗi (Orphan nodes, Missing links...).

```bash
# Xem thống kê tổng số Node / Edge hiện có
hidow status

# Chạy full Health Check
hidow --wiki-path /path/to/wiki lint
```

### 4. Truy vấn Graph (Query)
`hidow` cung cấp **11 preset queries** để phân tích hệ thống:

#### 🔎 Khám phá hệ thống (Discovery)
```bash
# Liệt kê toàn bộ nodes theo loại
hidow query list module          # 14 modules
hidow query list entity          # 22 entities
hidow query list all             # Tất cả 54 nodes

# Tìm kiếm theo từ khóa (tìm trong title + tags)
hidow query search premium
hidow query search "technical account"

# Xem chi tiết metadata của 1 node (tags, sources, relationship counts, BR counts)
hidow query info module:accounting
hidow query info entity:voucher
```

#### 📊 Phân tích kiến trúc (Analysis)
```bash
# Phân tích tác động: Module này ảnh hưởng đến những Module / Entity nào?
hidow query impact module:technical_account

# Xem toàn bộ Dependency của 1 node
hidow query deps entity:voucher

# Business Rules liên quan đến 1 node cụ thể
hidow query rules-for module:accounting
hidow query rules-for entity:voucher

# Liệt kê Business Rules theo severity
hidow query rules critical
```

#### 🧭 Phân tích nâng cao (Advanced)
```bash
# Tìm đường đi & shared entities giữa 2 node
hidow query path module:claim module:accounting

# Tính toán mức độ Coupling: Xếp hạng module phức tạp nhất
hidow query coupling

# Entity Usage: Entity nào được Read/Write bởi nhiều Module nhất
hidow query entity-usage

# Chạy truy vấn SurrealQL tùy chỉnh
hidow query raw "SELECT title FROM module WHERE count(->depends_on) > 5"
```
*(Thêm cờ `--format json` ở cuối nếu bạn muốn output format JSON thay vì Table)*

### 5. Xuất dữ liệu và Vẽ sơ đồ (Export)
```bash
# Xuất toàn bộ Database ra file JSON
hidow export --format json > dump.json

# Xuất ra định dạng DOT (Graphviz) để vẽ sơ đồ trực quan (toàn bộ)
hidow export --format dot > hidow_graph.dot

# Chỉ xuất Graph của các Modules (bỏ qua Entity/Concept để đỡ rối)
hidow export --format dot --node-type module > modules.dot
```

**Mẹo vẽ sơ đồ:** Copy nội dung file `.dot` và dán vào [Edotor.net](https://edotor.net/) hoặc [Graphviz Online](https://dreampuf.github.io/GraphvizOnline/) để xem mô hình đồ thị tương tác.

---

## Data Model Architecture

Hệ thống Graph sử dụng các models sau:

**Nodes (Đỉnh):**
- `module`
- `entity`
- `concept`
- `flow`
- `question`
- `business_rule`

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
*Built with Rust & SurrealDB v2.*
