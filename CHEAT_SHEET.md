# Hidow CLI - Cheat Sheet

Công cụ `hidow` cung cấp toàn bộ các lệnh để quản lý và truy vấn đồ thị hệ thống NIMP trên SurrealDB. Dưới đây là danh sách toàn bộ **16 query presets** và các lệnh quản trị, được tổ chức theo mục đích sử dụng.

---

## 🛠 1. Quản trị & Đồng bộ (Admin & Sync)

| Lệnh | Ý nghĩa |
|------|---------|
| `hidow init` | Khởi tạo Schema Database ban đầu (Chỉ chạy 1 lần). |
| `hidow ingest` | Đồng bộ thông minh Wiki vào Graph + generate embeddings. Auto-init schema nếu lần đầu. |
| `hidow ingest --full` | Bỏ qua cache, ghi đè toàn bộ Wiki vào Graph từ đầu. |
| `hidow ingest --dry-run`| Chạy thử Ingest, xem trước số lượng Nodes/Edges sẽ tạo mà không ghi vào DB. |
| `hidow status` | Xem thống kê số lượng Module, Entity, Concept và các liên kết hiện có. |
| `hidow lint` | Chạy bộ kiểm tra sức khỏe Graph (Orphan nodes, Missing links, Sync status). |

---

## 🔎 2. Khám phá hệ thống (Discovery)

Nhóm lệnh dùng khi LLM hoặc Developer chưa biết chính xác cấu trúc và ID của các thành phần trong hệ thống.

| Lệnh | Ý nghĩa | Ví dụ tham số |
|------|---------|---------------|
| `hidow query search <keyword>` | Tìm kiếm hybrid (keyword + vector RRF khi có embeddings). | `search premium` <br> `search voucher` |
| `hidow query list <type>` | Liệt kê toàn bộ nodes theo loại. | `list module`<br>`list entity`<br>`list all` |
| `hidow query info <id>` | Xem toàn bộ Metadata, Tags, Quan hệ, và Số lượng Business Rules của 1 Node. | `info module:accounting`<br>`info entity:voucher` |
| `hidow query content <id>` | Đọc toàn bộ nội dung wiki page (markdown body) trực tiếp từ DB. | `content module:accounting`<br>`content entity:voucher` |
| `hidow query neighbors <id>` | Xem TẤT CẢ nodes liên quan (in/out, all edge types) trong 1 lệnh. | `neighbors module:claim`<br>`neighbors entity:contract` |

---

## 📊 3. Phân tích kiến trúc & Tác động (Analysis)

Nhóm lệnh dùng để đánh giá rủi ro trước khi code hoặc sửa đổi hệ thống.

| Lệnh | Ý nghĩa | Ví dụ tham số |
|------|---------|---------------|
| `hidow query impact <id>` | (Impact Analysis) Thành phần nào sẽ bị "vỡ" nếu tôi sửa node này? | `impact module:claim` |
| `hidow query deps <id>` | Node này đang phụ thuộc và gọi đến những thằng nào? | `deps entity:contract` |
| `hidow query rules-for <id>`| Sửa Node này thì có nguy cơ vi phạm những Business Rules nào? | `rules-for module:accounting` |
| `hidow query rules [severity]`| Liệt kê toàn bộ Business Rules của hệ thống. | `rules`<br>`rules critical` |

---

## 🧭 4. Phân tích đồ thị nâng cao (Advanced)

Nhóm lệnh tính toán tổng thể hoặc truy vấn đường đi phức tạp.

| Lệnh | Ý nghĩa | Ví dụ tham số |
|------|---------|---------------|
| `hidow query path <A> <B>` | Tìm đường đi và các mối liên kết chung giữa Node A và Node B. | `path module:claim module:accounting` |
| `hidow query coupling` | Xếp hạng các Module phức tạp nhất (nhiều liên kết In/Out nhất). | `coupling` |
| `hidow query entity-usage` | Xếp hạng các Entity bị thao tác nhiều nhất (được consume bởi nhiều module nhất).| `entity-usage` |
| `hidow query raw "<sql>"` | Chạy truy vấn SurrealQL tự do. | `raw "SELECT * FROM module"` |

---

## 📤 5. Xuất dữ liệu (Export)

| Lệnh | Ý nghĩa |
|------|---------|
| `hidow export --format json > dump.json`| Xuất toàn bộ (nodes + edges + **business_rules**) ra JSON. |
| `hidow export --format dot > graph.dot` | Xuất Graphviz DOT để dán vào Edotor.net vẽ sơ đồ mạng nhện. |
| `hidow export --format csv` | Xuất Nodes và Edges ra bảng CSV. |

---

## 🧠 6. Vector Search

Nhóm lệnh sử dụng embedding vectors để tìm kiếm theo ngữ nghĩa. Embeddings được tự động generate khi `ingest`.

| Lệnh | Ý nghĩa | Ví dụ tham số |
|------|---------|---------------|
| `hidow query similar <id>` | Tìm Top-5 nodes tương tự nhất (KNN cosine similarity). | `similar module:claim`<br>`similar entity:voucher` |
| `hidow query semantic <text>` | Tìm kiếm theo ý nghĩa (semantic search, hỗ trợ tiếng Việt). | `semantic "tính phí bảo hiểm"`<br>`semantic "premium calculation"` |
| `hidow query ask <question>` | RAG context retrieval — trả full content cho LLM system prompt. | `ask "XOL calculation" --format json`<br>`ask "cách tính retro" --top 5` |

---

## 💡 Mẹo sử dụng (Tips)
1. Thêm cờ `--format json` vào cuối bất kỳ lệnh `query` nào nếu bạn muốn LLM đọc kết quả dưới dạng JSON thay vì dạng bảng.
2. Bạn có thể truyền `--data-dir <PATH>` hoặc `--wiki-path <PATH>` nếu chạy tool ở ngoài thư mục gốc. Mặc định tool lưu database ở `~/.hidow/data` và đọc wiki ở `./wiki`.
3. **Workflow LLM tối ưu**: `search` → `content` (đọc chi tiết) → `impact` + `rules-for` (đánh giá rủi ro).
4. **1 lệnh xem toàn cảnh**: Dùng `neighbors <id>` thay vì gọi riêng `impact` + `deps` + `rules-for`.
5. **RAG pipeline**: `hidow query ask "câu hỏi" --format json` → inject vào LLM system prompt → trả lời chính xác hơn.
6. **So sánh keyword vs semantic**: `search "tính phí"` (hybrid) vs `semantic "tính phí"` (pure vector) — hybrid cho kết quả balanced hơn.
7. **ONNX Runtime**: Binary tự detect `~/.hidow/ort/lib/libonnxruntime.so.*`, không cần set `ORT_DYLIB_PATH` thủ công.
