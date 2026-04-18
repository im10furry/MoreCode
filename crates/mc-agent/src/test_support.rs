use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use mc_llm::{
    CacheCapability, ChatMessage, ChatRequest, ChatResponse, FinishReason, LlmError, LlmProvider,
    MessageRole, ModelInfo, StreamEvent, TokenUsage,
};
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub(crate) struct MockLlmProvider {
    model_info: ModelInfo,
    responses: HashMap<String, String>,
    requests: Arc<Mutex<Vec<ChatRequest>>>,
}

impl MockLlmProvider {
    pub(crate) fn new(responses: HashMap<String, String>) -> Self {
        Self {
            model_info: ModelInfo::new("mock-model", "Mock Model", "mock-provider"),
            responses,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn requests(&self) -> Arc<Mutex<Vec<ChatRequest>>> {
        Arc::clone(&self.requests)
    }
}

impl LlmProvider for MockLlmProvider {
    fn provider_id(&self) -> &str {
        "mock-provider"
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    fn chat(
        &self,
        request: ChatRequest,
        _cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, LlmError>> + Send + '_>> {
        Box::pin(async move {
            self.requests.lock().expect("request lock").push(request.clone());
            let schema_name = match &request.response_format {
                Some(mc_llm::ResponseFormat::JsonSchema { name, .. }) => name.clone(),
                _ => {
                    return Err(LlmError::Internal(
                        "mock provider requires json schema response format".to_string(),
                    ));
                }
            };
            let body = self
                .responses
                .get(&schema_name)
                .cloned()
                .unwrap_or_else(|| "{}".to_string());

            Ok(ChatResponse {
                id: format!("resp-{schema_name}"),
                model: self.model_info.id.clone(),
                message: ChatMessage::text(MessageRole::Assistant, body),
                usage: TokenUsage {
                    prompt_tokens: 8,
                    completion_tokens: 16,
                    cached_tokens: 0,
                    total_tokens: 24,
                },
                finish_reason: FinishReason::Stop,
                latency_ms: 10,
                raw_response: None,
            })
        })
    }

    fn chat_stream(
        &self,
        _request: ChatRequest,
        _cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<mpsc::Receiver<StreamEvent>, LlmError>> + Send + '_>>
    {
        Box::pin(async move {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        })
    }

    fn cache_capability(&self) -> CacheCapability {
        CacheCapability::default()
    }

    fn list_models(
        &self,
        _cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>> {
        Box::pin(async move { Ok(vec![self.model_info.clone()]) })
    }

    fn cancel_request(&self, _request_id: &str) -> Result<(), LlmError> {
        Ok(())
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.len().div_ceil(4)
    }
}

pub(crate) fn create_test_project() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join(".gitignore"),
        "target/\n.assistant-memory/\nnode_modules/\n",
    )
    .expect("gitignore");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        r#"[workspace]
members = ["crates/app", "crates/core"]
resolver = "2"
"#,
    )
    .expect("workspace cargo");
    std::fs::create_dir_all(dir.path().join("crates/app/src")).expect("app src");
    std::fs::create_dir_all(dir.path().join("crates/core/src")).expect("core src");
    std::fs::write(
        dir.path().join("crates/app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
core = { path = "../core" }
tokio = "1"
"#,
    )
    .expect("app cargo");
    std::fs::write(
        dir.path().join("crates/core/Cargo.toml"),
        r#"[package]
name = "core"
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("core cargo");
    std::fs::write(
        dir.path().join("crates/core/src/lib.rs"),
        r#"
pub struct SharedState;

pub fn compute() -> usize {
    42
}
"#,
    )
    .expect("core lib");
    std::fs::write(
        dir.path().join("crates/app/src/main.rs"),
        r#"
use core::compute;

fn main() {
    println!("{}", compute());
}
"#,
    )
    .expect("app main");
    std::fs::write(
        dir.path().join("README.md"),
        "# Sample Workspace\n\nA tiny Rust workspace for cognitive-agent tests.\n",
    )
    .expect("readme");
    dir
}
