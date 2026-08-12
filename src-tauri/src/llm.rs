use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

pub const DEEPSEEK_URL: &str = "https://api.deepseek.com/chat/completions";
pub const DEFAULT_MODEL: &str = "deepseek-v4-flash";

async fn run_stream(
    app: &AppHandle,
    request_id: &str,
    mut payload: Value,
    api_key: &str,
    api_endpoint: &str,
) -> Result<(), String> {
    if payload.get("model").and_then(Value::as_str).is_none() {
        payload["model"] = json!(DEFAULT_MODEL);
    }
    if !payload
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        payload["stream"] = json!(true);
    }

	let api_url: &str;

    if api_endpoint.is_empty() {
        api_url = DEEPSEEK_URL;
    } else {
        api_url = api_endpoint; // no .as_str() needed, it's already &str
    }

    let client = reqwest::Client::new();
    let response = client
        .post(api_url)
        .timeout(Duration::from_secs(300))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {api_key}"))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("DeepSeek responded {status}: {text}"));
    }

    let mut stream = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
        buffer.extend_from_slice(&chunk);

        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let mut raw: Vec<u8> = buffer.drain(..=pos).collect();
            raw.pop();
            while raw.last() == Some(&b'\r') {
                raw.pop();
            }
            let line = String::from_utf8_lossy(&raw).trim().to_string();
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                app.emit("llm:done", json!({ "requestId": request_id }))
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            if data.is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(data) {
                Ok(chunk) => {
                    app.emit(
                        "llm:delta",
                        json!({ "requestId": request_id, "chunk": chunk }),
                    )
                    .map_err(|e| e.to_string())?;
                }
                Err(e) => eprintln!("[llm] ignoring malformed SSE chunk: {e}"),
            }
        }
    }

    app.emit("llm:done", json!({ "requestId": request_id }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Proxy chat-completions requests to DeepSeek and stream the raw SSE chunks
/// back to the webview as `llm:delta` / `llm:done` / `llm:error` events.
#[tauri::command]
pub async fn llm_stream_chat(
    app: AppHandle,
    request_id: String,
    payload: Value,
    api_key: String,
	api_endpoint: String,
) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("DeepSeek API key is not set".into());
    }
    tokio::spawn(async move {
        if let Err(e) = run_stream(&app, &request_id, payload, &api_key, &api_endpoint).await {
            let _ = app.emit("llm:error", json!({ "requestId": request_id, "error": e }));
        }
    });
    Ok(())
}
