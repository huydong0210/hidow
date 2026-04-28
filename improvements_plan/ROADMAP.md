# Hidow — Improvement Roadmap

> Last updated: 2026-04-28

## Tổng quan

Tracking toàn bộ các phase cải tiến của `hidow`. Mỗi phase có file chi tiết riêng trong folder `improvements_plan/`.

---

## Progress

| Phase | Tiêu đề | Priority | Status | File |
|-------|---------|----------|--------|------|
| 3 | OpenCode Integration — `hidow code` (Boss-Worker MVP) | `critical` | `planned` | [phase-03](phase-03-opencode-integration.md) |
| 4 | OpenCode Server Mode + Session Management | `medium` | `planned` | [phase-04](phase-04-server-mode.md) |
| 5 | Structured Result Parsing + Custom Tools | `medium` | `planned` | [phase-05](phase-05-result-parsing-custom-tools.md) |

---

## Completed

| Phase | Tiêu đề | Completed | Notes |
|-------|---------|-----------|-------|
| 0 | Rename nimp-graph → hidow + Embedded SurrealKV | 2026-04-22 | Bỏ Docker, auto-init, clean JSON output |
| 1 | LLM Query Enhancements (content, neighbors, export BRs) | 2026-04-22 | [phase-01](phase-01-llm-query-enhancements.md) |
| 2 | Vector Search + Stabilization | 2026-04-23 | [phase-02](phase-02-vector-search.md) — similar, semantic, ask, hybrid RRF. Includes: always-on embeddings, auto-detect ORT, setup.sh, uninstall cmd, overview ingest fix, rules-for entity, bulk queries (list-detail, context) |

---

## Backlog (Chưa lên phase)

Các ý tưởng cải tiến chưa được schedule vào phase cụ thể:

- [ ] **Enrich questions wiki** — Thêm questions từ quá trình phát triển để mở rộng Q&A coverage (hiện chỉ 1 question)
- [ ] **Embedding chunking / model upgrade** — Content hiện chỉ dùng 500 chars đầu, page dài ~10KB bị truncate. Cân nhắc chunk hoặc upgrade model bge-m3
- [ ] **Surrealist GUI support** — Docker compose riêng cho debug/browse graph trực quan
- [ ] **`--format csv` cho query** — Output CSV cho từng query preset
- [ ] **Full-text search trong content** — SurrealDB full-text index trên content field
- [ ] **Interactive TUI** — Terminal UI với ratatui cho graph browsing

---

## Conventions

- Mỗi phase = 1 file: `phase-XX-short-name.md` (VD: `phase-01-content-query.md`)
- Dùng `_TEMPLATE.md` làm template khi tạo phase mới
- Status values: `planned` → `in-progress` → `completed` | `on-hold`
- Priority values: `critical` > `high` > `medium` > `low`
