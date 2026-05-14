# Phase 7: Multi-Instance Support

> **Status**: `completed`  
> **Priority**: `high`  
> **Estimated Effort**: 2-3 hours  
> **Created**: 2026-05-14  
> **Completed**: 2026-05-14  

---

## Mục tiêu

Cho phép hidow quản lý **nhiều Knowledge Graph instances** trong cùng 1 SurrealDB storage. Mỗi project/docs = 1 instance riêng, isolated hoàn toàn.

## Thiết kế

Cùng data directory (`~/.hidow/data`), mỗi instance = 1 SurrealDB **database** riêng trong namespace `hidow`:

```
~/.hidow/data/
  └── namespace: "hidow"
      ├── db: "nimp"        ← hidow -i nimp
      ├── db: "project_x"   ← hidow -i project_x
      └── db: "default"     ← hidow (không truyền -i)
```

## CLI

```bash
# Tất cả commands yêu cầu -i <instance>
hidow -i nimp query list all
hidow -i nimp ingest --wiki-path /path/to/wiki
hidow -i nimp status

# Không truyền → dùng "default" + warning
hidow query list all
# ⚠️  No instance specified, using 'default'. Use -i <name> to specify.

# Quản lý instances
hidow instance list
# Instances:
#   nimp       (55 nodes, 366 edges)
#   project_x  (12 nodes, 28 edges)
```

---

## Thay đổi chi tiết

### 1. CLI: Global `--instance` flag

**File**: `src/main.rs` [MODIFY]

- [ ] Add `-i / --instance` global arg to `Cli` struct
- [ ] Add `Instance` subcommand with `list` preset
- [ ] Resolve instance: `cli.instance.unwrap_or("default")` + warning
- [ ] Pass `instance` to all command functions

```rust
struct Cli {
    #[arg(short = 'i', long, global = true)]
    instance: Option<String>,
    // ...existing fields...
}

enum Commands {
    // ...existing...
    /// Manage hidow instances
    Instance {
        /// Preset: list
        preset: String,
    },
}
```

### 2. DB connect: Use instance as database name

**File**: `src/db/mod.rs` [MODIFY]

- [ ] Change `connect()` signature: `(data_dir, instance)` instead of `(data_dir, ns, db_name)`
- [ ] Fixed namespace `"hidow"`, database = instance name
- [ ] Update `is_initialized()` if needed

```rust
pub async fn connect(data_dir: &str, instance: &str) -> Result<DbConn> {
    let path = Path::new(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    let db = Surreal::new::<SurrealKv>(data_dir).await?;
    db.use_ns("hidow").use_db(instance).await?;
    Ok(db)
}
```

### 3. Instance list command

**File**: `src/commands/instance.rs` [NEW]

- [ ] Function `run(data_dir, preset)` — handle "list"
- [ ] Connect with ns `"hidow"`, query `INFO FOR NS;` to get all databases
- [ ] For each database: connect, count nodes + edges, display summary

### 4. Update all command signatures

**Files**: All command files [MODIFY]

Every command currently receives `data_dir: &str` and internally calls `db::connect(data_dir, "nimp", "wiki")`. Change to receive `instance: &str` and call `db::connect(data_dir, instance)`.

- [ ] `commands/init.rs` — `run(data_dir, instance)`
- [ ] `commands/ingest.rs` — `run(data_dir, instance, ...)`
- [ ] `commands/query.rs` — `run(data_dir, instance, ...)`
- [ ] `commands/export.rs` — `run(data_dir, instance, ...)`
- [ ] `commands/status.rs` — `run(data_dir, instance)`
- [ ] `commands/lint.rs` — `run(data_dir, instance, ...)`

### 5. Module registration

**File**: `src/commands/mod.rs` [MODIFY]

- [ ] Add `pub mod instance;`

---

## Verification

```bash
# 1. Build
cargo build

# 2. Default instance (with warning)
./target/debug/hidow status
# ⚠️  No instance specified, using 'default'. Use -i <name> to specify.

# 3. Named instance
./target/debug/hidow -i nimp init
./target/debug/hidow -i nimp ingest --wiki-path /path/to/wiki
./target/debug/hidow -i nimp status
./target/debug/hidow -i nimp query list all

# 4. Second instance
./target/debug/hidow -i test init
./target/debug/hidow -i test status    # empty, separate from nimp

# 5. Instance list
./target/debug/hidow instance list
```

---

## Ghi chú

1. **Namespace cố định**: `"hidow"` — không cần user chỉ định.
2. **No config file**: Instance không lưu config. Mỗi lần phải truyền `--wiki-path` khi ingest.
3. **Backward compatible**: Không truyền `-i` → dùng `"default"` + warning. Data cũ trong `ns: "nimp"`, `db: "wiki"` sẽ cần re-ingest vào instance mới.
4. **Instance name validation**: Chỉ cho phép `[a-z0-9_]` — phù hợp SurrealDB table naming.

---

## Dependencies

- Phase(s) phải hoàn thành trước: Phase 6 ✅ (Dynamic Page Types)
- External: None
