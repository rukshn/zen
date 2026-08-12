mod llm;
mod mcp;
mod toolsearch;

use mcp::client::{McpManager, WizardManager};
use std::sync::Mutex;
use toolsearch::{AppState, Bm25Index, ToolCatalog};
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
        .manage(Mutex::new(None::<AppState>))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = AppState::load(&app_handle).unwrap_or_else(|_| AppState {
                catalog: ToolCatalog { tools: Vec::new() },
                index: Bm25Index {
                    docs: Vec::new(),
                    doc_len: Vec::new(),
                    avg_doc_len: 0.0,
                    doc_freq: std::collections::HashMap::new(),
                    n_docs: 0,
                    k1: 1.5,
                    b: 0.75,
                },
            });
            app.state::<Mutex<Option<AppState>>>().lock().unwrap().replace(state);
            Ok(())
        })
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
            llm::llm_stream_chat,
            toolsearch::search_tools,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<WizardManager>().stop();
            }
        });
}
