# Phase 3: OpenCode Integration — `hidow code` (Boss-Worker MVP)

> **Status**: `planned`  
> **Priority**: `critical`  
> **Estimated Effort**: 4-6 hours  
> **Created**: 2026-04-28  
> **Completed**: —  

---

## Mục tiêu

Thêm subcommand `hidow code` cho phép IDE agents (Antigravity, etc.) giao task coding cho hidow. Hidow nhận **Task Spec JSON** chứa đầy đủ context do Boss chuẩn bị, build enriched prompt, và delegate execution cho `opencode run`.

## Bối cảnh / Lý do

Hidow hiện chỉ là **Knowledge Graph reader** — query, search, analyze. Để trở thành coding tool hoàn chỉnh, cần khả năng **execute code changes** thông qua OpenCode CLI.

**Boss-Worker model**: Antigravity (Boss) đã tự research qua `hidow query`, tổng hợp context, rồi giao task cụ thể cho hidow (Worker). Hidow **không tự khám phá context** — chỉ nhận Task Spec → build prompt → gọi opencode → trả result.

```
Antigravity (Boss)           hidow code (Worker)          opencode (Engine)
    │                              │                           │
    │── hidow query (research) ──▶ │                           │
    │◀── context results ──────── │                           │
    │                              │                           │
    │── Task Spec JSON ──────────▶ │                           │
    │                              │── build prompt ──────────▶│
    │                              │◀── code output ───────── │
    │◀── structured result ─────── │                           │
```

---

## Thay đổi chi tiết

### 1. Task Spec Data Models

**File**: `src/commands/task_spec.rs` [NEW]

**Input (Task Spec) models:**

- [ ] Struct `TaskSpec` — top-level container (version, task, context, constraints, output)
- [ ] Struct `TaskInfo` — description + type (implement, modify, fix, test, refactor, review)
- [ ] Struct `TaskContext` — modules, entities, concepts, business_rules, related_flows
- [ ] Struct `ContextNode` — id, title, content, summary
- [ ] Struct `BusinessRuleContext` — id, severity, rule
- [ ] Struct `TaskConstraints` — language, conventions, files_to_read, files_to_modify
- [ ] Struct `TaskOutput` — format, include_diff
- [ ] All fields optional via `#[serde(default)]` — Boss chỉ đưa những gì cần

**Output (Response) models:**

- [ ] Struct `CodeResponse` — structured response trả về cho Boss
- [ ] Struct `TaskResult` — response_text, files_changed
- [ ] Struct `FileChanged` — path, action (created/modified/deleted)
- [ ] Struct `ResponseMetadata` — session_id, tokens, cost, duration_ms
- [ ] Struct `TokenUsage` — input, output, total
- [ ] Struct `CodeError` — message, stderr (khi status = "error")

**Response JSON Schema (trả về cho Boss):**

```json
{
  "version": "1",
  "status": "success",
  "task": {
    "description": "Implement XOL calculation function",
    "type": "implement"
  },
  "result": {
    "response_text": "Created XOL calculation with 3-layer support...",
    "files_changed": [
      {"path": "src/engine/xol.rs", "action": "created"},
      {"path": "src/engine/mod.rs", "action": "modified"}
    ]
  },
  "metadata": {
    "session_id": "ses_22b25acd9ffe...",
    "tokens": {"input": 2524, "output": 86, "total": 2610},
    "cost": 0.0,
    "duration_ms": 6200
  }
}
```

**Error response:**

```json
{
  "version": "1",
  "status": "error",
  "task": {"description": "...", "type": "implement"},
  "error": {
    "message": "opencode exited with code 1",
    "stderr": "Error: no provider configured..."
  },
  "metadata": {
    "session_id": "",
    "tokens": {"input": 0, "output": 0, "total": 0},
    "cost": 0.0,
    "duration_ms": 1200
  }
}
```

**Task Spec JSON Schema**:

```json
{
  "version": "1",
  "task": {
    "description": "Implement XOL calculation function",
    "type": "implement"
  },
  "context": {
    "modules": [
      { "id": "module:retain_engine", "title": "Retain Engine", "content": "..." }
    ],
    "entities": [
      { "id": "entity:contract", "title": "Contract" }
    ],
    "concepts": [
      { "id": "concept:non_proportional_treaty", "content": "..." }
    ],
    "business_rules": [
      { "id": "BR_RE_001", "severity": "critical", "rule": "XOL recovery <= limit" }
    ],
    "related_flows": [
      { "id": "flow:calculation_flow", "summary": "..." }
    ]
  },
  "constraints": {
    "language": "rust",
    "conventions": ["Use Decimal for money", "Add doc comments"],
    "files_to_read": ["src/engine/proportional.rs"],
    "files_to_modify": ["src/engine/xol.rs"]
  },
  "output": {
    "format": "json",
    "include_diff": true
  }
}
```

### 2. Prompt Builder

**File**: `src/commands/prompt_builder.rs` [NEW]

- [ ] Function `build_prompt(spec: &TaskSpec) -> String` — generates enriched markdown prompt
- [ ] Function `build_inline_prompt(description: &str) -> String` — minimal prompt for inline mode
- [ ] Section builders: modules → "### Relevant Modules", BRs → "### Business Rules (MUST comply)", etc.
- [ ] Business rules sorted by severity (critical first)
- [ ] Conventions rendered as numbered list
- [ ] Files split into "Reference" vs "Modify" sections

**Generated prompt template**:

```markdown
## NIMP System Context

### Relevant Modules
- **Retain Engine**: Engine tính toán chính...

### Business Rules (MUST comply)
- [CRITICAL] BR_RE_001: XOL recovery <= limit

### Domain Knowledge
...content from concepts...

### Related Flows
- calculation_flow: Retrocession → Claim → Inward TA...

### Coding Conventions
1. Use Decimal for money
2. Add doc comments

### Files to Reference
- src/engine/proportional.rs

### Files to Modify
- src/engine/xol.rs

## Task
Implement XOL calculation function
```

### 3. Code Command Handler

**File**: `src/commands/code.rs` [NEW]

- [ ] Function `run()` — main entry point
- [ ] Input parsing: `--task-file` path → read file → serde deserialize
- [ ] Input parsing: `--stdin` → read from stdin → serde deserialize
- [ ] Input parsing: inline `task_description` → build minimal TaskSpec
- [ ] Dry-run mode: print enriched prompt to stdout, exit 0
- [ ] OpenCode execution via `std::process::Command`:
  ```rust
  Command::new("opencode")
      .arg("run")
      .arg("--dangerously-skip-permissions")
      .arg("--format").arg("json")
      .arg(&prompt)
      .current_dir(cwd)
      .output()?
  ```
- [ ] Measure execution duration via `std::time::Instant`
- [ ] Call `parse_ndjson_response()` → `CodeResponse`
- [ ] Serialize `CodeResponse` to JSON → stdout
- [ ] Error handling: opencode not found, non-zero exit, invalid task spec

### 4. ndjson Response Parser

**File**: `src/commands/code.rs` (internal function)

OpenCode `--format json` trả **ndjson** (1 JSON per line). Mỗi line là 1 event (verified by testing):

```json
{"type":"step_start","sessionID":"ses_...","part":{...}}
{"type":"tool_use","tool":"write","state":{"input":{"filePath":"...","content":"..."},"output":"Wrote file successfully."}}
{"type":"text","part":{"text":"Created hello.txt..."}}
{"type":"step_finish","reason":"stop","tokens":{"total":2610,"input":2524,"output":86},"cost":0}
```

- [ ] Function `parse_ndjson_response(stdout: &str, stderr: &str, exit_code: i32, duration_ms: u128, task: &TaskInfo) -> CodeResponse`
- [ ] Parse line-by-line, skip malformed lines gracefully
- [ ] Filter `type: "tool_use"` where `tool: "write"` → extract `filePath` → `FileChanged { path, action: "created" | "modified" }`
- [ ] Filter `type: "text"` → concat `.part.text` fields → `response_text`
- [ ] Filter `type: "step_finish"` (last one) → extract `.tokens` + `.cost`
- [ ] Extract `sessionID` from first event that has it
- [ ] Determine `action` (created vs modified): check if `tool_use.state.metadata.exists` is `false` → "created", else "modified"

**Parsing logic sketch:**

```rust
fn parse_ndjson_response(stdout: &str, ...) -> CodeResponse {
    let mut files_changed = Vec::new();
    let mut response_text = String::new();
    let mut tokens = TokenUsage::default();
    let mut session_id = String::new();
    let mut cost = 0.0;

    for line in stdout.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        
        match event.get("type").and_then(|v| v.as_str()) {
            Some("tool_use") => {
                if event.pointer("/part/tool").and_then(|v| v.as_str()) == Some("write") {
                    if let Some(path) = event.pointer("/part/state/input/filePath").and_then(|v| v.as_str()) {
                        let exists = event.pointer("/part/state/metadata/exists")
                            .and_then(|v| v.as_bool()).unwrap_or(true);
                        files_changed.push(FileChanged {
                            path: path.to_string(),
                            action: if exists { "modified" } else { "created" }.to_string(),
                        });
                    }
                }
            }
            Some("text") => {
                if let Some(text) = event.pointer("/part/text").and_then(|v| v.as_str()) {
                    response_text.push_str(text);
                }
            }
            Some("step_finish") => {
                if let Some(t) = event.pointer("/part/tokens") {
                    tokens.total = t.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                    tokens.input = t.get("input").and_then(|v| v.as_u64()).unwrap_or(0);
                    tokens.output = t.get("output").and_then(|v| v.as_u64()).unwrap_or(0);
                }
                cost = event.pointer("/part/cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
            }
            _ => {
                // Extract session_id from any event that has it
                if session_id.is_empty() {
                    if let Some(sid) = event.get("sessionID").and_then(|v| v.as_str()) {
                        session_id = sid.to_string();
                    }
                }
            }
        }
    }
    
    CodeResponse { status: "success", result: TaskResult { response_text, files_changed }, metadata: ... }
}
```

### 5. CLI Registration

**File**: `src/main.rs` [MODIFY]

- [ ] Add `Code` variant to `Commands` enum with args:
  - `task_description: Option<String>` — inline task text
  - `--task-file <PATH>` — Task Spec JSON file
  - `--stdin` — read from stdin
  - `--dry-run` — show prompt only
  - `--format <FMT>` — output format (json/text, default: json)
  - `--opencode-args <ARGS>` — extra flags for opencode run
- [ ] Add match arm in `main()` to call `commands::code::run()`

### 6. Module Registration

**File**: `src/commands/mod.rs` [MODIFY]

- [ ] Add `pub mod code;`
- [ ] Add `pub mod task_spec;`
- [ ] Add `pub mod prompt_builder;`

### 7. No New Dependencies

**File**: `Cargo.toml` [NO CHANGE]

Tất cả dùng crate có sẵn: `serde_json`, `serde`, `anyhow`, `colored`, `clap`, `std::process::Command`, `std::time::Instant`.

---

## Verification

### Automated
```bash
# 1. Build
cargo build

# 2. Help
./target/debug/hidow code --help

# 3. Dry-run with task file
cat > /tmp/test-task.json << 'EOF'
{
  "version": "1",
  "task": {"description": "Create a hello world function", "type": "implement"},
  "context": {
    "modules": [{"id": "module:test", "title": "Test Module", "content": "A test module"}],
    "business_rules": [{"id": "BR_T_001", "severity": "critical", "rule": "Must return string"}]
  },
  "constraints": {"language": "rust"}
}
EOF
./target/debug/hidow code --task-file /tmp/test-task.json --dry-run

# 4. Dry-run with inline text
./target/debug/hidow code --dry-run "Add input validation"

# 5. Actual execution (requires opencode)
mkdir -p /tmp/test-project && cd /tmp/test-project
hidow code --task-file /tmp/test-task.json
```

### Manual
- [ ] Verify `opencode` binary accessible: `which opencode`
- [ ] Test enriched prompt contains all context sections
- [ ] Test response JSON contains `files_changed` with correct paths
- [ ] Test response JSON contains `response_text` (LLM summary)
- [ ] Test response JSON contains `metadata.session_id` and `metadata.tokens`
- [ ] Test error response when opencode binary missing
- [ ] Test error response when invalid task spec JSON
- [ ] Verify Boss (Antigravity) can parse response: `hidow code ... | jq '.result.files_changed'`

---

## Ghi chú / Quyết định thiết kế

1. **Boss-Worker model**: Hidow KHÔNG query Knowledge Graph during `code` command. Antigravity đã research và đưa context vào Task Spec. Tránh duplicate work + context mismatch.
2. **Structured response**: Hidow PHẢI trả structured JSON cho Boss. Boss cần biết: task OK?, files nào đã thay đổi?, LLM nói gì?, tốn bao nhiêu token?. Không trả raw output.
3. **ndjson parsing**: OpenCode `--format json` trả ndjson events, KHÔNG phải single JSON. Cần parse line-by-line, filter by event type. Malformed lines bỏ qua gracefully.
4. **File action detection**: OpenCode `tool_use` event có `metadata.exists` field — `false` = file mới (created), `true` = file có sẵn (modified).
5. **`--dangerously-skip-permissions`**: Full automation mode. OpenCode auto-approve mọi file writes. Phù hợp cho headless/scripted invocation.
6. **Prompt length**: Linux CLI arg limit ~2MB, đủ cho Task Spec 10-20KB context. Nếu vượt, fallback sang `--file` flag (pipe prompt qua temp file).
7. **Working directory**: OpenCode dùng `cwd` của process. `hidow code` chạy trong project dir (cwd mặc định).

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 2 ✅ (Vector Search + bulk queries)
- External: `opencode` CLI v1.4.7+ (đã cài tại `~/.opencode/bin/opencode`)
