mod llm;
mod mcp;

use mcp::client::{McpManager, WizardManager};
use tauri::Manager;

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
        .manage(WizardManager::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            mcp::client::mcp_connect,
            mcp::client::mcp_list_tools,
            mcp::client::mcp_tool_defs,
            mcp::client::mcp_call_tool,
            mcp::client::mcp_status,
            mcp::client::mcp_disconnect,
            mcp::client::mcp_server_auth,
            mcp::client::imap_open_setup_wizard,
            llm::llm_stream_chat
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<WizardManager>().stop();
            }
        });
}
