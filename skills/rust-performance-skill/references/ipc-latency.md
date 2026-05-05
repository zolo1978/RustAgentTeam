# IPC 延迟优化 — ClipVault (Tauri v2)

## 目标
单次 IPC round-trip < 5ms。

## 延迟来源分析

| 阶段 | 典型耗时 | 优化空间 |
|------|---------|---------|
| JS invoke 调用 | < 0.1ms | 无需优化 |
| 序列化参数 (serde_json) | 0.1-1ms | 视数据大小 |
| 跨线程调度 | 0.1-0.5ms | spawn_blocking 开销 |
| Rust 命令执行 | 1-50ms | 主要优化点 |
| 序列化返回值 | 0.1-2ms | 视返回大小 |
| JS Promise resolve | < 0.1ms | 无需优化 |

## 测量方法

### 前端测量（精确到单次 IPC）
```typescript
async function measureIpc<T>(
    name: string,
    fn: () => Promise<T>
): Promise<T> {
    const start = performance.now();
    const result = await fn();
    const elapsed = performance.now() - start;
    if (elapsed > 5) {
        console.warn(`IPC slow: ${name} = ${elapsed.toFixed(2)}ms`);
    }
    return result;
}

// 使用
const clips = await measureIpc('get_recent_clips', () =>
    invoke('get_recent_clips', { limit: 50 })
);
```

### Rust 侧测量
```rust
use std::time::Instant;

#[tauri::command]
async fn get_recent_clips(limit: u32, pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<Vec<ClipEntry>, String>
{
    let t0 = Instant::now();
    let result = fetch_clips(&pool, limit).map_err(|e| e.to_string())?;
    log::debug!("get_recent_clips: {}ms, {} entries", t0.elapsed().as_millis(), result.len());
    Ok(result)
}
```

## 序列化选择

### serde_json (默认，推荐大多数场景)
```rust
// Tauri 内置，零配置
// 适合: < 100KB 数据、命令参数、配置
// 性能: 1KB JSON 序列化 ~50us，反序列化 ~80us

#[derive(serde::Serialize, serde::Deserialize)]
struct ClipEntry {
    id: i64,
    content: String,
    timestamp: i64,
}
```

### bincode (大数据场景)
```rust
// 适合: > 100KB 批量数据、导出/导入
// 性能: 比 serde_json 快 2-5x，体积小 30-50%

// Cargo.toml
// bincode = "2" (v2 API)

#[tauri::command]
async fn export_clips(
    pool: State<'_, Pool<SqliteConnectionManager>>,
) -> Result<Vec<u8>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let clips = load_all_clips(&conn)?;
        bincode::serialize(&clips).map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}
```

### 性能对比 (1000 条 ClipEntry)
| 方案 | 序列化 | 反序列化 | 体积 |
|------|--------|---------|------|
| serde_json | ~800us | ~1.2ms | ~150KB |
| bincode v2 | ~200us | ~350us | ~80KB |

## 阻塞优化

### spawn_blocking vs async
```rust
// 错误: SQLite 查询直接在 async runtime 执行（阻塞其他任务）
#[tauri::command]
async fn search(query: String, pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<Vec<ClipEntry>, String>
{
    let conn = pool.get().map_err(|e| e.to_string())?;
    // SQLite 操作是同步的，会阻塞 tokio worker thread!
    search_clips(&conn, &query).map_err(|e| e.to_string())
}

// 正确: spawn_blocking 将同步 IO 放到专用线程池
#[tauri::command]
async fn search(query: String, pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<Vec<ClipEntry>, String>
{
    let conn = pool.get().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        search_clips(&conn, &query).map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}
```

## 大数据传输优化

### 分块传输
```rust
#[tauri::command]
async fn get_clips_batch(
    after_id: i64,
    batch_size: u32,
    pool: State<'_, Pool<SqliteConnectionManager>>,
) -> Result<ClipBatch, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let entries = conn.prepare(
            "SELECT id, content, timestamp FROM clips WHERE id > ?1
             ORDER BY id LIMIT ?2"
        ).map_err(|e| e.to_string())?
        .query_map(rusqlite::params![after_id, batch_size], |row| {
            Ok(ClipEntry { id: row.get(0)?, content: row.get(1)?, timestamp: row.get(2)? })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

        let last_id = entries.last().map(|e| e.id).unwrap_or(after_id);
        Ok(ClipBatch { entries, last_id, has_more: entries.len() == batch_size as usize })
    }).await.map_err(|e| e.to_string())?
}
```

### 文件/图片：不走 IPC
```rust
// 图片: 保存到临时文件，前端通过 asset protocol 读取
#[tauri::command]
async fn get_clip_image(id: i64, pool: State<'_, Pool<SqliteConnectionManager>>)
    -> Result<String, String>  // 返回文件路径
{
    let path = save_image_to_temp(id, &pool)?;
    // 前端: <img src="asset://localhost/{path}" />
    Ok(path)
}
```

## 合并 IPC 调用
```typescript
// 坏: 多次串行 invoke
const clips = await invoke('get_recent_clips', { limit: 50 });
const stats = await invoke('get_stats');
const tags = await invoke('get_all_tags');

// 好: 合并为单次 invoke
const { clips, stats, tags } = await invoke('get_initial_data');

// 或并行（无法合并时）
const [clips, stats, tags] = await Promise.all([
    invoke('get_recent_clips', { limit: 50 }),
    invoke('get_stats'),
    invoke('get_all_tags'),
]);
```

## 验证清单
- [ ] 单次 IPC round-trip < 5ms (前端 performance.now 测量)
- [ ] SQLite 命令全部使用 spawn_blocking
- [ ] > 100KB 数据使用 bincode 或分块
- [ ] 图片通过 asset protocol 传输，不走 IPC 序列化
- [ ] 无串行多次 invoke（用 Promise.all 或合并命令）
