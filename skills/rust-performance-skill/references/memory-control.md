# 内存控制 — ClipVault (Tauri v2)

## 目标
idle < 30MB，无内存泄漏。

## 内存分配来源

| 来源 | 典型占用 | 优化方向 |
|------|---------|---------|
| SQLite 缓冲 | 5-10MB | WAL 模式 + 限制 page_cache |
| WebView (前端) | 15-25MB | 懒加载 + 虚拟列表 |
| 剪贴板历史缓存 | 1-50MB | 分页加载 + LRU |
| 连接池 | < 1MB | 控制连接数 |
| Rust 堆分配 | 2-5MB | 减少克隆、用 Cow |

## 测量方法

### 1. RSS 监控（快速检查）
```bash
# macOS RSS (KB)
ps aux | grep ClipVault | awk '{print $6}'

# 持续监控
while true; do
    rss=$(ps aux | grep '[C]lipVault' | awk '{print $6}')
    echo "$(date +%H:%M:%S) RSS: ${rss}KB ($((rss/1024))MB)"
    sleep 5
done
```

### 2. Instruments Allocations（详细分析）
```bash
instruments -t "Allocations" -D mem.trace ./target/release/ClipVault
# 打开 Instruments.app 分析 malloc 调用栈
```

### 3. jemalloc profiling（Linux / 自定义构建）
```toml
# Cargo.toml
[dependencies]
tikv-jemallocator = { version = "0.6", features = ["profiling"] }
```
```rust
// main.rs
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```
```bash
MALLOC_CONF="prof:true,prof_prefix:jeprof" ./target/release/ClipVault &
# 运行一段时间后
kill -USR1 <pid>  # dump heap profile
jeprof --svg ./target/release/ClipVault jeprof.heap > heap.svg
```

## SQLite 连接池配置

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

fn create_pool(db_path: &str) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(4)                  // 桌面应用最多 4 连接
        .min_idle(Some(1))            // 保持 1 热连接
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .expect("pool creation failed")
}

fn init_db(conn: &rusqlite::Connection) {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;       -- 并发读写
         PRAGMA synchronous=NORMAL;     -- 平衡安全和性能
         PRAGMA cache_size=-4000;       -- 4MB 页缓存
         PRAGMA temp_store=MEMORY;      -- 临时表在内存
         PRAGMA mmap_size=0;"           // 桌面应用不用 mmap
    ).expect("PRAGMA failed");
}
```

## 剪贴板历史内存管理

```rust
use std::collections::HashMap;
use std::sync::Mutex;

struct ClipCache {
    pages: HashMap<u32, Vec<ClipEntry>>,  // 分页缓存
    max_pages: usize,                      // 最多缓存页数
    per_page: usize,                       // 每页条数
}

impl ClipCache {
    fn new() -> Self {
        Self {
            pages: HashMap::new(),
            max_pages: 10,  // 缓存 10 页 = 500 条 (50条/页)
            per_page: 50,
        }
    }

    fn get_or_load(&mut self, page: u32, pool: &Pool<SqliteConnectionManager>)
        -> Result<&Vec<ClipEntry>>
    {
        if !self.pages.contains_key(&page) {
            // LRU 淘汰
            if self.pages.len() >= self.max_pages {
                let oldest = *self.pages.keys().next().unwrap();
                self.pages.remove(&oldest);
            }
            let entries = load_page_from_db(pool, page, self.per_page)?;
            self.pages.insert(page, entries);
        }
        Ok(&self.pages[&page])
    }
}

// 全局缓存（Arc<Mutex> 保证线程安全）
lazy_static::lazy_static! {
    static ref CLIP_CACHE: Mutex<ClipCache> = Mutex::new(ClipCache::new());
}
```

## WebView 内存优化

```typescript
// 虚拟列表：只渲染可见区域（react-virtuoso）
import { Virtuoso } from 'react-virtuoso';

function ClipList({ clips }: { clips: ClipEntry[] }) {
    return (
        <Virtuoso
            data={clips}
            itemContent={(index, clip) => <ClipCard key={clip.id} clip={clip} />}
            overscan={200}  // 预渲染 200px 缓冲区
        />
    );
}

// 图片缩略图：后端生成时限制尺寸
// Rust 侧用 image crate
fn generate_thumbnail(data: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;
    let thumb = img.thumbnail(100, 100);  // 最大 100x100
    let mut buf = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
    Ok(buf)
}
```

## 泄漏检测

```rust
// 后台监控线程：每 60s 检查 RSS
fn start_memory_monitor() {
    std::thread::spawn(|| {
        let mut baseline_rss = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(60));
            let rss = get_process_rss();  // 读取 /proc/self/statm 或 macOS sysctl
            if baseline_rss == 0 { baseline_rss = rss; }
            let growth = rss as f64 / baseline_rss as f64;
            if growth > 1.5 {
                log::warn!("memory growth detected: {}MB ({}x baseline)",
                    rss / 1024 / 1024, growth);
            }
        }
    });
}
```

## 验证清单
- [ ] idle RSS < 30MB
- [ ] 1 小时运行后 RSS 增长 < 5MB
- [ ] 连接池 max_size = 4
- [ ] 前端使用虚拟列表
- [ ] 图片缩略图 <= 100x100
- [ ] PRAGMA 配置生效（WAL + cache_size）
