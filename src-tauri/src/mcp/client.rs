use std::collections::HashMap;

use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{serde_json::Value, RoleClient, ServiceExt};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tokio::process::Command;
use tokio::sync::Mutex;

use super::servers;

type McpClient = RunningService<RoleClient, ()>;

pub struct McpManager {
	clients: Mutex<HashMap<String, McpClient>>,
}

impl Default for McpManager {
	fn default() -> Self {
		Self {
			clients: Mutex::new(HashMap::new()),
		}
	}
}

fn resolve_credentials(
	app: &AppHandle,
	client_id: &str,
	client_secret: &str,
) -> Result<String, String> {
	let app_data_dir = app
		.path()
		.app_data_dir()
		.map_err(|e| format!("failed to resolve app data dir: {e}"))?;
	let path = servers::ensure_credentials(&app_data_dir, client_id, client_secret)?;
	Ok(path.to_string_lossy().to_string())
}

#[derive(Serialize)]
pub struct McpToolInfo {
	pub name: String,
	pub description: Option<String>,
}

#[derive(Serialize)]
pub struct McpToolResult {
	pub is_error: bool,
	pub text: String,
	pub structured: Option<Value>,
}

fn to_tool_info(name: String, description: Option<String>) -> McpToolInfo {
	McpToolInfo { name, description }
}

#[tauri::command]
pub async fn mcp_connect(
	app: AppHandle,
	state: State<'_, McpManager>,
	server: String,
	client_id: String,
	client_secret: String,
) -> Result<Vec<McpToolInfo>, String> {
	let mut guard = state.clients.lock().await;

	if let Some(client) = guard.get(&server) {
		let tools = client.list_all_tools().await.map_err(|e| e.to_string())?;
		return Ok(tools
			.into_iter()
			.map(|t| to_tool_info(t.name.to_string(), t.description.map(|d| d.to_string())))
			.collect());
	}

	let credentials_path = resolve_credentials(&app, &client_id, &client_secret)?;
	let cfg = servers::resolve(&server, Some(&credentials_path))?;

	let mut command = Command::new(&cfg.command);
	command.args(&cfg.args);
	for (key, value) in &cfg.env {
		command.env(key, value);
	}
	let transport = TokioChildProcess::new(command).map_err(|e| e.to_string())?;
	let client = ().serve(transport).await.map_err(|e| e.to_string())?;

	let tools = client.list_all_tools().await.map_err(|e| e.to_string())?;
	let info: Vec<McpToolInfo> = tools
		.into_iter()
		.map(|t| to_tool_info(t.name.to_string(), t.description.map(|d| d.to_string())))
		.collect();

	guard.insert(server, client);
	Ok(info)
}

#[tauri::command]
pub async fn mcp_list_tools(
	state: State<'_, McpManager>,
	server: String,
) -> Result<Vec<McpToolInfo>, String> {
	let guard = state.clients.lock().await;
	let client = guard
		.get(&server)
		.ok_or_else(|| format!("MCP server '{server}' is not connected"))?;
	let result = client.list_tools(None).await.map_err(|e| e.to_string())?;
	Ok(result
		.tools
		.into_iter()
		.map(|t| to_tool_info(t.name.to_string(), t.description.map(|d| d.to_string())))
		.collect())
}

#[tauri::command]
pub async fn mcp_call_tool(
	state: State<'_, McpManager>,
	server: String,
	tool: String,
	arguments: Value,
) -> Result<McpToolResult, String> {
	let guard = state.clients.lock().await;
	let client = guard
		.get(&server)
		.ok_or_else(|| format!("MCP server '{server}' is not connected"))?;

	let params = CallToolRequestParams::new(tool).with_arguments(
		arguments
			.as_object()
			.cloned()
			.unwrap_or_default(),
	);

	let result = client.call_tool(params).await.map_err(|e| e.to_string())?;

	let mut text = String::new();
	for block in &result.content {
		if let rmcp::model::ContentBlock::Text(content) = block {
			if !text.is_empty() {
				text.push('\n');
			}
			text.push_str(&content.text);
		}
	}

	Ok(McpToolResult {
		is_error: result.is_error.unwrap_or(false),
		text,
		structured: result.structured_content,
	})
}

#[tauri::command]
pub async fn mcp_status(state: State<'_, McpManager>, server: String) -> Result<bool, String> {
	let guard = state.clients.lock().await;
	Ok(guard.contains_key(&server))
}

#[tauri::command]
pub async fn mcp_disconnect(state: State<'_, McpManager>, server: String) -> Result<(), String> {
	let mut guard = state.clients.lock().await;
	guard.remove(&server);
	Ok(())
}

#[tauri::command]
pub fn mcp_server_auth(
	app: AppHandle,
	server: String,
	client_id: String,
	client_secret: String,
) -> Result<(), String> {
	eprintln!(
		"[mcp_server_auth] cwd={}",
		std::env::current_dir()
			.map(|p| p.display().to_string())
			.unwrap_or_else(|e| format!("<error: {e}>"))
	);
	match servers::resolve_script_path(&server) {
		Ok(p) => eprintln!("[mcp_server_auth] script={}", p.display()),
		Err(e) => eprintln!("[mcp_server_auth] script resolution failed: {e}"),
	}

	let credentials_path = resolve_credentials(&app, &client_id, &client_secret)?;
	eprintln!("[mcp_server_auth] credentials={credentials_path}");
	let mut cmd = servers::auth_command(&server, Some(&credentials_path))?;
	cmd.stdin(std::process::Stdio::null());
	cmd.stdout(std::process::Stdio::null());
	cmd.stderr(std::process::Stdio::piped());
	let mut child = cmd.spawn().map_err(|e| e.to_string())?;
	if let Some(stderr) = child.stderr.take() {
		std::thread::spawn(move || {
			use std::io::{BufRead, BufReader};
			for line in BufReader::new(stderr).lines() {
				match line {
					Ok(l) => eprintln!("[google-calendar-auth] {l}"),
					Err(_) => break,
				}
			}
		});
	}
	Ok(())
}
