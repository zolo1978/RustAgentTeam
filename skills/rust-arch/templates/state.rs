// state.rs — AppState DI 容器脚手架
// 使用方法：将此文件复制到 src-tauri/src/state.rs 并根据业务扩展字段

use std::sync::{Arc, RwLock};

/// 数据库连接池（示例，替换为实际类型）
struct DbPool;
impl DbPool {
    fn new(_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
    fn new_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
}

/// 应用配置
pub struct AppConfig {
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
        }
    }
}

/// 应用全局状态，通过 Tauri State 注入
pub struct AppState {
    // RwLock<DbPool> — DbPool is a placeholder; replace with rusqlite::Connection, sqlx::Pool, etc.
    pub db: Arc<RwLock<DbPool>>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    /// 生产环境初始化
    pub fn new(cfg: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db: Arc::new(RwLock::new(DbPool::new(&cfg.database_url)?)),
            config: Arc::new(cfg),
        })
    }

    /// 测试环境工厂（内存数据库，隔离真实环境）
    pub fn new_test() -> Self {
        Self {
            db: Arc::new(RwLock::new(DbPool::new_in_memory().expect("in-memory db must not fail"))),
            config: Arc::new(AppConfig::default()),
        }
    }
}
