// commands.rs — Tauri v2 Command 脚手架
// 使用方法：将此文件复制到 src-tauri/src/commands.rs 并根据业务扩展 command

use std::path::PathBuf;
use tauri::State;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::error::AppError;
use crate::state::AppState;

/// 示例 command：获取数据
#[tauri::command]
pub async fn get_data(
    id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    // Example: uncomment to access the database
    // let db = state.db.read().map_err(|e| AppError::Internal(e.to_string()))?;
    // let user = db.query_row("SELECT ...", [&id], |row| ...)?;
    Ok(serde_json::json!({ "id": id, "data": "example" }))
}

/// 流式日志：逐行读取文件并通过 IPC Channel 推送到前端
#[tauri::command]
pub async fn stream_log(
    path: PathBuf,
    ch: tauri::ipc::Channel<String>,
) -> Result<(), AppError> {
    let mut reader = BufReader::new(
        tokio::fs::File::open(&path)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?,
    );
    let mut line = String::new();
    loop {
        line.clear();
        if reader
            .read_line(&mut line)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            == 0
        {
            break;
        }
        ch.send(line.clone())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}
