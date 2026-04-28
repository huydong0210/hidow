# Phase 4: OpenCode Server Mode + Session Management

> **Status**: `planned`  
> **Priority**: `medium`  
> **Estimated Effort**: 4-6 hours  
> **Created**: 2026-04-28  
> **Completed**: —  

---

## Mục tiêu

Tối ưu `hidow code` bằng cách sử dụng `opencode serve` (persistent HTTP server) thay vì cold-start subprocess mỗi lần gọi. Hỗ trợ session management để chain nhiều tasks liên quan.

## Bối cảnh / Lý do

Phase 6 dùng `opencode run` subprocess — mỗi lần gọi cold-start ~3-5s (boot MCP servers, load config). Khi Antigravity chain nhiều tasks (implement → test → docs), tổng overhead tích lũy đáng kể.

`opencode serve` chạy 1 lần ở background, nhận requests qua HTTP API:
- Không cold-start (server đã warm)
- Giữ session context — `--continue` cùng conversation
- Full REST API: session/message management, SSE event streaming

---

## Thay đổi chi tiết

### 1. Server Lifecycle Management

**File**: `src/commands/code.rs` [MODIFY]

- [ ] Subcommand `hidow code --serve` — start `opencode serve --port 4096` background, lưu PID vào `~/.hidow/opencode.pid`
- [ ] Subcommand `hidow code --stop` — kill server bằng PID
- [ ] Auto-detect: nếu server đang chạy → dùng `--attach`, nếu không → fallback `opencode run`
- [ ] Health check: `GET http://localhost:4096/global/health` trước khi gọi

### 2. HTTP Client Integration

**File**: `Cargo.toml` [MODIFY]

- [ ] Thêm `reqwest` dependency (blocking mode, chỉ cần cho Phase 7)

**File**: `src/commands/code.rs` [MODIFY]

- [ ] `opencode run --attach http://localhost:4096` thay vì bare `opencode run`
- [ ] Hoặc direct HTTP: `POST /session` → `POST /message` → poll `GET /session/:id/status`

### 3. Session Continuity

**File**: `src/commands/code.rs` [MODIFY], `src/commands/task_spec.rs` [MODIFY]

- [ ] Flag `--continue` — tiếp tục session trước (OpenCode nhớ conversation context)
- [ ] Flag `--session <ID>` — attach vào session cụ thể
- [ ] Lưu last session ID vào `~/.hidow/last_session`
- [ ] `CodeResult` thêm field `session_id` để Boss track

### 4. CLI Extensions

**File**: `src/main.rs` [MODIFY]

- [ ] `hidow code --serve` — start persistent server
- [ ] `hidow code --stop` — stop server
- [ ] `hidow code --continue` — continue last session
- [ ] `hidow code --session <ID>` — continue specific session
- [ ] `hidow code --attach <URL>` — connect to external server

---

## Verification

### Automated
```bash
# Start server
hidow code --serve
sleep 3

# Health check
curl http://localhost:4096/global/health

# Run task (should be fast, no cold start)
time hidow code --task-file /tmp/test-task.json

# Continue same session
hidow code --continue "Now add unit tests"

# Stop server
hidow code --stop
```

### Manual
- [ ] Verify server PID tracking works across process restarts
- [ ] Verify session continuity (context preserved between calls)
- [ ] Verify fallback to subprocess when server not running

---

## Ghi chú / Quyết định thiết kế

1. **Opt-in**: Server mode là optimization, không bắt buộc. Phase 6 subprocess vẫn là default.
2. **PID file**: `~/.hidow/opencode.pid` — simple process lifecycle management.
3. **Attach vs HTTP**: Ưu tiên `opencode run --attach` (ít code hơn direct HTTP). Direct HTTP chỉ cần nếu cần fine-grained control.

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 3 ✅ (MVP `hidow code`)
- External: `opencode serve` command (available in opencode v1.4.7)
- New crate: `reqwest` (chỉ nếu dùng direct HTTP, không cần nếu dùng `--attach`)
