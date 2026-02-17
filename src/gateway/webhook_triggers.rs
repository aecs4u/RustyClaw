//! Webhook trigger endpoint for external integrations.

use super::{ChatMessage, ModelContext, ProviderRequest, SharedModelCtx, providers};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    #[serde(default)]
    message: String,
    #[serde(default)]
    context: Option<serde_json::Value>,
}

pub async fn start_webhook_server(
    listen_addr: &str,
    path_prefix: &str,
    secret: Option<String>,
    shared_model_ctx: SharedModelCtx,
    cancel: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind webhook server to {}", listen_addr))?;
    let path_prefix = normalize_path_prefix(path_prefix);
    let http = reqwest::Client::new();

    eprintln!(
        "[webhook] Listening on http://{}{}",
        listen_addr, path_prefix
    );

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                eprintln!("[webhook] Shutting down webhook server");
                break;
            }
            accepted = listener.accept() => {
                let (mut stream, peer) = accepted?;
                let ctx = shared_model_ctx.clone();
                let path_prefix = path_prefix.clone();
                let secret = secret.clone();
                let http = http.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_request(&mut stream, &http, ctx, &path_prefix, secret.as_deref()).await {
                        eprintln!("[webhook] Request error from {}: {}", peer, e);
                    }
                });
            }
        }
    }

    Ok(())
}

fn normalize_path_prefix(path_prefix: &str) -> String {
    let trimmed = path_prefix.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/webhook".to_string()
    } else if trimmed.starts_with('/') {
        trimmed.trim_end_matches('/').to_string()
    } else {
        format!("/{}", trimmed.trim_end_matches('/'))
    }
}

async fn handle_request(
    stream: &mut tokio::net::TcpStream,
    http: &reqwest::Client,
    shared_model_ctx: SharedModelCtx,
    path_prefix: &str,
    secret: Option<&str>,
) -> Result<()> {
    let mut buf = vec![0u8; 256 * 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]).to_string();
    let (header_part, body_part) = request
        .split_once("\r\n\r\n")
        .unwrap_or((request.as_str(), ""));

    let mut lines = header_part.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut request_bits = request_line.split_whitespace();
    let method = request_bits.next().unwrap_or_default();
    let path = request_bits.next().unwrap_or("/");

    if method != "POST" {
        return write_json(stream, "405 Method Not Allowed", json!({
            "ok": false,
            "error": "Only POST is supported",
        }))
        .await;
    }

    if !(path == path_prefix || path.starts_with(&format!("{}/", path_prefix))) {
        return write_json(stream, "404 Not Found", json!({
            "ok": false,
            "error": "Unknown webhook path",
            "path": path,
        }))
        .await;
    }

    if let Some(expected) = secret {
        let provided = extract_header(&request, "x-webhook-secret");
        if provided.as_deref() != Some(expected) {
            return write_json(stream, "401 Unauthorized", json!({
                "ok": false,
                "error": "Missing or invalid webhook secret",
            }))
            .await;
        }
    }

    let payload: WebhookPayload = serde_json::from_str(body_part).context("Invalid webhook JSON")?;
    if payload.message.trim().is_empty() {
        return write_json(stream, "400 Bad Request", json!({
            "ok": false,
            "error": "Field 'message' is required",
        }))
        .await;
    }

    let model_ctx = shared_model_ctx.read().await.clone();
    let Some(model_ctx) = model_ctx else {
        return write_json(stream, "503 Service Unavailable", json!({
            "ok": false,
            "error": "No model configured",
        }))
        .await;
    };

    let response = run_webhook_prompt(http, &model_ctx, &payload).await?;
    write_json(stream, "200 OK", json!({
        "ok": true,
        "provider": model_ctx.provider,
        "model": model_ctx.model,
        "response": response,
    }))
    .await
}

fn extract_header(request: &str, header_name: &str) -> Option<String> {
    let target = format!("{}:", header_name.to_ascii_lowercase());
    for line in request.lines().skip(1) {
        if line.trim().is_empty() {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if lower.starts_with(&target) {
            return line
                .split_once(':')
                .map(|(_, v)| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
    }
    None
}

async fn run_webhook_prompt(
    http: &reqwest::Client,
    model_ctx: &ModelContext,
    payload: &WebhookPayload,
) -> Result<String> {
    let mut content = payload.message.clone();
    if let Some(ctx) = &payload.context {
        content = format!("Webhook context:\n{}\n\n{}", serde_json::to_string_pretty(ctx)?, content);
    }

    let req = ProviderRequest {
        provider: model_ctx.provider.clone(),
        model: model_ctx.model.clone(),
        base_url: model_ctx.base_url.clone(),
        api_key: model_ctx.api_key.clone(),
        messages: vec![ChatMessage::text("user", &content)],
    };

    let resp = if req.provider == "anthropic" {
        providers::call_anthropic_with_tools(http, &req, None).await?
    } else if req.provider == "google" {
        providers::call_google_with_tools(http, &req).await?
    } else {
        providers::call_openai_with_tools(http, &req).await?
    };

    Ok(resp.text)
}

async fn write_json(
    stream: &mut tokio::net::TcpStream,
    status: &str,
    body: serde_json::Value,
) -> Result<()> {
    let body = body.to_string();
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_prefix() {
        assert_eq!(normalize_path_prefix("/webhook"), "/webhook");
        assert_eq!(normalize_path_prefix("webhook"), "/webhook");
        assert_eq!(normalize_path_prefix("/webhook/"), "/webhook");
        assert_eq!(normalize_path_prefix(""), "/webhook");
    }
}
