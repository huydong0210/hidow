# Hidow — Improvement Roadmap

> Last updated: 2026-04-22

## Tổng quan

Tracking toàn bộ các phase cải tiến của `hidow`. Mỗi phase có file chi tiết riêng trong folder `improvements_plan/`.

---

## Progress

| Phase | Tiêu đề | Priority | Status | File |
|-------|---------|----------|--------|------|
| 1 | LLM Query Enhancements (content, neighbors, export BRs) | `high` | `completed` | [phase-01](phase-01-llm-query-enhancements.md) |

---

## Completed

| Phase | Tiêu đề | Completed | Notes |
|-------|---------|-----------|-------|
| 0 | Rename nimp-graph → hidow + Embedded SurrealKV | 2026-04-22 | Bỏ Docker, auto-init, clean JSON output |

---

## Backlog (Chưa lên phase)

Các ý tưởng cải tiến chưa được schedule vào phase cụ thể:

- [ ] **Surrealist GUI support** — Docker compose riêng cho debug/browse graph trực quan
- [ ] **`--format csv` cho query** — Output CSV cho từng query preset
- [ ] **Full-text search trong content** — Tìm kiếm trong body markdown, không chỉ title/tags

---

## Conventions

- Mỗi phase = 1 file: `phase-XX-short-name.md` (VD: `phase-01-content-query.md`)
- Dùng `_TEMPLATE.md` làm template khi tạo phase mới
- Status values: `planned` → `in-progress` → `completed` | `on-hold`
- Priority values: `critical` > `high` > `medium` > `low`
