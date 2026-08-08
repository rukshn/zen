mod mcp;

use mcp::client::McpManager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .manage(McpManager::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            mcp::client::mcp_connect,
            mcp::client::mcp_list_tools,
            mcp::client::mcp_call_tool,
            mcp::client::mcp_status,
            mcp::client::mcp_disconnect,
            mcp::client::mcp_server_auth
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
