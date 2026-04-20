use std::sync::Arc;
use std::time::Duration;

use mc_sandbox::{
    Capability, CapabilityDeclaration, Guardian, GuardianDecision, PermissionLevel, ToolCallArgs,
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde_json::json;

use crate::types::{PermissionScope, Tool, ToolCategory, ToolResult};

static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<[^>]+>").expect("valid html regex"));
static SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?is)<(script|style)[^>]*>.*?</(script|style)>").expect("valid html cleanup regex")
});
static WHITESPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+").expect("valid whitespace regex"));

const DEFAULT_TIMEOUT_SECS: u64 = 20;
const DEFAULT_MAX_CHARS: usize = 8_000;

pub struct BrowserTool {
    client: Client,
    guardian: Option<Arc<Guardian>>,
    timeout: Duration,
}

impl BrowserTool {
    pub fn new(guardian: Option<Arc<Guardian>>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("browser tool http client should build");
        Self {
            client,
            guardian,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn fetch_url(&self, caller: &str, url: &str, mode: &str, max_chars: usize) -> ToolResult {
        let start = std::time::Instant::now();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return ToolResult::error("url must start with http:// or https://");
        }

        if let Some(guardian) = &self.guardian {
            let args = ToolCallArgs::shell_exec(format!("browser_fetch {url}"))
                .with_capability(self.capability());
            match guardian.check_tool_call(caller, self.name(), &args).await {
                GuardianDecision::Allow => {}
                GuardianDecision::Simulate { mock_result } => {
                    return ToolResult::content(mock_result).with_duration(start.elapsed());
                }
                GuardianDecision::Deny { reason }
                | GuardianDecision::ConfirmRequired { reason } => {
                    return ToolResult::error(reason).with_duration(start.elapsed());
                }
            }
        }

        let response = match tokio::time::timeout(self.timeout, self.client.get(url).send()).await {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                return ToolResult::error(format!("browser request failed: {error}"))
                    .with_duration(start.elapsed())
            }
            Err(_) => {
                return ToolResult::error(format!(
                    "browser request timed out after {}s",
                    self.timeout.as_secs()
                ))
                .with_duration(start.elapsed())
            }
        };

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => {
                return ToolResult::error(format!("failed to read browser response: {error}"))
                    .with_duration(start.elapsed())
            }
        };

        let processed = if mode.eq_ignore_ascii_case("html") {
            truncate(&body, max_chars)
        } else {
            truncate(&extract_text(&body), max_chars)
        };

        let status_code = status.as_u16();
        let mut result = if status.is_success() {
            ToolResult::success_with_data(
                processed,
                json!({
                    "url": url,
                    "status": status_code,
                    "mode": mode,
                    "content_type": content_type,
                    "truncated": body.len() > max_chars,
                }),
            )
        } else {
            ToolResult::content_with_data(
                format!("HTTP {status_code}\n{processed}"),
                json!({
                    "url": url,
                    "status": status_code,
                    "mode": mode,
                    "content_type": content_type,
                    "truncated": body.len() > max_chars,
                }),
            )
        };
        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    }
}

impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Fetch a web page and return either extracted text or raw HTML."
    }

    fn execute(
        &self,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let url = match params.get("url").and_then(serde_json::Value::as_str) {
                Some(url) => url,
                None => return ToolResult::error("missing required parameter: url"),
            };
            let mode = params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("text");
            let max_chars = params
                .get("max_chars")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(DEFAULT_MAX_CHARS);
            let caller = params
                .get("caller")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            self.fetch_url(caller, url, mode, max_chars).await
        })
    }

    fn required_parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "HTTP or HTTPS URL to fetch"
                },
                "mode": {
                    "type": "string",
                    "enum": ["text", "html"],
                    "default": "text",
                    "description": "Whether to return extracted text or raw HTML"
                },
                "max_chars": {
                    "type": "integer",
                    "default": DEFAULT_MAX_CHARS,
                    "description": "Maximum number of characters to return"
                }
            },
            "required": ["url"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Extended
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Elevated
    }

    fn permission_scope(&self) -> PermissionScope {
        PermissionScope::Search
    }

    fn capability(&self) -> CapabilityDeclaration {
        CapabilityDeclaration::new(
            "browser",
            "Fetch web content over HTTP",
            self.permission_level(),
            vec![Capability::NetworkAccess {
                pattern: "https?://.*".into(),
            }],
        )
    }
}

impl Default for BrowserTool {
    fn default() -> Self {
        Self::new(None)
    }
}

fn extract_text(html: &str) -> String {
    let without_scripts = SCRIPT_RE.replace_all(html, " ");
    let without_tags = TAG_RE.replace_all(&without_scripts, " ");
    WHITESPACE_RE
        .replace_all(without_tags.trim(), " ")
        .to_string()
}

fn truncate(content: &str, max_chars: usize) -> String {
    content.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::{extract_text, truncate};

    #[test]
    fn html_extraction_removes_tags_and_scripts() {
        let html = r#"<html><head><script>alert(1)</script></head><body><h1>Hello</h1><p>World</p></body></html>"#;
        let text = extract_text(html);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let content = "你好 world";
        assert_eq!(truncate(content, 2), "你好");
    }
}
