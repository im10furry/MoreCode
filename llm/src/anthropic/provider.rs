use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::header::{HeaderValue, RETRY_AFTER};
use reqwest::{Client, Response, StatusCode};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::{
    CacheCapability, CacheStrategy, ChatMessage, ChatRequest, ChatResponse, ContentPart,
    FinishReason, LlmError, LlmProvider, MessageContent, MessageRole, ModelInfo, ResponseFormat,
    StreamEvent, TokenUsage, ToolCall, ToolDefinition,
};

use super::AnthropicProviderConfig;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

const DEFAULT_MAX_TOKENS: u32 = 4_096;

pub struct AnthropicProvider {
    client: Client,
    base_url: String,
    api_key: Arc<str>,
    anthropic_version: Arc<str>,
    beta_headers: Vec<String>,
    model: ModelInfo,
    default_headers: HashMap<String, String>,
    cache_capability: CacheCapability,
    active_requests: Arc<RwLock<HashMap<String, CancellationToken>>>,
    request_timeout: Duration,
    stream_buffer_size: usize,
    default_max_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: ModelInfo,
    ) -> Result<Self, LlmError> {
        let default_max_tokens = if model.max_output_tokens > 0 {
            model.max_output_tokens
        } else {
            DEFAULT_MAX_TOKENS
        };

        Self::from_config(AnthropicProviderConfig {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model,
            anthropic_version: "2023-06-01".to_string(),
            beta_headers: Vec::new(),
            default_headers: HashMap::new(),
            request_timeout: Duration::from_secs(120),
            stream_buffer_size: 64,
            default_max_tokens,
        })
    }

    pub fn from_config(config: AnthropicProviderConfig) -> Result<Self, LlmError> {
        config.validate()?;

        let client = Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|error| {
                LlmError::ApiError(format!("failed to create HTTP client: {error}"))
            })?;

        Ok(Self {
            client,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key: Arc::<str>::from(config.api_key),
            anthropic_version: Arc::<str>::from(config.anthropic_version),
            beta_headers: config.beta_headers,
            model: config.model,
            default_headers: config.default_headers,
            cache_capability: CacheCapability {
                supports_prompt_caching: false,
                max_cache_ttl_secs: None,
                min_cacheable_tokens: 0,
                supported_control_types: Vec::new(),
                strategy: CacheStrategy::None,
            },
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: config.request_timeout,
            stream_buffer_size: config.stream_buffer_size,
            default_max_tokens: config.default_max_tokens,
        })
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    fn build_messages_body(&self, request: &ChatRequest, stream: bool) -> Result<Value, LlmError> {
        self.validate_request_capabilities(request)?;

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.model.id.clone());
        let mut system_blocks = Vec::new();
        let mut messages = Vec::new();

        for message in &request.messages {
            match message.role {
                // Anthropic hoists system instructions to the top-level `system` field.
                MessageRole::System => {
                    if !message.tool_calls.is_empty() || message.tool_call_id.is_some() {
                        return Err(LlmError::ApiError(
                            "anthropic system messages cannot contain tool calls".into(),
                        ));
                    }

                    system_blocks.extend(self.message_content_to_blocks(&message.content, false)?);
                }
                // `tool` role in the shared abstraction maps to a synthetic `user` turn with a
                // single `tool_result` content block.
                MessageRole::Tool => {
                    let tool_use_id = message.tool_call_id.clone().ok_or_else(|| {
                        LlmError::ApiError(
                            "anthropic tool messages require tool_call_id to map to tool_result"
                                .into(),
                        )
                    })?;

                    let mut tool_result = json!({
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": self.tool_result_content_to_json(&message.content)?,
                    });

                    if let Some(name) = &message.name {
                        tool_result["tool_name"] = Value::String(name.clone());
                    }

                    messages.push(json!({
                        "role": "user",
                        "content": [tool_result],
                    }));
                }
                MessageRole::User | MessageRole::Assistant => {
                    let content = self.message_to_anthropic_content(message)?;
                    messages.push(json!({
                        "role": anthropic_role_name(message.role)?,
                        "content": Value::Array(content),
                    }));
                }
            }
        }

        if messages.is_empty() {
            return Err(LlmError::ApiError(
                "anthropic requests require at least one non-system message".into(),
            ));
        }

        let mut body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(self.default_max_tokens),
            "temperature": request.temperature,
            "stream": stream,
        });

        if !system_blocks.is_empty() {
            body["system"] = Value::Array(system_blocks);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        if !request.stop_sequences.is_empty() {
            body["stop_sequences"] = json!(request.stop_sequences);
        }
        if !request.tools.is_empty() {
            body["tools"] = Value::Array(
                request
                    .tools
                    .iter()
                    .map(tool_definition_to_anthropic_json)
                    .collect(),
            );
        }

        Ok(body)
    }

    fn validate_request_capabilities(&self, request: &ChatRequest) -> Result<(), LlmError> {
        if request.messages.is_empty() {
            return Err(LlmError::ApiError(
                "anthropic requests require at least one message".into(),
            ));
        }

        if !request.cache_control_points.is_empty()
            || request
                .messages
                .iter()
                .any(|message| message.cache_control.is_some())
        {
            return Err(LlmError::ApiError(
                "anthropic provider draft does not yet map prompt cache control points".into(),
            ));
        }

        if let Some(response_format) = &request.response_format {
            if !matches!(response_format, ResponseFormat::Text) {
                return Err(LlmError::ApiError(
                    "anthropic native messages API does not support the current response_format abstraction directly"
                        .into(),
                ));
            }
        }

        Ok(())
    }

    fn message_to_anthropic_content(&self, message: &ChatMessage) -> Result<Vec<Value>, LlmError> {
        match message.role {
            MessageRole::System | MessageRole::Tool => {
                return Err(LlmError::Internal(
                    "message_to_anthropic_content only supports user/assistant messages".into(),
                ));
            }
            MessageRole::User => {
                if !message.tool_calls.is_empty() {
                    return Err(LlmError::ApiError(
                        "anthropic user messages cannot contain tool_calls".into(),
                    ));
                }
                if message.tool_call_id.is_some() {
                    return Err(LlmError::ApiError(
                        "anthropic user messages cannot contain tool_call_id".into(),
                    ));
                }
            }
            MessageRole::Assistant => {
                if message.tool_call_id.is_some() {
                    return Err(LlmError::ApiError(
                        "anthropic assistant messages cannot contain tool_call_id".into(),
                    ));
                }
            }
        }

        let mut blocks = self.message_content_to_blocks(&message.content, true)?;
        if matches!(message.role, MessageRole::Assistant) {
            for tool_call in &message.tool_calls {
                blocks.push(tool_call_to_anthropic_block(tool_call)?);
            }
        }

        if blocks.is_empty() {
            blocks.push(json!({
                "type": "text",
                "text": "",
            }));
        }

        Ok(blocks)
    }

    fn message_content_to_blocks(
        &self,
        content: &MessageContent,
        allow_files: bool,
    ) -> Result<Vec<Value>, LlmError> {
        match content {
            MessageContent::Text(text) => Ok(vec![json!({
                "type": "text",
                "text": text,
            })]),
            MessageContent::Parts(parts) => parts
                .iter()
                .map(|part| content_part_to_anthropic_block(part, allow_files))
                .collect(),
        }
    }

    fn tool_result_content_to_json(&self, content: &MessageContent) -> Result<Value, LlmError> {
        match content {
            MessageContent::Text(text) => Ok(Value::String(text.clone())),
            MessageContent::Parts(parts) => Ok(Value::Array(
                parts
                    .iter()
                    .map(content_part_to_tool_result_block)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    async fn register_request(&self, request_id: &str, cancel_token: CancellationToken) {
        self.active_requests
            .write()
            .await
            .insert(request_id.to_string(), cancel_token);
    }

    async fn unregister_request(&self, request_id: &str) {
        self.active_requests.write().await.remove(request_id);
    }

    async fn send_request(
        &self,
        request: &ChatRequest,
        cancel_token: CancellationToken,
        stream: bool,
    ) -> Result<Response, LlmError> {
        let body = self.build_messages_body(request, stream)?;
        let url = format!("{}/messages", self.base_url);
        let timeout = request.timeout.unwrap_or(self.request_timeout);
        let timeout_ms = duration_to_millis(timeout);

        let mut builder = self
            .client
            .post(url)
            .timeout(timeout)
            .header("x-api-key", self.api_key.as_ref())
            .header("anthropic-version", self.anthropic_version.as_ref())
            .json(&body);

        if !self.beta_headers.is_empty() {
            builder = builder.header("anthropic-beta", self.beta_headers.join(","));
        }
        for (key, value) in &self.default_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
        for (key, value) in &request.extra_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        tokio::select! {
            _ = cancel_token.cancelled() => Err(LlmError::Cancelled {
                reason: "request cancelled before send".into(),
            }),
            response = builder.send() => response.map_err(|error| map_reqwest_error(error, timeout_ms)),
        }
    }

    async fn read_text(
        &self,
        response: Response,
        cancel_token: CancellationToken,
        timeout: Duration,
    ) -> Result<String, LlmError> {
        let timeout_ms = duration_to_millis(timeout);
        tokio::select! {
            _ = cancel_token.cancelled() => Err(LlmError::Cancelled {
                reason: "request cancelled while reading response".into(),
            }),
            body = response.text() => body.map_err(|error| {
                if error.is_timeout() {
                    LlmError::Timeout { timeout_ms }
                } else {
                    LlmError::ApiError(format!("failed to read response body: {error}"))
                }
            }),
        }
    }

    fn parse_chat_response(&self, value: Value, latency_ms: u64) -> Result<ChatResponse, LlmError> {
        let id = string_field(&value, "id").unwrap_or_else(|| "unknown".into());
        let model = string_field(&value, "model").unwrap_or_else(|| self.model.id.clone());
        let usage = parse_usage(value.get("usage"))?.unwrap_or_default();
        let message = parse_chat_message(&value)?;
        let finish_reason =
            parse_stop_reason_value(value.get("stop_reason"))?.unwrap_or(FinishReason::Stop);

        Ok(ChatResponse {
            id,
            model,
            message,
            usage,
            finish_reason,
            latency_ms,
            raw_response: Some(value),
        })
    }

    fn parse_models_response(&self, value: Value) -> Vec<ModelInfo> {
        value
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                let id = string_field(&item, "id").unwrap_or_else(|| self.model.id.clone());
                if id == self.model.id {
                    self.model.clone()
                } else {
                    let mut model = self.model.clone();
                    model.id = id.clone();
                    model.display_name = string_field(&item, "display_name").unwrap_or(id);
                    model.provider_id = self.model.provider_id.clone();
                    model
                }
            })
            .collect()
    }
}

impl LlmProvider for AnthropicProvider {
    fn provider_id(&self) -> &str {
        &self.model.provider_id
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model
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

            let request_id = request_id_for(&request);
            let request_cancel = cancel_token.child_token();
            self.register_request(&request_id, request_cancel.clone())
                .await;

            let start = Instant::now();
            let timeout = request.timeout.unwrap_or(self.request_timeout);
            let result = async {
                let response = self
                    .send_request(&request, request_cancel.clone(), false)
                    .await?;
                let status = response.status();
                let headers = response.headers().clone();
                let body_text = self
                    .read_text(response, request_cancel.clone(), timeout)
                    .await?;

                if status.is_success() {
                    let value: Value = serde_json::from_str(&body_text).map_err(|error| {
                        LlmError::ApiError(format!(
                            "failed to parse anthropic chat response JSON: {error}"
                        ))
                    })?;
                    self.parse_chat_response(value, start.elapsed().as_millis() as u64)
                } else {
                    Err(map_status_error(
                        status,
                        headers.get(RETRY_AFTER),
                        &self.model,
                        request.estimated_prompt_tokens() as u32,
                        body_text,
                    ))
                }
            }
            .await;

            self.unregister_request(&request_id).await;
            result
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

            let request_id = request_id_for(&request);
            let request_cancel = cancel_token.child_token();
            self.register_request(&request_id, request_cancel.clone())
                .await;

            let timeout = request.timeout.unwrap_or(self.request_timeout);
            let response = match self
                .send_request(&request, request_cancel.clone(), true)
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    self.unregister_request(&request_id).await;
                    return Err(error);
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let headers = response.headers().clone();
                let body_text = self
                    .read_text(response, request_cancel.clone(), timeout)
                    .await?;
                self.unregister_request(&request_id).await;
                return Err(map_status_error(
                    status,
                    headers.get(RETRY_AFTER),
                    &self.model,
                    request.estimated_prompt_tokens() as u32,
                    body_text,
                ));
            }

            let prompt_tokens_estimate = request.estimated_prompt_tokens() as u32;
            let (tx, rx) = mpsc::channel(self.stream_buffer_size);
            let active_requests = Arc::clone(&self.active_requests);
            let request_id_for_task = request_id.clone();

            tokio::spawn(async move {
                let result = pump_sse_stream(
                    response,
                    tx.clone(),
                    request_cancel.clone(),
                    prompt_tokens_estimate,
                )
                .await;

                if let Err(error) = result {
                    let _ = tx.send(StreamEvent::Error(error.to_string())).await;
                }

                active_requests.write().await.remove(&request_id_for_task);
            });

            Ok(rx)
        })
    }

    fn cache_capability(&self) -> CacheCapability {
        self.cache_capability.clone()
    }

    fn list_models(
        &self,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>> {
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "model listing cancelled before execution".into(),
                });
            }

            let timeout_ms = duration_to_millis(self.request_timeout);
            let mut builder = self
                .client
                .get(format!("{}/models", self.base_url))
                .timeout(self.request_timeout)
                .header("x-api-key", self.api_key.as_ref())
                .header("anthropic-version", self.anthropic_version.as_ref());

            if !self.beta_headers.is_empty() {
                builder = builder.header("anthropic-beta", self.beta_headers.join(","));
            }
            for (key, value) in &self.default_headers {
                builder = builder.header(key.as_str(), value.as_str());
            }

            let response = tokio::select! {
                _ = cancel_token.cancelled() => Err(LlmError::Cancelled {
                    reason: "model listing cancelled".into(),
                }),
                response = builder.send() => response.map_err(|error| map_reqwest_error(error, timeout_ms)),
            }?;

            let status = response.status();
            let headers = response.headers().clone();
            let body_text = self
                .read_text(response, cancel_token.child_token(), self.request_timeout)
                .await?;

            if !status.is_success() {
                return Err(map_status_error(
                    status,
                    headers.get(RETRY_AFTER),
                    &self.model,
                    0,
                    body_text,
                ));
            }

            let value: Value = serde_json::from_str(&body_text).map_err(|error| {
                LlmError::ApiError(format!(
                    "failed to parse anthropic models response JSON: {error}"
                ))
            })?;
            Ok(self.parse_models_response(value))
        })
    }

    fn cancel_request(&self, request_id: &str) -> Result<(), LlmError> {
        let mut requests = self
            .active_requests
            .try_write()
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

async fn pump_sse_stream(
    response: Response,
    tx: mpsc::Sender<StreamEvent>,
    cancel_token: CancellationToken,
    prompt_tokens_estimate: u32,
) -> Result<(), LlmError> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut state = StreamState::new(prompt_tokens_estimate);

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                state.finish_reason = FinishReason::Cancelled;
                send_stream_event(&tx, state.finish_event()).await?;
                return Err(LlmError::Cancelled {
                    reason: "stream request cancelled".into(),
                });
            }
            maybe_chunk = stream.next() => {
                match maybe_chunk {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(frame) = next_sse_frame(&mut buffer) {
                            if process_sse_frame(&frame, &mut state, &tx).await? {
                                return Ok(());
                            }
                        }
                    }
                    Some(Err(error)) => {
                        return Err(LlmError::StreamError(format!(
                            "failed to read streaming response chunk: {error}"
                        )));
                    }
                    None => break,
                }
            }
        }
    }

    if !buffer.trim().is_empty() {
        let _ = process_sse_frame(&buffer, &mut state, &tx).await?;
    }

    send_stream_event(&tx, state.finish_event()).await?;
    Ok(())
}

async fn process_sse_frame(
    frame: &str,
    state: &mut StreamState,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<bool, LlmError> {
    let parsed = parse_sse_frame(frame);
    let Some(event_type) = parsed.event else {
        return Ok(false);
    };

    if parsed.data.trim().is_empty() {
        return Ok(false);
    }

    let value: Value = serde_json::from_str(&parsed.data).map_err(|error| {
        LlmError::StreamError(format!("failed to parse anthropic SSE payload: {error}"))
    })?;

    match event_type.as_str() {
        "ping" | "content_block_stop" => Ok(false),
        "message_start" => {
            let message = value.get("message").cloned().unwrap_or(Value::Null);
            if state.response_id.is_empty() {
                state.response_id = string_field(&message, "id").unwrap_or_default();
            }
            if let Some(usage) = parse_usage(message.get("usage"))? {
                state.usage = Some(usage);
            }
            Ok(false)
        }
        "content_block_start" => {
            let index = u32_field(&value, "index").unwrap_or(0);
            let block = value.get("content_block").cloned().unwrap_or(Value::Null);
            if string_field(&block, "type").as_deref() == Some("tool_use") {
                state.tool_calls.insert(
                    index,
                    ToolCallStreamState {
                        id: string_field(&block, "id"),
                        name: string_field(&block, "name"),
                    },
                );
            }
            Ok(false)
        }
        "content_block_delta" => {
            let index = u32_field(&value, "index").unwrap_or(0);
            let delta = value.get("delta").cloned().unwrap_or(Value::Null);
            match string_field(&delta, "type").as_deref() {
                Some("text_delta") => {
                    let text = string_field(&delta, "text").unwrap_or_default();
                    if !text.is_empty() {
                        state.content.push_str(&text);
                        send_stream_event(
                            tx,
                            StreamEvent::Delta {
                                content: text,
                                cumulative_tokens: Some(
                                    crate::estimate_text_tokens(&state.content) as u32,
                                ),
                            },
                        )
                        .await?;
                    }
                    Ok(false)
                }
                Some("input_json_delta") => {
                    let partial_json = string_field(&delta, "partial_json").unwrap_or_default();
                    if partial_json.is_empty() {
                        return Ok(false);
                    }

                    let tool_state = state.tool_calls.entry(index).or_default();
                    send_stream_event(
                        tx,
                        StreamEvent::ToolCallDelta {
                            index,
                            id: tool_state.id.clone(),
                            name: tool_state.name.clone(),
                            arguments_delta: partial_json,
                        },
                    )
                    .await?;
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        "message_delta" => {
            let delta = value.get("delta").cloned().unwrap_or(Value::Null);
            if let Some(finish_reason) = parse_stop_reason_value(delta.get("stop_reason"))? {
                state.finish_reason = finish_reason;
            }
            if let Some(usage) = parse_usage(value.get("usage"))? {
                state.usage = Some(usage);
            }
            Ok(false)
        }
        "message_stop" => {
            send_stream_event(tx, state.finish_event()).await?;
            Ok(true)
        }
        "error" => {
            let message = value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("anthropic streaming error");
            Err(LlmError::StreamError(message.to_string()))
        }
        _ => Ok(false),
    }
}

async fn send_stream_event(
    tx: &mpsc::Sender<StreamEvent>,
    event: StreamEvent,
) -> Result<(), LlmError> {
    tx.send(event)
        .await
        .map_err(|_| LlmError::StreamError("stream receiver dropped".into()))
}

fn next_sse_frame(buffer: &mut String) -> Option<String> {
    let split_index = buffer.find("\r\n\r\n").or_else(|| buffer.find("\n\n"))?;

    let delimiter_len = if buffer[split_index..].starts_with("\r\n\r\n") {
        4
    } else {
        2
    };

    let frame = buffer[..split_index].to_string();
    let remainder = buffer[(split_index + delimiter_len)..].to_string();
    *buffer = remainder;
    Some(frame)
}

fn parse_chat_message(value: &Value) -> Result<ChatMessage, LlmError> {
    let content_blocks = value
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| LlmError::ApiError("anthropic response missing content array".into()))?;
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .map(role_from_str)
        .transpose()?
        .unwrap_or(MessageRole::Assistant);

    let mut text_segments = Vec::new();
    let mut tool_calls = Vec::new();

    for block in content_blocks {
        match string_field(block, "type").as_deref() {
            Some("text") => {
                if let Some(text) = string_field(block, "text") {
                    text_segments.push(text);
                }
            }
            Some("tool_use") => {
                let name = string_field(block, "name")
                    .ok_or_else(|| LlmError::ApiError("tool_use block missing name".into()))?;
                let arguments = serde_json::to_string(
                    &block.get("input").cloned().unwrap_or_else(|| json!({})),
                )
                .map_err(|error| {
                    LlmError::ApiError(format!("failed to serialize anthropic tool input: {error}"))
                })?;

                tool_calls.push(ToolCall {
                    id: string_field(block, "id").unwrap_or_else(|| "tool_use".into()),
                    name,
                    arguments,
                });
            }
            _ => {}
        }
    }

    Ok(ChatMessage {
        role,
        content: MessageContent::Text(text_segments.join("")),
        name: None,
        tool_calls,
        tool_call_id: None,
        cache_control: None,
    })
}

fn parse_usage(value: Option<&Value>) -> Result<Option<TokenUsage>, LlmError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }

    let direct_input_tokens = u32_field(value, "input_tokens").unwrap_or(0);
    let cache_creation_tokens = u32_field(value, "cache_creation_input_tokens").unwrap_or(0);
    let cache_read_tokens = u32_field(value, "cache_read_input_tokens").unwrap_or(0);
    let prompt_tokens = direct_input_tokens
        .saturating_add(cache_creation_tokens)
        .saturating_add(cache_read_tokens);
    let completion_tokens = u32_field(value, "output_tokens").unwrap_or(0);
    let total_tokens = prompt_tokens.saturating_add(completion_tokens);

    Ok(Some(TokenUsage {
        prompt_tokens,
        completion_tokens,
        cached_tokens: cache_read_tokens,
        total_tokens,
    }))
}

fn parse_stop_reason_value(value: Option<&Value>) -> Result<Option<FinishReason>, LlmError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let reason = value
        .as_str()
        .ok_or_else(|| LlmError::ApiError("stop_reason must be a string".into()))?;
    Ok(Some(parse_stop_reason_str(reason)))
}

fn parse_stop_reason_str(reason: &str) -> FinishReason {
    match reason {
        "max_tokens" => FinishReason::Length,
        "tool_use" => FinishReason::ToolCalls,
        "refusal" => FinishReason::ContentFilter,
        "end_turn" | "stop_sequence" | "pause_turn" => FinishReason::Stop,
        _ => FinishReason::Stop,
    }
}

fn map_reqwest_error(error: reqwest::Error, timeout_ms: u64) -> LlmError {
    if error.is_timeout() {
        LlmError::Timeout { timeout_ms }
    } else {
        LlmError::ApiError(format!("HTTP request failed: {error}"))
    }
}

fn map_status_error(
    status: StatusCode,
    retry_after: Option<&HeaderValue>,
    model: &ModelInfo,
    prompt_tokens: u32,
    body_text: String,
) -> LlmError {
    let error_message = anthropic_error_message(&body_text);
    let error_type = anthropic_error_type(&body_text);

    if status == StatusCode::PAYLOAD_TOO_LARGE || looks_like_context_limit(&error_message) {
        return LlmError::ContextLengthExceeded {
            prompt_tokens,
            max_tokens: model.max_context_tokens,
        };
    }

    match status.as_u16() {
        401 | 403 => LlmError::AuthenticationFailed {
            provider: model.provider_id.clone(),
            reason: error_message,
        },
        404 => LlmError::ModelUnavailable {
            model_id: model.id.clone(),
            reason: error_message,
        },
        408 => LlmError::Timeout {
            timeout_ms: retry_after_to_millis(retry_after).unwrap_or(0),
        },
        429 | 529 => LlmError::RateLimited {
            provider: model.provider_id.clone(),
            retry_after_ms: retry_after_to_millis(retry_after),
        },
        _ if matches!(error_type.as_deref(), Some("overloaded_error")) => LlmError::RateLimited {
            provider: model.provider_id.clone(),
            retry_after_ms: retry_after_to_millis(retry_after),
        },
        _ => LlmError::ApiError(format!("API returned status {status}: {error_message}")),
    }
}

fn anthropic_error_message(body_text: &str) -> String {
    serde_json::from_str::<Value>(body_text)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| body_text.to_string())
}

fn anthropic_error_type(body_text: &str) -> Option<String> {
    serde_json::from_str::<Value>(body_text)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("type"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn looks_like_context_limit(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("prompt is too long")
        || lower.contains("too many tokens")
        || lower.contains("context length")
        || lower.contains("maximum context length")
}

fn anthropic_role_name(role: MessageRole) -> Result<&'static str, LlmError> {
    match role {
        MessageRole::User => Ok("user"),
        MessageRole::Assistant => Ok("assistant"),
        MessageRole::System | MessageRole::Tool => Err(LlmError::Internal(
            "anthropic role helper only supports user/assistant messages".into(),
        )),
    }
}

fn role_from_str(role: &str) -> Result<MessageRole, LlmError> {
    match role {
        "assistant" => Ok(MessageRole::Assistant),
        "user" => Ok(MessageRole::User),
        other => Err(LlmError::ApiError(format!(
            "unsupported anthropic message role '{other}'"
        ))),
    }
}

fn content_part_to_anthropic_block(
    part: &ContentPart,
    allow_files: bool,
) -> Result<Value, LlmError> {
    match part {
        ContentPart::Text { text } => Ok(json!({
            "type": "text",
            "text": text,
        })),
        ContentPart::Image { url, .. } => Ok(json!({
            "type": "image",
            "source": {
                "type": "url",
                "url": url,
            }
        })),
        ContentPart::File {
            filename,
            mime_type,
            data,
        } => {
            if !allow_files {
                return Err(LlmError::ApiError(
                    "anthropic tool_result content currently supports only text/image parts".into(),
                ));
            }

            if mime_type.starts_with("image/") {
                return Ok(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": mime_type,
                        "data": data,
                    }
                }));
            }

            let source_type = if mime_type.starts_with("text/") {
                "text"
            } else {
                "base64"
            };
            let mut block = json!({
                "type": "document",
                "source": {
                    "type": source_type,
                    "media_type": mime_type,
                    "data": data,
                }
            });

            if !filename.is_empty() {
                block["title"] = Value::String(filename.clone());
            }

            Ok(block)
        }
    }
}

fn content_part_to_tool_result_block(part: &ContentPart) -> Result<Value, LlmError> {
    match part {
        ContentPart::Text { text } => Ok(json!({
            "type": "text",
            "text": text,
        })),
        ContentPart::Image { url, .. } => Ok(json!({
            "type": "image",
            "source": {
                "type": "url",
                "url": url,
            }
        })),
        ContentPart::File { .. } => Err(LlmError::ApiError(
            "anthropic tool_result content does not yet support file parts in this abstraction"
                .into(),
        )),
    }
}

fn tool_definition_to_anthropic_json(tool: &ToolDefinition) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters,
    })
}

fn tool_call_to_anthropic_block(tool_call: &ToolCall) -> Result<Value, LlmError> {
    let parsed_arguments = if tool_call.arguments.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&tool_call.arguments).map_err(|error| {
            LlmError::ApiError(format!(
                "anthropic tool calls require JSON object arguments: {error}"
            ))
        })?
    };

    if !parsed_arguments.is_object() {
        return Err(LlmError::ApiError(
            "anthropic tool call arguments must decode to a JSON object".into(),
        ));
    }

    Ok(json!({
        "type": "tool_use",
        "id": tool_call.id,
        "name": tool_call.name,
        "input": parsed_arguments,
    }))
}

fn parse_sse_frame(frame: &str) -> ParsedSseFrame {
    let mut event = None;
    let mut data_lines = Vec::new();

    for line in frame.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.trim().to_string());
        }
    }

    ParsedSseFrame {
        event,
        data: data_lines.join("\n"),
    }
}

fn request_id_for(request: &ChatRequest) -> String {
    request
        .request_id
        .clone()
        .unwrap_or_else(generate_request_id)
}

fn generate_request_id() -> String {
    let sequence = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("anthropic-req-{sequence}")
}

fn retry_after_to_millis(value: Option<&HeaderValue>) -> Option<u64> {
    value
        .and_then(|header| header.to_str().ok())
        .and_then(|text| text.parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1000))
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn u32_field(value: &Value, field: &str) -> Option<u32> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .map(|number| number as u32)
}

fn duration_to_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

struct ParsedSseFrame {
    event: Option<String>,
    data: String,
}

#[derive(Default)]
struct ToolCallStreamState {
    id: Option<String>,
    name: Option<String>,
}

struct StreamState {
    response_id: String,
    content: String,
    usage: Option<TokenUsage>,
    finish_reason: FinishReason,
    prompt_tokens_estimate: u32,
    tool_calls: HashMap<u32, ToolCallStreamState>,
}

impl StreamState {
    fn new(prompt_tokens_estimate: u32) -> Self {
        Self {
            response_id: String::new(),
            content: String::new(),
            usage: None,
            finish_reason: FinishReason::Stop,
            prompt_tokens_estimate,
            tool_calls: HashMap::new(),
        }
    }

    fn finish_event(&self) -> StreamEvent {
        let usage = self.usage.or_else(|| {
            let completion_tokens = crate::estimate_text_tokens(&self.content) as u32;
            Some(TokenUsage {
                prompt_tokens: self.prompt_tokens_estimate,
                completion_tokens,
                cached_tokens: 0,
                total_tokens: self
                    .prompt_tokens_estimate
                    .saturating_add(completion_tokens),
            })
        });

        StreamEvent::Finish {
            reason: self.finish_reason,
            usage,
            response_id: if self.response_id.is_empty() {
                "unknown".into()
            } else {
                self.response_id.clone()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use serde_json::json;
    use tokio_util::sync::CancellationToken;

    use crate::{ChatMessage, ChatRequest, FinishReason, LlmProvider, MessageRole, ModelInfo};

    use super::AnthropicProvider;

    fn test_model() -> ModelInfo {
        let mut model = ModelInfo::new("claude-sonnet-4-20250514", "Claude Sonnet 4", "anthropic");
        model.max_context_tokens = 200_000;
        model.max_output_tokens = 8_192;
        model
    }

    #[test]
    fn build_messages_body_hoists_system_and_maps_tool_calls() {
        let provider = AnthropicProvider::new("https://example.com/v1", "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::System, "You are precise."));
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "Check weather."));
        request.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: crate::MessageContent::Text("Calling tool".into()),
            name: None,
            tool_calls: vec![crate::ToolCall {
                id: "toolu_1".into(),
                name: "get_weather".into(),
                arguments: r#"{"city":"Shanghai"}"#.into(),
            }],
            tool_call_id: None,
            cache_control: None,
        });

        let body = provider
            .build_messages_body(&request, false)
            .unwrap_or_else(|_| panic!("chat body should build successfully"));

        assert_eq!(body["system"][0]["text"], "You are precise.");
        assert_eq!(body["messages"][1]["content"][1]["type"], "tool_use");
        assert_eq!(body["messages"][1]["content"][1]["name"], "get_weather");
    }

    #[tokio::test]
    async fn chat_parses_anthropic_response() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-20250514",
            "content": [
                { "type": "text", "text": "Calling tool" },
                {
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "get_weather",
                    "input": { "city": "Shanghai" }
                }
            ],
            "stop_reason": "tool_use",
            "usage": {
                "input_tokens": 10,
                "cache_read_input_tokens": 2,
                "output_tokens": 5
            }
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/messages");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = AnthropicProvider::new(server.url("/v1"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));
        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "hello"));

        let response = provider.chat(request, CancellationToken::new()).await;
        assert!(response.is_ok(), "chat request should succeed");
        let response = response.unwrap_or_else(|_| panic!("response should be present"));

        mock.assert_async().await;
        assert_eq!(response.id, "msg_123");
        assert_eq!(response.message.content.to_text(), "Calling tool");
        assert_eq!(response.finish_reason, FinishReason::ToolCalls);
        assert_eq!(response.usage.prompt_tokens, 12);
        assert_eq!(response.message.tool_calls.len(), 1);
        assert_eq!(response.message.tool_calls[0].name, "get_weather");
    }

    #[tokio::test]
    async fn stream_parses_text_and_tool_deltas() {
        let server = MockServer::start_async().await;
        let sse_body = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":7,\"output_tokens\":1}}}\n\n",
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n\n",
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_weather\",\"input\":{}}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\":\\\"Shang\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"hai\\\"}\"}}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":7,\"cache_read_input_tokens\":1,\"output_tokens\":4}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n"
        );

        let mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/messages");
                then.status(200)
                    .header("content-type", "text/event-stream")
                    .body(sse_body);
            })
            .await;

        let provider = AnthropicProvider::new(server.url("/v1"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));
        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "hello"));

        let mut rx = provider
            .chat_stream(request, CancellationToken::new())
            .await
            .unwrap_or_else(|_| panic!("stream should start"));

        let mut delta_text = String::new();
        let mut tool_args = String::new();
        let mut finish = None;

        while let Some(event) = rx.recv().await {
            match event {
                crate::StreamEvent::Delta { content, .. } => delta_text.push_str(&content),
                crate::StreamEvent::ToolCallDelta {
                    name,
                    arguments_delta,
                    ..
                } => {
                    assert_eq!(name.as_deref(), Some("get_weather"));
                    tool_args.push_str(&arguments_delta);
                }
                crate::StreamEvent::Finish { reason, usage, .. } => {
                    finish = Some((reason, usage.unwrap_or_default()));
                    break;
                }
                crate::StreamEvent::Error(message) => {
                    panic!("unexpected stream error: {message}");
                }
            }
        }

        mock.assert_async().await;
        assert_eq!(delta_text, "Hello");
        assert_eq!(tool_args, "{\"city\":\"Shanghai\"}");
        let (reason, usage) = finish.unwrap_or_else(|| panic!("finish event should be emitted"));
        assert_eq!(reason, FinishReason::ToolCalls);
        assert_eq!(usage.prompt_tokens, 8);
        assert_eq!(usage.total_tokens, 12);
    }

    #[tokio::test]
    async fn list_models_parses_response() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "data": [
                { "id": "claude-sonnet-4-20250514", "display_name": "Claude Sonnet 4" },
                { "id": "claude-opus-4-20250514", "display_name": "Claude Opus 4" }
            ]
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/models");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = AnthropicProvider::new(server.url("/v1"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let models = provider.list_models(CancellationToken::new()).await;
        assert!(models.is_ok(), "model listing should succeed");
        let models = models.unwrap_or_default();
        mock.assert_async().await;
        assert_eq!(models.len(), 2);
        assert_eq!(models[1].id, "claude-opus-4-20250514");
        assert_eq!(models[1].display_name, "Claude Opus 4");
    }
}
