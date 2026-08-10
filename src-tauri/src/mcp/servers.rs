use std::path::Path;

pub const GOOGLE_CALENDAR_SERVER: &str = "google-calendar";
pub const IMAP_MAIL_SERVER: &str = "imap-mail";

pub struct McpServerConfig {
	pub command: String,
	pub args: Vec<String>,
	pub env: Vec<(String, String)>,
}

fn google_calendar_server_script() -> String {
	if let Ok(path) = std::env::var("GOOGLE_CALENDAR_MCP_SERVER_PATH") {
		return path;
	}
	"node_modules/@cocal/google-calendar-mcp/build/index.js".to_string()
}

fn find_upwards_script(raw: &str) -> Option<std::path::PathBuf> {
	let start = std::env::current_dir().ok()?;
	let mut dir = Some(start.as_path());
	while let Some(current) = dir {
		let candidate = current.join(raw);
		if candidate.exists() {
			return Some(candidate);
		}
		dir = current.parent();
	}
	None
}

fn find_near_executable(raw: &str) -> Option<std::path::PathBuf> {
	let exe = std::env::current_exe().ok()?;
	let mut dir = exe.parent();
	while let Some(current) = dir {
		let candidate = current.join(raw);
		if candidate.exists() {
			return Some(candidate);
		}
		dir = current.parent();
	}
	None
}

pub fn resolve_script_path(server: &str) -> Result<std::path::PathBuf, String> {
	match server {
		GOOGLE_CALENDAR_SERVER => {
			let raw = google_calendar_server_script();
			let path = std::path::PathBuf::from(&raw);
			if path.is_absolute() {
				if path.exists() {
					return Ok(path);
				}
				return Err(format!("MCP server script not found at {}", path.display()));
			}
			let abs = find_upwards_script(&raw)
				.or_else(|| find_near_executable(&raw))
				.ok_or_else(|| {
					format!(
						"MCP server script '{raw}' not found from cwd={:?}",
						std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|e| format!("<error: {e}>"))
					)
				})?;
			Ok(abs)
		}
		other => Err(format!("unknown MCP server: {other}")),
	}
}

fn derive_project_id(client_id: &str) -> Option<String> {
	client_id
		.strip_suffix(".apps.googleusercontent.com")
		.map(|prefix| prefix.to_string())
}

pub fn ensure_credentials(
	app_data_dir: &Path,
	client_id: &str,
	client_secret: &str,
) -> Result<std::path::PathBuf, String> {
	if client_id.trim().is_empty() || client_secret.trim().is_empty() {
		return Err("Google Client ID and Client Secret are required".into());
	}

	let project_id = derive_project_id(client_id).unwrap_or_default();

	let credentials = serde_json::json!({
		"installed": {
			"project_id": project_id,
			"client_id": client_id,
			"client_secret": client_secret,
			"auth_uri": "https://accounts.google.com/o/oauth2/auth",
			"token_uri": "https://oauth2.googleapis.com/token",
			"redirect_uris": ["http://localhost"]
		}
	});

	std::fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;

	let path = app_data_dir.join("gcp-oauth.keys.json");
	std::fs::write(&path, serde_json::to_string_pretty(&credentials).map_err(|e| e.to_string())?)
		.map_err(|e| e.to_string())?;

	Ok(path)
}

pub fn resolve(server: &str, credentials_path: Option<&str>) -> Result<McpServerConfig, String> {
	match server {
		GOOGLE_CALENDAR_SERVER => {
			let script = resolve_script_path(server)?;
			let mut env = Vec::new();
			if let Some(path) = credentials_path {
				if !path.trim().is_empty() {
					env.push(("GOOGLE_OAUTH_CREDENTIALS".to_string(), path.to_string()));
				}
			}
			Ok(McpServerConfig {
				command: "node".to_string(),
				args: vec![script.to_string_lossy().into_owned()],
				env,
			})
		}
		IMAP_MAIL_SERVER => Ok(McpServerConfig {
			command: "npx".to_string(),
			args: vec!["-y".to_string(), "imap-mcp-server".to_string()],
			env: Vec::new(),
		}),
		other => Err(format!("unknown MCP server: {other}")),
	}
}

pub fn auth_command(
	server: &str,
	credentials_path: Option<&str>,
) -> Result<std::process::Command, String> {
	let cfg = resolve(server, credentials_path)?;
	let mut cmd = std::process::Command::new(&cfg.command);
	cmd.args(&cfg.args);
	cmd.arg("auth");
	for (key, value) in &cfg.env {
		cmd.env(key, value);
	}
	Ok(cmd)
}

pub fn setup_wizard_command(port: u16) -> std::process::Command {
	let mut cmd = std::process::Command::new("npx");
	cmd.arg("-y")
		.arg("-p")
		.arg("imap-mcp-server")
		.arg("imap-setup")
		.arg("--skip-claude")
		.arg("--no-open")
		.arg("-p")
		.arg(port.to_string());
	cmd
}
