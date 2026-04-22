# Hidow CLI - Cheat Sheet

Công cụ `hidow` cung cấp toàn bộ các lệnh để quản lý và truy vấn đồ thị hệ thống NIMP trên SurrealDB. Dưới đây là danh sách toàn bộ các lệnh được tổ chức theo mục đích sử dụng.

---

## 🛠 1. Quản trị & Đồng bộ (Admin & Sync)

| Lệnh | Ý nghĩa |
|------|---------|
| `hidow init` | Khởi tạo Schema Database ban đầu (Chỉ chạy 1 lần). |
| `hidow ingest` | Đồng bộ thông minh Wiki vào Graph (chỉ đẩy các file có thay đổi). |
| `hidow ingest --full` | Bỏ qua cache, ghi đè toàn bộ Wiki vào Graph từ đầu. |
| `hidow ingest --dry-run`| Chạy thử Ingest, xem trước số lượng Nodes/Edges sẽ tạo mà không ghi vào DB. |
| `hidow status` | Xem thống kê số lượng Module, Entity, Concept và các liên kết hiện có. |
| `hidow lint` | Chạy bộ kiểm tra sức khỏe Graph (Orphan nodes, Missing links, Sync status). |

---

## 🔎 2. Khám phá hệ thống (Discovery)

Nhóm lệnh dùng khi LLM hoặc Developer chưa biết chính xác cấu trúc và ID của các thành phần trong hệ thống.

| Lệnh | Ý nghĩa | Ví dụ tham số |
|------|---------|---------------|
| `hidow query search <keyword>` | Tìm kiếm node theo từ khóa trong tiêu đề và tags (Quan trọng nhất). | `search premium` <br> `search voucher` |
| `hidow query list <type>` | Liệt kê toàn bộ nodes theo loại. | `list module`<br>`list entity`<br>`list all` |
| `hidow query info <id>` | Xem toàn bộ Metadata, Tags, Quan hệ, và Số lượng Business Rules của 1 Node. | `info module:accounting`<br>`info entity:voucher` |

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
| `hidow export --format dot > graph.dot` | Xuất Graphviz DOT để dán vào Edotor.net vẽ sơ đồ mạng nhện. |
| `hidow export --format json > dump.json`| Xuất toàn bộ dữ liệu ra cục JSON để backup hoặc import nơi khác. |
| `hidow export --format csv` | Xuất Nodes và Edges ra bảng CSV. |

---

## 💡 Mẹo sử dụng (Tips)
1. Thêm cờ `--format json` vào cuối bất kỳ lệnh `query` nào nếu bạn muốn LLM đọc kết quả dưới dạng JSON thay vì dạng bảng.
2. Bạn có thể truyền `--db-url <URL>` hoặc `--wiki-path <PATH>` nếu chạy tool ở ngoài thư mục gốc. Mặc định tool kết nối vào `127.0.0.1:8123` và đọc wiki ở `./wiki`.
