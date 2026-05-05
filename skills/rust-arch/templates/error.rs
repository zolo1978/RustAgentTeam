// error.rs — 统一错误类型脚手架
// 使用方法：将此文件复制到 src-tauri/src/error.rs 并根据业务扩展 AppError 枚举

// 统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

// Tauri command 要求错误实现 Serialize
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
