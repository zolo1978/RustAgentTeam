// lib.rs — Tauri v2 应用入口脚手架
// 使用方法：将此文件复制到 src-tauri/src/lib.rs 并根据业务注册 command 和 plugin

mod commands;
mod error;
mod state;

use state::AppState;

/// 加载应用配置（示例，替换为实际实现）
fn load_config() -> state::AppConfig {
    state::AppConfig::default()
}

/// 应用入口
/// #[cfg_attr(mobile, tauri::mobile_entry_point)] 保证桌面 + 移动端共用入口。
/// 桌面端由 main.rs 调用 run()，移动端由 iOS/Android 的 main 直接调用。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(
            AppState::new(load_config()).expect("state init"),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_data,
            commands::stream_log,
        ])
        .run(tauri::generate_context!())
        .expect("launch failed");
}
