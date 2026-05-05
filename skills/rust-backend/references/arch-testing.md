# 测试架构（金字塔）

```
E2E (WebDriver/tauri-driver) → Integration (mock_invoke) → Unit (纯 Rust，与 Tauri 无耦合)
```

## dev-dependencies

```toml
# Cargo.toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util", "macros"] }
tauri = { version = "2", features = ["test"] }
mockall = "0.13"
```

## 测试用 AppState 工厂

```rust
impl AppState {
    /// 创建内存数据库的测试实例，与真实环境隔离
    pub fn new_test() -> Self {
        Self {
            db: Arc::new(RwLock::new(DbPool::new_in_memory().unwrap())),
            config: Arc::new(AppConfig::default()),
        }
    }
}
```

## Unit Test（纯逻辑，占 70%+）

```rust
#[tokio::test]
async fn tax_calc_rounds() {
    let svc = TaxService::new(mock_cfg());
    assert!(svc.calculate(&order()).is_ok());
}
```

特点：不依赖 Tauri runtime，纯 Rust 逻辑验证。

## Integration Test（mock Tauri command）

```rust
#[tauri::test]
async fn test_get_data_cmd() {
    let state = AppState::new_test();
    let result = commands::get_data("id1".into(), state.into()).await;
    assert!(result.is_ok());
}
```

特点：使用 `#[tauri::test]` 宏，mock Tauri State 注入。

## E2E Test（WebDriver）

```bash
# 使用 tauri-driver + WebDriver 协议
cargo install tauri-driver
cargo test --test e2e -- --webdriver-port 4444
```

特点：真实窗口交互，验证完整用户流程。数量少、成本高，仅覆盖关键路径。

## 覆盖率目标

- Unit: >= 70%（纯逻辑，快速反馈）
- Integration: >= 20%（command 层验证）
- E2E: <= 10%（关键路径冒烟测试）
