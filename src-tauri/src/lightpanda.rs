//! Managed installation and updates for the Lightpanda headless browser.
//!
//! The binary is downloaded from the Lightpanda nightly GitHub release into
//! `<app_data_dir>/bin/lightpanda` and kept up to date in the background.
//! Set `LIGHTPANDA_BROWSER_PATH` to bypass the managed install entirely.

use serde::Serialize;
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex as TokioMutex;

const NIGHTLY_API_URL: &str =
	"https://api.github.com/repos/lightpanda-io/browser/releases/tags/nightly";
const BINARY_NAME: &str = "lightpanda";

#[derive(Clone, Serialize)]
pub struct LightpandaStatus {
	pub installed: bool,
	pub updating: bool,
	pub version: Option<String>,
	pub error: Option<String>,
}

impl LightpandaStatus {
	fn ready(version: Option<String>) -> Self {
		Self {
			installed: true,
			updating: false,
			version,
			error: None,
		}
	}
}

#[derive(Default)]
pub struct LightpandaManager {
	status: StdMutex<Option<LightpandaStatus>>,
	install_lock: TokioMutex<()>,
}

impl LightpandaManager {
	fn set_status(&self, status: LightpandaStatus) {
		if let Ok(mut guard) = self.status.lock() {
			*guard = Some(status);
		}
	}

	fn current_status(&self) -> Option<LightpandaStatus> {
		self.status.lock().ok().and_then(|g| g.clone())
	}
}

pub fn env_override() -> Option<PathBuf> {
	std::env::var_os("LIGHTPANDA_BROWSER_PATH")
		.map(PathBuf::from)
		.filter(|p| p.exists())
}

pub fn managed_binary_path(app: &AppHandle) -> Result<PathBuf, String> {
	let dir = app
		.path()
		.app_data_dir()
		.map_err(|e| format!("failed to resolve app data dir: {e}"))?;
	Ok(dir.join("bin").join(BINARY_NAME))
}

fn platform_asset_name() -> Result<String, String> {
	let arch = match std::env::consts::ARCH {
		"x86_64" => "x86_64",
		"aarch64" => "aarch64",
		other => return Err(format!("unsupported architecture: {other}")),
	};
	let os = match std::env::consts::OS {
		"linux" => "linux",
		"macos" => "macos",
		other => return Err(format!("unsupported platform: {other}")),
	};
	Ok(format!("{BINARY_NAME}-{arch}-{os}"))
}

fn http_client() -> Result<reqwest::Client, String> {
	reqwest::Client::builder()
		.user_agent(concat!("miccy-app/", env!("CARGO_PKG_VERSION")))
		.timeout(Duration::from_secs(600))
		.build()
		.map_err(|e| e.to_string())
}

struct NightlyAsset {
	download_url: String,
	updated_at: String,
}

async fn latest_asset(client: &reqwest::Client) -> Result<NightlyAsset, String> {
	let name = platform_asset_name()?;
	let release: Value = client
		.get(NIGHTLY_API_URL)
		.header("Accept", "application/vnd.github+json")
		.send()
		.await
		.map_err(|e| e.to_string())?
		.error_for_status()
		.map_err(|e| e.to_string())?
		.json()
		.await
		.map_err(|e| e.to_string())?;

	for asset in release
		.get("assets")
		.and_then(|a| a.as_array())
		.into_iter()
		.flatten()
	{
		if asset.get("name").and_then(|n| n.as_str()) == Some(name.as_str()) {
			let download_url = asset
				.get("browser_download_url")
				.and_then(|u| u.as_str())
				.ok_or_else(|| "nightly asset missing download url".to_string())?
				.to_string();
			let updated_at = asset
				.get("updated_at")
				.and_then(|u| u.as_str())
				.unwrap_or_default()
				.to_string();
			return Ok(NightlyAsset {
				download_url,
				updated_at,
			});
		}
	}
	Err(format!("nightly asset '{name}' not found"))
}

/// Returns (asset_updated_at, version) stored next to the binary.
fn read_meta(bin: &PathBuf) -> Option<(String, String)> {
	let content = std::fs::read_to_string(bin.with_extension("json")).ok()?;
	let meta: Value = serde_json::from_str(&content).ok()?;
	Some((
		meta.get("asset_updated_at")
			.and_then(|v| v.as_str())
			.unwrap_or_default()
			.to_string(),
		meta.get("version")
			.and_then(|v| v.as_str())
			.unwrap_or_default()
			.to_string(),
	))
}

fn write_meta(bin: &PathBuf, asset_updated_at: &str, version: &str) {
	let payload = serde_json::json!({
		"asset_updated_at": asset_updated_at,
		"version": version,
	});
	let _ = std::fs::write(bin.with_extension("json"), payload.to_string());
}

fn detect_version(bin: &PathBuf) -> Option<String> {
	let out = std::process::Command::new(bin).arg("version").output().ok()?;
	if !out.status.success() {
		return None;
	}
	let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
	if s.is_empty() {
		None
	} else {
		Some(s)
	}
}

async fn download_to(
	client: &reqwest::Client,
	url: &str,
	dest: &PathBuf,
) -> Result<(), String> {
	let bytes = client
		.get(url)
		.send()
		.await
		.map_err(|e| e.to_string())?
		.error_for_status()
		.map_err(|e| e.to_string())?
		.bytes()
		.await
		.map_err(|e| e.to_string())?;

	let tmp = dest.with_extension("download");
	{
		let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
		f.write_all(&bytes).map_err(|e| e.to_string())?;
		f.sync_all().map_err(|e| e.to_string())?;
	}
	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
			.map_err(|e| e.to_string())?;
	}
	std::fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
	Ok(())
}

/// Downloads the binary if missing or outdated. Returns true if a new
/// binary was installed. Caller must hold the install lock.
async fn install_or_update(
	app: &AppHandle,
	manager: &LightpandaManager,
) -> Result<bool, String> {
	let bin = managed_binary_path(app)?;
	let client = http_client()?;
	let asset = latest_asset(&client).await?;

	if let Some((stored_at, stored_version)) = read_meta(&bin) {
		if bin.exists() && stored_at == asset.updated_at {
			manager.set_status(LightpandaStatus::ready(if stored_version.is_empty() {
				None
			} else {
				Some(stored_version)
			}));
			return Ok(false);
		}
	}

	manager.set_status(LightpandaStatus {
		installed: bin.exists(),
		updating: true,
		version: None,
		error: None,
	});

	if let Some(parent) = bin.parent() {
		std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
	}
	download_to(&client, &asset.download_url, &bin).await?;

	let version = detect_version(&bin);
	write_meta(&bin, &asset.updated_at, version.as_deref().unwrap_or(""));
	manager.set_status(LightpandaStatus::ready(version));
	Ok(true)
}

/// Resolves a usable lightpanda binary path, installing it on first use.
pub async fn ensure_ready(app: &AppHandle) -> Result<PathBuf, String> {
	if let Some(path) = env_override() {
		return Ok(path);
	}
	let manager = app.state::<LightpandaManager>();
	let _guard = manager.install_lock.lock().await;
	let bin = managed_binary_path(app)?;
	if bin.exists() {
		return Ok(bin);
	}
	install_or_update(app, &manager).await?;
	Ok(bin)
}

/// Fire-and-forget nightly freshness check at app startup. Failures are
/// tolerated as long as an existing binary is still usable.
pub fn spawn_background_update(app: AppHandle) {
	tauri::async_runtime::spawn(async move {
		let result = {
			let manager = app.state::<LightpandaManager>();
			let _guard = manager.install_lock.lock().await;
			install_or_update(&app, &manager).await
		};
		if let Err(e) = result {
			eprintln!("[lightpanda] background update failed: {e}");
			let manager = app.state::<LightpandaManager>();
			let installed = managed_binary_path(&app)
				.map(|b| b.exists())
				.unwrap_or(false);
			manager.set_status(LightpandaStatus {
				installed,
				updating: false,
				version: None,
				error: if installed { None } else { Some(e) },
			});
		}
	});
}

#[tauri::command]
pub fn lightpanda_status(
	app: AppHandle,
	manager: State<LightpandaManager>,
) -> LightpandaStatus {
	if let Some(path) = env_override() {
		return LightpandaStatus::ready(detect_version(&path));
	}
	let mut status = manager.current_status().unwrap_or(LightpandaStatus {
		installed: false,
		updating: false,
		version: None,
		error: None,
	});
	if !status.installed {
		status.installed = managed_binary_path(&app)
			.map(|b| b.exists())
			.unwrap_or(false);
	}
	if status.version.is_none() && status.installed {
		if let Ok(bin) = managed_binary_path(&app) {
			if let Some((_, v)) = read_meta(&bin) {
				if !v.is_empty() {
					status.version = Some(v);
				}
			}
		}
	}
	status
}
