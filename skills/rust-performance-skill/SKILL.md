---
name: rust-performance
description: 'Tauri v2 桌面应用性能优化 -- 冷启动、内存控制、搜索性能、IPC 延迟、基准测试。决策树格式——给结论，不列菜单。'
tools: ['Read', 'Glob', 'Grep', 'Bash']
---

# Rust Performance (Tauri v2 / ClipVault)

## Quick Start
1. 冷启动慢？测四个阶段耗时：Runtime init -> Plugin load -> Window create -> First paint
2. 内存超标？SQLite 连接池 + 分页加载，idle < 30MB
3. 搜索慢？FTS5 索引 + EXPLAIN QUERY PLAN，10K 条 < 30ms
4. IPC 卡顿？serde_json 小数据 / bincode 大数据，round-trip < 5ms
5. 要量化？用 `templates/benchmark-template.rs`，criterion 跑分

## 适用范围
**适用于：** ClipVault（Tauri v2 剪贴板管理器）的性能优化——冷启动、内存、搜索、IPC、基准测试。
**不适用于：** 通用 Rust 性能调优（见 `rust-core`）、架构设计（见 `rust-arch`）、安全审计（见 `rust-security`）。
**量化目标：** 冷启动 < 300ms (macOS M1)，idle < 30MB，搜索 < 30ms (10K)，IPC < 5ms。

## 1. 冷启动优化

**目标：** 冷启动 < 300ms（macOS M1）。

**触发信号：**
- 冷启动超过 500ms
- 二进制体积 > 10MB
- 用户反馈"打开慢"

**启动链分析：**
```
Runtime init (~20ms) -> Plugin load (~50-150ms) -> Window create (~50ms) -> First paint (~30ms)
```

**决策树：**
```
启动阶段分析
├─ 测量方法: instruments timeprofiler 或内埋时间戳
├─ Runtime init > 30ms?
│   └─ 检查 Cargo.toml 依赖树，裁剪不必要 feature
├─ Plugin load > 100ms?
│   └─ 非核心插件延迟初始化（clipboard-listener 推迟到窗口就绪后）
├─ Window create > 80ms?
│   └─ 检查前端首屏 JS bundle 体积，lazy-load 非首屏组件
└─ First paint > 50ms?
    └─ 检查 CSS/字体加载，骨架屏占位

二进制体积优化
├─ Cargo.toml [profile.release]
│   ├─ strip = true
│   ├─ lto = true (thin LTO 平衡编译时间和体积)
│   ├─ codegen-units = 1 (最优化，编译慢)
│   └─ opt-level = "z" (体积优先) 或 "s" (平衡)
└─ 检查依赖: cargo bloat --crates
```

**怎么做：**
```rust
// src/main.rs — 延迟初始化模式
fn main() {
    let start = std::time::Instant::now();

    // 阶段 1: 核心初始化（必须）
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_cmds])
        .setup(|app| {
            // 阶段 2: 窗口创建（必须）
            let window = tauri::WindowBuilder::new(app, "main")
                .title("ClipVault")
                .inner_size(400.0, 600.0)
                .build()?;
            log::info!("window ready: {:?}", start.elapsed());

            // 阶段 3: 延迟加载（非阻塞）
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                init_clipboard_watcher(&handle);
                init_search_index(&handle);
                log::info!("deferred init done: {:?}", start.elapsed());
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
```

```toml
# Cargo.toml — release 优化
[profile.release]
strip = true
lto = "thin"
codegen-units = 1
opt-level = "s"
panic = "abort"      # 减小体积，不用 catch_unwind
```

**测量命令：**
```bash
# 冷启动计时
time /path/to/clipvault --quit-after-startup
# macOS Instruments
instruments -t "Time Profiler" -D startup.trace ./target/release/ClipVault
# 二进制体积分析
cargo bloat --crates --release
```

-> 完整参考: [references/cold-start-optimization.md]

## 2. 内存控制

**目标：** idle < 30MB。

**触发信号：**
- 内存占用 > 50MB (idle)
- 内存持续增长（泄漏）
- 用户反馈"占内存"

**决策树：**
```
内存来源分析
├─ 工具: Activity Monitor / instruments malloc_history / jemalloc profiling
├─ SQLite 连接池
│   ├─ 使用 r2d2-sqlite，max_size = 4（桌面应用 4 足够）
│   ├─ WAL 模式: PRAGMA journal_mode=WAL
│   └─ 定期 VACUUM: 不自动执行，用户触发或低频后台
├─ 剪贴板历史内存
│   ├─ < 1000 条: 全量缓存（内存占用可控）
│   ├─ 1000-10000 条: 分页加载，每页 50 条
│   └─ > 10000 条: 分页 + LRU 淘汰非活跃页
├─ WebView 内存
│   ├─ 限制: webview.set_size 合理值
│   ├─ 图片缩略图: 生成时压缩到 100x100，不缓存原始图
│   └─ 定期: window.eval("if(window.gc) window.gc()")
└─ 检测泄漏
    ├─ cargo leak-detection 或手动 drop 验证
    └─ 监控: 后台线程每 60s 记录 rss，持续增长则告警
```

**怎么做：**
```rust
// SQLite 连接池配置
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

fn create_pool(db_path: &str) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(4)              // 桌面应用 4 连接足够
        .min_idle(Some(1))        // 保持 1 个热连接
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("failed to create pool");
    // WAL 模式提升并发读性能
    let conn = pool.get().unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .unwrap();
    pool
}

// 分页加载
fn load_history(pool: &Pool<SqliteConnectionManager>, page: u32, per_page: u32)
    -> Result<Vec<ClipEntry>>
{
    let offset = page * per_page;
    let conn = pool.get()?;
    let entries = conn.prepare(
        "SELECT id, content, timestamp FROM clips ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
    )?
    .query_map(rusqlite::params![per_page, offset], |row| {
        Ok(ClipEntry { id: row.get(0)?, content: row.get(1)?, timestamp: row.get(2)? })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}
```

**测量命令：**
```bash
# macOS 内存分析
ps aux | grep ClipVault | awk '{print $6}'  # RSS in KB
# Instruments 内存跟踪
instruments -t "Allocations" -D mem.trace ./target/release/ClipVault
# jemalloc profiling (需 feature = "profiling")
MALLOC_CONF="prof:true,prof_prefix:jeprof" ./target/release/ClipVault
jeprof --svg ./target/release/ClipVault jeprof.heap
```

-> 完整参考: [references/memory-control.md]

## 3. 搜索性能

**目标：** 10K 条记录搜索 < 30ms。

**触发信号：**
- 搜索延迟 > 50ms (10K 条)
- 搜索结果无 FTS5 索引
- LIKE '%keyword%' 全表扫描

**决策树：**
```
数据量评估
├─ < 1K 条: SQLite LIKE 足够（简单，无需 FTS）
├─ 1K-100K 条: FTS5 索引（推荐默认方案）
└─ > 100K 条: FTS5 + 分页 + 结果缓存

FTS5 配置
├─ 中文内容? unicode61 tokenizer + remove_diacritics 2
├─ 需要前缀匹配? tokenize="unicode61 tokenchars _"
└─ 内容类型: text (正文) + tag (标签) 分列索引

查询优化
├─ EXPLAIN QUERY PLAN 检查是否命中 fts index
├─ 分页策略
│   ├─ < 10 页: LIMIT/OFFSET（简单）
│   └─ > 10 页: cursor-based (WHERE rowid < last_id LIMIT N)
└─ 避免在 FTS 查询中排序，用 rank() 代替
```

**怎么做：**
```sql
-- 创建 FTS5 索引
CREATE VIRTUAL TABLE clips_fts USING fts5(
    content,
    tags,
    content='clips',
    content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);

-- 同步触发器（content table 模式）
CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
    INSERT INTO clips_fts(rowid, content, tags) VALUES (new.id, new.content, new.tags);
END;
CREATE TRIGGER clips_ad AFTER DELETE ON clips BEGIN
    INSERT INTO clips_fts(clips_fts, rowid, content, tags) VALUES('delete', old.id, old.content, old.tags);
END;
CREATE TRIGGER clips_au AFTER UPDATE ON clips BEGIN
    INSERT INTO clips_fts(clips_fts, rowid, content, tags) VALUES('delete', old.id, old.content, old.tags);
    INSERT INTO clips_fts(rowid, content, tags) VALUES (new.id, new.content, new.tags);
END;

-- 搜索查询（带分页）
SELECT c.* FROM clips c
JOIN clips_fts f ON c.id = f.rowid
WHERE clips_fts MATCH ?1
ORDER BY f.rank
LIMIT 50 OFFSET ?2;
```

```rust
// Rust 搜索封装
fn search(pool: &Pool<SqliteConnectionManager>, query: &str, page: u32)
    -> Result<Vec<ClipEntry>>
{
    let conn = pool.get()?;
    let offset = page * 50;
    // 安全转义 FTS5 特殊字符
    let safe_query = query
        .replace('"', "\"\"")
        .replace("*", "")
        .replace(":", " ");
    let fts_query = format!("\"{}\"", safe_query);

    let mut stmt = conn.prepare(
        "SELECT c.id, c.content, c.timestamp FROM clips c \
         JOIN clips_fts f ON c.id = f.rowid \
         WHERE clips_fts MATCH ?1 \
         ORDER BY f.rank LIMIT 50 OFFSET ?2"
    )?;
    let entries = stmt.query_map(rusqlite::params![fts_query, offset], |row| {
        Ok(ClipEntry { id: row.get(0)?, content: row.get(1)?, timestamp: row.get(2)? })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}
```

**测量命令：**
```bash
# 查询计划分析
sqlite3 clipvault.db "EXPLAIN QUERY PLAN SELECT * FROM clips_fts WHERE clips_fts MATCH 'test'"
# 基准测试
cargo bench -- search
```

-> 完整参考: [references/search-performance.md]

## 4. IPC 延迟

**目标：** 单次 IPC round-trip < 5ms。

**触发信号：**
- IPC round-trip > 10ms
- 大数据传输卡顿 UI
- 前端 invoke 后白屏/等待

**决策树：**
```
数据大小评估
├─ < 1KB (命令调用): serde_json，开销可忽略
├─ 1KB-100KB (列表/搜索结果): serde_json，考虑分块
├─ 100KB-1MB (批量数据): bincode 序列化，分块传输
└─ > 1MB (文件/图片): 直接文件路径传递，不走 IPC 序列化

序列化选择
├─ 默认: serde_json (Tauri 内置，零额外依赖)
├─ 性能敏感: bincode (2-5x 更快，体积小 30-50%)
└─ 注意: bincode 需要 Tauri 自定义 command 返回 bytes，前端用 ArrayBuffer 接收

阻塞分析
├─ 命令内有同步 IO (SQLite 查询)?
│   └─ 用 #[tauri::command] + spawn_blocking 或 async runtime
├─ 命令内有 CPU 密集计算?
│   └─ spawn_blocking，不要阻塞 async runtime
└─ 多次串行 invoke?
    └─ 合并为单次 invoke，减少 IPC 往返
```

**怎么做：**
```rust
// 方案 1: 标准 serde_json 命令（小数据）
#[tauri::command]
async fn get_recent_clips(limit: u32, pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<Vec<ClipEntry>, String>
{
    let conn = pool.get().map_err(|e| e.to_string())?;
    // SQLite 查询用 spawn_blocking 防止阻塞 async runtime
    tokio::task::spawn_blocking(move || {
        let mut stmt = conn.prepare(
            "SELECT id, content, timestamp FROM clips ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;
        stmt.query_map(rusqlite::params![limit], |row| {
            Ok(ClipEntry { id: row.get(0)?, content: row.get(1)?, timestamp: row.get(2)? })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}

// 方案 2: bincode 大数据传输
#[tauri::command]
async fn export_all_clips(pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<Vec<u8>, String>
{
    let conn = pool.get().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let clips = load_all_clips(&conn)?;  // Vec<ClipEntry>
        bincode::serialize(&clips).map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}
```

```typescript
// 前端调用 — bincode 接收
const { invoke } = window.__TAURI__.core;
const bytes: Uint8Array = await invoke('export_all_clips');
// 用 msgpack 或自定义解析处理
```

**测量命令：**
```bash
# IPC 延迟测量（前端 console）
console.time('ipc:get_recent_clips');
await invoke('get_recent_clips', { limit: 50 });
console.timeEnd('ipc:get_recent_clips');
```

-> 完整参考: [references/ipc-latency.md]

## 5. 基准测试

**目标：** 所有性能优化必须有 criterion 基准数据支撑。

**触发信号：**
- 需要量化性能指标
- 对比优化前后效果
- CI 性能回归检测

**决策树：**
```
测试目标
├─ SQLite 查询性能 → Bencher::iter + 真实数据库 fixture
├─ 序列化开销 → criterion black_box + 序列化/反序列化 cycle
├─ 字符串处理 → 多输入规模 (100B / 1KB / 10KB) 参数化
└─ 全链路端到端 → 集成测试 + time::Instant

基准类型
├─ 微基准 (函数级) → criterion::black_box, 几 us ~ 几 ms
├─ 宏基准 (模块级) → criterion + setup/teardown, 几 ms ~ 几 s
└─ 端到端 → cargo test + 断言耗时上限

报告格式
├─ 本地开发: cargo bench → target/criterion/report/index.html
├─ CI 对比: criterion → JSON output + 自定义脚本对比基线
└─ PR review: 贴 criterion 的 median / mean / change%
```

**怎么做：**
```bash
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "clipvault_bench"
harness = false
path = "benches/clipvault_bench.rs"
```

```bash
# 运行基准
cargo bench
# 打开报告
open target/criterion/report/index.html
# 对比基线
cargo bench -- --save-baseline before_optimization
# ... 做优化 ...
cargo bench -- --baseline before_optimization
```

-> 使用模板: [templates/benchmark-template.rs]

## 6. 性能回归检测

**目标：** PR 不引入性能退化。

**触发信号：**
- PR 涉及 SQLite 查询、IPC command、序列化逻辑
- CI 构建触发性能测试阶段
- 手动性能 review 需求

**决策树：**
```
变更范围
├─ 仅 UI/CSS? → 跳过性能测试
├─ 涉及 Rust 命令/查询?
│   ├─ 新增命令 → 补充基准测试
│   ├─ 修改查询 → 对比现有基准
│   └─ 修改序列化 → 对比序列化基准
└─ 涉及依赖升级?
    ├─ serde/serde_json 版本 → 跑序列化基准
    ├─ rusqlite 版本 → 跑查询基准
    └─ tauri 版本 → 跑全量基准

影响评估
├─ criterion change% > 10%: 阻断合并
├─ criterion change% 5-10%: 需 comment 说明原因
└─ criterion change% < 5%: 正常波动

测试范围
├─ CI 流程: cargo bench --save-baseline main → PR 分支 cargo bench --baseline main
├─ 报告: 自动生成 criterion HTML + 贴 PR comment
└─ 失败: > 10% 回归自动标记 PR
```

**怎么做：**
```yaml
# .github/workflows/bench.yml
name: Performance
on: [pull_request]
jobs:
  bench:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo bench --save-baseline pr_branch
      - run: |
          git checkout main
          cargo bench --save-baseline main_branch
          git checkout -
          cargo bench --baseline main_branch 2>&1 | tee bench_results.txt
      - uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const results = fs.readFileSync('bench_results.txt', 'utf8');
            // Post results as PR comment
            github.rest.issues.createComment({
              ...context.repo,
              issue_number: context.issue.number,
              body: '## Benchmark Results\n```\n' + results + '\n```'
            });
```

## References
| Topic | File |
|-------|------|
| 冷启动优化 | [references/cold-start-optimization.md](references/cold-start-optimization.md) |
| 内存控制 | [references/memory-control.md](references/memory-control.md) |
| 搜索性能 (FTS5) | [references/search-performance.md](references/search-performance.md) |
| IPC 延迟 | [references/ipc-latency.md](references/ipc-latency.md) |
| 基准测试模板 | [templates/benchmark-template.rs](templates/benchmark-template.rs) |

## 工具命令速查

```bash
# 冷启动
time ./target/release/ClipVault --quit-after-startup
instruments -t "Time Profiler" -D startup.trace ./target/release/ClipVault

# 内存
ps aux | grep ClipVault | awk '{print $6}'
instruments -t "Allocations" -D mem.trace ./target/release/ClipVault
cargo bloat --crates --release

# 搜索
sqlite3 clipvault.db "EXPLAIN QUERY PLAN SELECT * FROM clips_fts WHERE clips_fts MATCH 'test'"

# IPC (前端 console)
console.time('ipc'); await invoke('cmd'); console.timeEnd('ipc');

# 基准测试
cargo bench
cargo bench -- --save-baseline before && cargo bench -- --baseline before
open target/criterion/report/index.html
```
