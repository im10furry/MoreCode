use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{CacheCapability, ChatRequest, ChatResponse, LlmError, ModelInfo, StreamEvent};

pub trait LlmProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    fn model_info(&self) -> &ModelInfo;

    fn chat(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, LlmError>> + Send + '_>>;

    fn chat_stream(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<mpsc::Receiver<StreamEvent>, LlmError>> + Send + '_>>;

    fn cache_capability(&self) -> CacheCapability;

    fn list_models(
        &self,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>>;

    fn cancel_request(&self, request_id: &str) -> Result<(), LlmError>;

    fn estimate_tokens(&self, text: &str) -> usize;
}
