# 冷启动优化 — ClipVault (Tauri v2)

## 目标
冷启动 < 300ms (macOS M1)，含窗口首帧绘制。

## 启动链分解

| 阶段 | 耗时目标 | 主要开销 |
|------|---------|---------|
| Runtime init | < 30ms | 依赖加载、全局初始化 |
| Plugin load | < 100ms | Tauri 插件注册、权限检查 |
| Window create | < 80ms | WebView 创建、HTML 加载 |
| First paint | < 50ms | JS 执行、CSS 渲染 |
| **总计** | **< 300ms** | |

## 测量方法

### 1. 内埋时间戳（推荐，精确到阶段）
```rust
fn main() {
    let t0 = std::time::Instant::now();
    let app = tauri::Builder::default();
    let t1 = t0.elapsed(); // Runtime init

    app.setup(move |_app| {
        let t2 = t2.elapsed(); // Plugin load
        let window = create_window(app)?;
        let t3 = t3.elapsed(); // Window create
        log::info!("startup: runtime={}ms plugin={}ms window={}ms",
            t1.as_millis(), (t2-t1).as_millis(), (t3-t2).as_millis());
        Ok(())
    });
}
```

### 2. macOS Instruments（系统级分析）
```bash
# Time Profiler — CPU 热点
instruments -t "Time Profiler" -D startup.trace \
    ./target/release/ClipVault
# 用 Instruments.app 打开 startup.trace 分析

# 启动时间（终端快速测量）
time ./target/release/ClipVault --quit-after-startup 2>/dev/null
```

### 3. 二进制体积分析
```bash
cargo install cargo-bloat
cargo bloat --crates --release
# 关注 > 5% 的 crate，评估是否可以 feature gate
```

## 优化策略

### 延迟加载（最大收益）
```rust
tauri::Builder::default()
    .setup(|app| {
        let handle = app.handle().clone();
        // 核心：只创建窗口 + 注册核心命令
        create_main_window(app)?;

        // 非核心：defer 到后台线程
        std::thread::spawn(move || {
            init_clipboard_watcher(&handle);  // 剪贴板监听
            init_search_index(&handle);       // FTS5 索引预热
            init_auto_update(&handle);        // 自动更新检查
        });
        Ok(())
    })
```

**哪些必须同步：** 窗口创建、数据库打开、核心命令注册。
**哪些可以延迟：** 剪贴板监听、搜索索引预热、自动更新、统计分析、快捷键注册。

### 二进制体积优化
```toml
[profile.release]
strip = true           # 去符号表，减小 30-40%
lto = "thin"           # 跨 crate 优化，thin 比 full 编译快
codegen-units = 1      # 最优代码生成，编译慢但运行快
opt-level = "s"        # 体积优化 (z 更小但可能更慢)
panic = "abort"        # 不需要 unwind，减小 5-10%
```

### 依赖裁剪
```bash
# 分析每个 crate 的编译体积
cargo bloat --crates --release -n 20

# 常见裁剪点：
# - serde_json: features = ["std"] 去掉后减小 ~100KB
# - tokio: 只启用需要的 features，不用 "full"
# - reqwest: 默认开启很多 TLS backend，选 rustls 或 native-tls 其中之一
```

### 前端优化
```typescript
// 骨架屏：首帧立即渲染占位
// main.tsx
function App() {
    return (
        <Suspense fallback={<Skeleton />}>
            <LazyHistory />  {/* lazy(() => import('./History')) */}
        </Suspense>
    );
}

// Vite 配置：代码分割
// vite.config.ts
export default defineConfig({
    build: {
        rollupOptions: {
            output: { manualChunks: { vendor: ['react', 'react-dom'] } }
        }
    }
});
```

## 验证清单
- [ ] `time` 测量 < 300ms
- [ ] cargo bloat 无 > 15% 的单一 crate
- [ ] 延迟功能在窗口就绪后 200ms 内完成
- [ ] release 构建已启用 strip + lto
- [ ] 首帧无空白闪烁（骨架屏）
