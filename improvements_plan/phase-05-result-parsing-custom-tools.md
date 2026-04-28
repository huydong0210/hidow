# Phase 5: Structured Result Parsing + OpenCode Custom Tools

> **Status**: `planned`  
> **Priority**: `medium`  
> **Estimated Effort**: 6-8 hours  
> **Created**: 2026-04-28  
> **Completed**: —  

---

## Mục tiêu

Hai mục tiêu bổ sung:

**8A**: Parse ndjson output từ OpenCode thành structured result (files modified, diffs, token usage, cost) — giúp Boss có report chi tiết.

**8B**: Tạo OpenCode Custom Tools + NIMP Agent — expose hidow queries **bên trong** OpenCode, cho phép LLM tự quyết định khi nào cần query Knowledge Graph.

## Bối cảnh / Lý do

**8A**: Phase 6 trả raw ndjson. Boss (Antigravity) phải tự parse. Structured result giúp Boss dễ chain tasks, track progress, và report cho user.

**8B**: Mở rộng integration theo chiều ngược — thay vì chỉ hidow → opencode, cho phép opencode → hidow. Khi OpenCode đang code và cần thêm context (BR, impact), LLM tự gọi hidow qua custom tools mà không cần Boss chỉ định trước.

---

## Thay đổi chi tiết

### Phase 8A: Structured Result Parsing

#### 1. ndjson Parser

**File**: `src/commands/code.rs` [MODIFY]

- [ ] Function `parse_ndjson(raw: &str) -> Vec<OpenCodeEvent>` — parse từng dòng
- [ ] Enum `OpenCodeEvent` — `StepStart`, `ToolUse`, `Text`, `StepFinish`
- [ ] Extract từ `ToolUse` events: `tool: "write"` → file path + content
- [ ] Extract từ `Text` events: final response text
- [ ] Extract từ `StepFinish` events: token usage + cost

#### 2. Structured CodeResult

**File**: `src/commands/task_spec.rs` [MODIFY]

- [ ] Enhanced `CodeResult`:
  ```rust
  pub struct CodeResult {
      pub status: String,           // "success" | "error"
      pub task: String,
      pub task_type: String,
      pub response_text: String,    // LLM's final message
      pub files_modified: Vec<FileChange>,
      pub tokens: TokenUsage,
      pub session_id: String,
      pub exit_code: i32,
  }

  pub struct FileChange {
      pub path: String,
      pub action: String,  // "created" | "modified" | "deleted"
  }

  pub struct TokenUsage {
      pub total: u64,
      pub input: u64,
      pub output: u64,
      pub cost: f64,
  }
  ```

---

### Phase 8B: OpenCode Custom Tools + NIMP Agent

#### 3. Command `hidow opencode-setup`

**File**: `src/commands/opencode_setup.rs` [NEW]

- [ ] Subcommand `hidow opencode-setup` — auto-generate `.opencode/` config in target project
- [ ] Flag `--project-dir <PATH>` — target project (default: cwd)
- [ ] Flag `--force` — overwrite existing files
- [ ] Generates:
  - `.opencode/tools/hidow_search.ts` — hybrid search tool
  - `.opencode/tools/hidow_impact.ts` — impact analysis tool
  - `.opencode/tools/hidow_rules.ts` — business rules lookup tool
  - `.opencode/tools/hidow_context.ts` — RAG context retrieval tool
  - `.opencode/tools/hidow_info.ts` — node info tool
  - `.opencode/agents/nimp-builder.md` — NIMP specialist agent
  - `.opencode/rules/nimp.md` — NIMP coding conventions

#### 4. Custom Tool Templates

**Embedded in Rust as string constants** (no file dependency):

```typescript
// .opencode/tools/hidow_context.ts
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Query NIMP knowledge graph for context about modules, entities, concepts, and business rules.",
  args: {
    query: tool.schema.string().describe("Natural language question about the NIMP system"),
    top: tool.schema.number().optional().describe("Number of results (default: 5)"),
  },
  async execute(args) {
    const top = args.top ?? 5
    const result = await Bun.$`hidow query ask "${args.query}" --format json --top ${top}`.text()
    return result.trim()
  },
})
```

#### 5. NIMP Agent Template

```markdown
---
description: "NIMP system specialist. Uses Knowledge Graph for domain-aware coding."
mode: subagent
temperature: 0.2
permission:
  edit: allow
  bash: allow
---
You are a NIMP (reinsurance system) specialist developer.

Before writing any code:
1. Use hidow_context to understand the relevant modules and business rules
2. Use hidow_impact to check downstream dependencies
3. Use hidow_rules to verify business rule constraints

Always follow NIMP coding conventions and respect existing business rules.
```

---

## Verification

### Phase 8A
```bash
# Test with real task, verify structured output
hidow code --task-file /tmp/test-task.json --format json | python3 -c "
import sys, json
result = json.load(sys.stdin)
print(f'Status: {result[\"status\"]}')
print(f'Files: {len(result.get(\"files_modified\", []))}')
print(f'Tokens: {result.get(\"tokens\", {}).get(\"total\", \"?\")}')
"
```

### Phase 8B
```bash
# Generate config
hidow opencode-setup --project-dir /tmp/test-project

# Verify files
ls -la /tmp/test-project/.opencode/tools/
ls -la /tmp/test-project/.opencode/agents/

# Test via opencode
cd /tmp/test-project
opencode run --agent nimp-builder "What modules handle claim processing?"
```

---

## Ghi chú / Quyết định thiết kế

1. **8A vs 8B independent**: Có thể implement song song hoặc chọn 1 trước.
2. **Custom tools in TypeScript**: OpenCode yêu cầu tool definitions bằng TS/JS. Hidow generates chúng nhưng bản thân hidow vẫn là Rust.
3. **Templates embedded in binary**: Không dependency on external template files. Rust `include_str!` hoặc string constants.

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 3 ✅ (MVP `hidow code`)
- 8B external: OpenCode custom tools runtime (TypeScript/Bun — bundled with opencode)
