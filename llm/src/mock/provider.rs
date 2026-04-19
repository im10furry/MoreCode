use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::{
    CacheCapability, ChatMessage, ChatRequest, ChatResponse, FinishReason, LlmError, LlmProvider,
    MessageRole, ModelInfo, StreamEvent, TokenUsage, ToolCall,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MockStreamChunk {
    pub content: String,
    pub tool_call: Option<ToolCall>,
}

impl MockStreamChunk {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tool_call: None,
        }
    }

    pub fn tool_call(tool_call: ToolCall) -> Self {
        Self {
            content: String::new(),
            tool_call: Some(tool_call),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub latency_ms: u64,
    pub raw_response: Option<Value>,
    pub error: Option<String>,
    pub stream_chunks: Vec<MockStreamChunk>,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            content: "{}".into(),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 8,
                completion_tokens: 16,
                cached_tokens: 0,
                total_tokens: 24,
            },
            latency_ms: 1,
            raw_response: None,
            error: None,
            stream_chunks: Vec::new(),
        }
    }
}

pub struct MockProvider {
    model_info: ModelInfo,
    models: Arc<RwLock<Vec<ModelInfo>>>,
    queued_responses: Arc<Mutex<VecDeque<MockResponse>>>,
    default_response: Arc<RwLock<MockResponse>>,
    recorded_requests: Arc<Mutex<Vec<ChatRequest>>>,
    active_requests: Arc<Mutex<HashMap<String, CancellationToken>>>,
    stream_buffer_size: usize,
}

impl MockProvider {
    pub fn new(model_info: ModelInfo) -> Self {
        Self {
            models: Arc::new(RwLock::new(vec![model_info.clone()])),
            model_info,
            queued_responses: Arc::new(Mutex::new(VecDeque::new())),
            default_response: Arc::new(RwLock::new(MockResponse::default())),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
            active_requests: Arc::new(Mutex::new(HashMap::new())),
            stream_buffer_size: 16,
        }
    }

    pub fn with_default_response(mut self, response: MockResponse) -> Self {
        self.default_response = Arc::new(RwLock::new(response));
        self
    }

    pub fn with_stream_buffer_size(mut self, stream_buffer_size: usize) -> Self {
        self.stream_buffer_size = stream_buffer_size.max(1);
        self
    }

    pub async fn enqueue_response(&self, response: MockResponse) {
        self.queued_responses.lock().await.push_back(response);
    }

    pub async fn set_default_response(&self, response: MockResponse) {
        *self.default_response.write().await = response;
    }

    pub async fn set_models(&self, models: Vec<ModelInfo>) {
        *self.models.write().await = models;
    }

    pub async fn recorded_requests(&self) -> Vec<ChatRequest> {
        self.recorded_requests.lock().await.clone()
    }

    async fn next_response(&self) -> MockResponse {
        if let Some(response) = self.queued_responses.lock().await.pop_front() {
            response
        } else {
            self.default_response.read().await.clone()
        }
    }

    async fn record_request(&self, request: ChatRequest) {
        self.recorded_requests.lock().await.push(request);
    }
}

impl LlmProvider for MockProvider {
    fn provider_id(&self) -> &str {
        &self.model_info.provider_id
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    fn chat(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, LlmError>> + Send + '_>> {
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "request cancelled before execution".into(),
                });
            }

            self.record_request(request.clone()).await;
            let response = self.next_response().await;
            if let Some(error) = response.error {
                return Err(LlmError::ApiError(error));
            }

            Ok(ChatResponse {
                id: request
                    .request_id
                    .clone()
                    .unwrap_or_else(|| "mock-response".into()),
                model: self.model_info.id.clone(),
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: crate::MessageContent::Text(response.content),
                    name: None,
                    tool_calls: response.tool_calls,
                    tool_call_id: None,
                    cache_control: None,
                },
                usage: response.usage,
                finish_reason: response.finish_reason,
                latency_ms: response.latency_ms,
                raw_response: response.raw_response,
            })
        })
    }

    fn chat_stream(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<mpsc::Receiver<StreamEvent>, LlmError>> + Send + '_>>
    {
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "request cancelled before stream start".into(),
                });
            }

            self.record_request(request.clone()).await;
            let request_id = request
                .request_id
                .clone()
                .unwrap_or_else(|| "mock-stream".into());
            let request_cancel = cancel_token.child_token();
            self.active_requests
                .lock()
                .await
                .insert(request_id.clone(), request_cancel.clone());

            let response = self.next_response().await;
            let (tx, rx) = mpsc::channel(self.stream_buffer_size);
            let active_requests = Arc::clone(&self.active_requests);

            tokio::spawn(async move {
                let result = async {
                    if let Some(error) = response.error {
                        return Err(LlmError::StreamError(error));
                    }

                    let mut emitted_content = String::new();
                    let mut tool_index = 0u32;

                    if response.stream_chunks.is_empty() {
                        if !response.content.is_empty() {
                            emitted_content.push_str(&response.content);
                            tx.send(StreamEvent::Delta {
                                content: response.content.clone(),
                                cumulative_tokens: Some(
                                    crate::estimate_text_tokens(&emitted_content) as u32,
                                ),
                            })
                            .await
                            .map_err(|_| {
                                LlmError::StreamError("stream receiver dropped".into())
                            })?;
                        }

                        for tool_call in &response.tool_calls {
                            tx.send(StreamEvent::ToolCallDelta {
                                index: tool_index,
                                id: Some(tool_call.id.clone()),
                                name: Some(tool_call.name.clone()),
                                arguments_delta: tool_call.arguments.clone(),
                            })
                            .await
                            .map_err(|_| {
                                LlmError::StreamError("stream receiver dropped".into())
                            })?;
                            tool_index = tool_index.saturating_add(1);
                        }
                    } else {
                        for chunk in response.stream_chunks {
                            tokio::select! {
                                _ = request_cancel.cancelled() => {
                                    return Err(LlmError::Cancelled {
                                        reason: "stream request cancelled".into(),
                                    });
                                }
                                send_result = async {
                                    if !chunk.content.is_empty() {
                                        emitted_content.push_str(&chunk.content);
                                        tx.send(StreamEvent::Delta {
                                            content: chunk.content,
                                            cumulative_tokens: Some(crate::estimate_text_tokens(&emitted_content) as u32),
                                        }).await.map_err(|_| LlmError::StreamError("stream receiver dropped".into()))?;
                                    }

                                    if let Some(tool_call) = chunk.tool_call {
                                        tx.send(StreamEvent::ToolCallDelta {
                                            index: tool_index,
                                            id: Some(tool_call.id),
                                            name: Some(tool_call.name),
                                            arguments_delta: tool_call.arguments,
                                        }).await.map_err(|_| LlmError::StreamError("stream receiver dropped".into()))?;
                                        tool_index = tool_index.saturating_add(1);
                                    }

                                    Ok::<(), LlmError>(())
                                } => send_result?,
                            }
                        }
                    }

                    tx.send(StreamEvent::Finish {
                        reason: response.finish_reason,
                        usage: Some(response.usage),
                        response_id: request_id.clone(),
                    })
                    .await
                    .map_err(|_| LlmError::StreamError("stream receiver dropped".into()))?;

                    Ok(())
                }
                .await;

                if let Err(error) = result {
                    let _ = tx.send(StreamEvent::Error(error.to_string())).await;
                }

                active_requests.lock().await.remove(&request_id);
            });

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
        Box::pin(async move { Ok(self.models.read().await.clone()) })
    }

    fn cancel_request(&self, request_id: &str) -> Result<(), LlmError> {
        let mut requests = self
            .active_requests
            .try_lock()
            .map_err(|_| LlmError::Internal("active request map is currently locked".into()))?;

        if let Some(token) = requests.remove(request_id) {
            token.cancel();
            Ok(())
        } else {
            Err(LlmError::ApiError(format!(
                "request '{request_id}' not found"
            )))
        }
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        crate::estimate_text_tokens(text)
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::sync::CancellationToken;

    use crate::{FinishReason, LlmProvider, StreamEvent, ToolCall};

    use super::{MockProvider, MockResponse, MockStreamChunk};

    fn provider() -> MockProvider {
        MockProvider::new(crate::ModelInfo::new("mock-model", "Mock Model", "mock"))
    }

    #[tokio::test]
    async fn chat_uses_enqueued_response() {
        let provider = provider();
        provider
            .enqueue_response(MockResponse {
                content: "ok".into(),
                ..Default::default()
            })
            .await;

        let response = provider
            .chat(crate::ChatRequest::default(), CancellationToken::new())
            .await
            .unwrap_or_else(|_| panic!("chat should succeed"));

        assert_eq!(response.message.content.to_text(), "ok");
    }

    #[tokio::test]
    async fn stream_emits_chunks_and_finish() {
        let provider = provider();
        provider
            .enqueue_response(MockResponse {
                finish_reason: FinishReason::ToolCalls,
                usage: crate::TokenUsage {
                    prompt_tokens: 5,
                    completion_tokens: 3,
                    cached_tokens: 0,
                    total_tokens: 8,
                },
                stream_chunks: vec![
                    MockStreamChunk::text("Hel"),
                    MockStreamChunk::text("lo"),
                    MockStreamChunk::tool_call(ToolCall {
                        id: "call_1".into(),
                        name: "search".into(),
                        arguments: r#"{"q":"rust"}"#.into(),
                    }),
                ],
                ..Default::default()
            })
            .await;

        let mut rx = provider
            .chat_stream(crate::ChatRequest::default(), CancellationToken::new())
            .await
            .unwrap_or_else(|_| panic!("stream should start"));

        let mut text = String::new();
        let mut saw_tool = false;
        let mut saw_finish = false;

        while let Some(event) = rx.recv().await {
            match event {
                StreamEvent::Delta { content, .. } => text.push_str(&content),
                StreamEvent::ToolCallDelta { name, .. } => {
                    assert_eq!(name.as_deref(), Some("search"));
                    saw_tool = true;
                }
                StreamEvent::Finish { reason, .. } => {
                    assert_eq!(reason, FinishReason::ToolCalls);
                    saw_finish = true;
                    break;
                }
                StreamEvent::Error(message) => panic!("unexpected stream error: {message}"),
            }
        }

        assert_eq!(text, "Hello");
        assert!(saw_tool);
        assert!(saw_finish);
    }
}
