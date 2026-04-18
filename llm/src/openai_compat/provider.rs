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
    CacheCapability, CacheControlType, CacheStrategy, ChatMessage, ChatRequest, ChatResponse,
    ContentPart, FinishReason, ImageDetail, LlmError, LlmProvider, MessageContent, MessageRole,
    ModelInfo, OpenAiCacheStrategy, ResponseFormat, StreamEvent, TokenUsage, ToolCall,
};

use super::OpenAiProviderConfig;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: Arc<str>,
    model: ModelInfo,
    default_headers: HashMap<String, String>,
    cache_capability: CacheCapability,
    active_requests: Arc<RwLock<HashMap<String, CancellationToken>>>,
    request_timeout: Duration,
    stream_buffer_size: usize,
}

impl OpenAiProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: ModelInfo,
    ) -> Result<Self, LlmError> {
        Self::from_config(OpenAiProviderConfig {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model,
            default_headers: HashMap::new(),
            request_timeout: Duration::from_secs(120),
            stream_buffer_size: 64,
        })
    }

    pub fn from_config(config: OpenAiProviderConfig) -> Result<Self, LlmError> {
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
            model: config.model,
            default_headers: config.default_headers,
            cache_capability: CacheCapability {
                supports_prompt_caching: true,
                max_cache_ttl_secs: Some(3600),
                min_cacheable_tokens: 128,
                supported_control_types: vec![
                    CacheControlType::CacheBreakpoint,
                    CacheControlType::Ephemeral,
                ],
                strategy: CacheStrategy::OpenAi(OpenAiCacheStrategy::default()),
            },
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: config.request_timeout,
            stream_buffer_size: config.stream_buffer_size,
        })
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    fn build_chat_body(&self, request: &ChatRequest, stream: bool) -> Result<Value, LlmError> {
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.model.id.clone());
        let request_cache = request
            .cache_control_points
            .iter()
            .map(|point| (point.message_index, point.control_type))
            .collect::<HashMap<_, _>>();

        let mut messages = Vec::with_capacity(request.messages.len());
        for (index, message) in request.messages.iter().enumerate() {
            let mut message_obj = json!({
                "role": role_name(message.role),
                "content": self.content_to_openai_json(&message.content)?,
            });

            if let Some(name) = &message.name {
                message_obj["name"] = Value::String(name.clone());
            }
            if !message.tool_calls.is_empty() {
                message_obj["tool_calls"] = Value::Array(
                    message
                        .tool_calls
                        .iter()
                        .map(tool_call_to_openai_json)
                        .collect(),
                );
            }
            if let Some(tool_call_id) = &message.tool_call_id {
                message_obj["tool_call_id"] = Value::String(tool_call_id.clone());
            }

            let cache_control = message
                .cache_control
                .or_else(|| request_cache.get(&index).copied());
            if let Some(cache_control) = cache_control {
                message_obj["cache_control"] = json!({
                    "type": cache_control_name(cache_control),
                });
            }

            messages.push(message_obj);
        }

        let mut body = json!({
            "model": model,
            "messages": messages,
            "temperature": request.temperature,
            "stream": stream,
        });

        if stream {
            body["stream_options"] = json!({ "include_usage": true });
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if !request.stop_sequences.is_empty() {
            body["stop"] = json!(request.stop_sequences);
        }
        if !request.tools.is_empty() {
            body["tools"] = Value::Array(
                request
                    .tools
                    .iter()
                    .map(tool_definition_to_openai_json)
                    .collect::<Vec<_>>(),
            );
        }
        if let Some(response_format) = &request.response_format {
            body["response_format"] = response_format_to_openai_json(response_format);
        }

        Ok(body)
    }

    fn content_to_openai_json(&self, content: &MessageContent) -> Result<Value, LlmError> {
        match content {
            MessageContent::Text(text) => Ok(Value::String(text.clone())),
            MessageContent::Parts(parts) => Ok(Value::Array(
                parts
                    .iter()
                    .map(content_part_to_openai_json)
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
        let body = self.build_chat_body(request, stream)?;
        let url = format!("{}/chat/completions", self.base_url);
        let timeout = request.timeout.unwrap_or(self.request_timeout);
        let timeout_ms = duration_to_millis(timeout);

        let mut builder = self
            .client
            .post(url)
            .timeout(timeout)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body);

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
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .ok_or_else(|| LlmError::ApiError("missing choices array in chat response".into()))?;
        let choice = choices
            .first()
            .ok_or_else(|| LlmError::ApiError("chat response contains no choices".into()))?;

        let message_value = choice
            .get("message")
            .ok_or_else(|| LlmError::ApiError("chat response missing choice message".into()))?;
        let message = parse_chat_message(message_value)?;
        let finish_reason =
            parse_finish_reason_value(choice.get("finish_reason"))?.unwrap_or(FinishReason::Stop);

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
        let items = value
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        items
            .into_iter()
            .map(|item| {
                let id = string_field(&item, "id").unwrap_or_else(|| self.model.id.clone());
                if id == self.model.id {
                    self.model.clone()
                } else {
                    let mut model = self.model.clone();
                    model.id = id.clone();
                    model.display_name = id;
                    model
                }
            })
            .collect()
    }
}

impl LlmProvider for OpenAiProvider {
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
                        LlmError::ApiError(format!("failed to parse chat response JSON: {error}"))
                    })?;
                    self.parse_chat_response(value, start.elapsed().as_millis() as u64)
                } else {
                    Err(map_status_error(
                        status,
                        headers.get(RETRY_AFTER),
                        &self.model.provider_id,
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
                    &self.model.provider_id,
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
            let builder = self
                .client
                .get(format!("{}/models", self.base_url))
                .timeout(self.request_timeout)
                .header("Authorization", format!("Bearer {}", self.api_key));

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
                    &self.model.provider_id,
                    body_text,
                ));
            }

            let value: Value = serde_json::from_str(&body_text).map_err(|error| {
                LlmError::ApiError(format!("failed to parse models response JSON: {error}"))
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
    let data = frame
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");

    if data.is_empty() {
        return Ok(false);
    }

    if data == "[DONE]" {
        send_stream_event(tx, state.finish_event()).await?;
        return Ok(true);
    }

    let value: Value = serde_json::from_str(&data)
        .map_err(|error| LlmError::StreamError(format!("failed to parse SSE payload: {error}")))?;

    if state.response_id.is_empty() {
        state.response_id = string_field(&value, "id").unwrap_or_default();
    }

    if let Some(usage) = parse_usage(value.get("usage"))? {
        state.usage = Some(usage);
    }

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(delta) = choice.get("delta") {
                if let Some(content) = delta.get("content").and_then(Value::as_str) {
                    if !content.is_empty() {
                        state.content.push_str(content);
                        send_stream_event(
                            tx,
                            StreamEvent::Delta {
                                content: content.to_string(),
                                cumulative_tokens: Some(
                                    crate::estimate_text_tokens(&state.content) as u32,
                                ),
                            },
                        )
                        .await?;
                    }
                }

                if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
                    for tool_call in tool_calls {
                        let index =
                            tool_call.get("index").and_then(Value::as_u64).unwrap_or(0) as u32;
                        let id = string_field(tool_call, "id");
                        let function = tool_call.get("function").cloned().unwrap_or(Value::Null);
                        let name = string_field(&function, "name");
                        let arguments_delta =
                            string_field(&function, "arguments").unwrap_or_default();

                        send_stream_event(
                            tx,
                            StreamEvent::ToolCallDelta {
                                index,
                                id,
                                name,
                                arguments_delta,
                            },
                        )
                        .await?;
                    }
                }
            }

            if let Some(finish_reason) = parse_finish_reason_value(choice.get("finish_reason"))? {
                state.finish_reason = finish_reason;
            }
        }
    }

    Ok(false)
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
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .map(role_from_str)
        .transpose()?
        .unwrap_or(MessageRole::Assistant);

    let content = parse_message_content(value.get("content"))?;
    let name = string_field(value, "name");
    let tool_call_id = string_field(value, "tool_call_id");
    let tool_calls = value
        .get("tool_calls")
        .and_then(Value::as_array)
        .map(|values| parse_tool_calls(values))
        .transpose()?
        .unwrap_or_default();

    Ok(ChatMessage {
        role,
        content,
        name,
        tool_calls,
        tool_call_id,
        cache_control: None,
    })
}

fn parse_tool_calls(values: &[Value]) -> Result<Vec<ToolCall>, LlmError> {
    values
        .iter()
        .map(|value| {
            let function = value.get("function").cloned().unwrap_or(Value::Null);
            let name = string_field(&function, "name")
                .ok_or_else(|| LlmError::ApiError("tool call missing function.name".into()))?;
            Ok(ToolCall {
                id: string_field(value, "id").unwrap_or_else(|| "tool_call".into()),
                name,
                arguments: string_field(&function, "arguments").unwrap_or_default(),
            })
        })
        .collect()
}

fn parse_message_content(value: Option<&Value>) -> Result<MessageContent, LlmError> {
    match value {
        None | Some(Value::Null) => Ok(MessageContent::Text(String::new())),
        Some(Value::String(text)) => Ok(MessageContent::Text(text.clone())),
        Some(Value::Array(parts)) => {
            let mut parsed_parts = Vec::new();
            for part in parts {
                let part_type = string_field(part, "type").unwrap_or_default();
                match part_type.as_str() {
                    "text" | "output_text" => {
                        if let Some(text) = string_field(part, "text") {
                            parsed_parts.push(ContentPart::Text { text });
                        }
                    }
                    "image_url" => {
                        let image_url = part.get("image_url").cloned().unwrap_or(Value::Null);
                        if let Some(url) = string_field(&image_url, "url") {
                            let detail = string_field(&image_url, "detail")
                                .map(|detail| image_detail_from_str(&detail))
                                .transpose()?
                                .unwrap_or(ImageDetail::Auto);
                            parsed_parts.push(ContentPart::Image { url, detail });
                        }
                    }
                    "file" => {
                        let file = part.get("file").cloned().unwrap_or(Value::Null);
                        let filename = string_field(&file, "filename").unwrap_or_default();
                        let mime_type = string_field(&file, "mime_type").unwrap_or_default();
                        let data = string_field(&file, "file_data")
                            .or_else(|| string_field(&file, "data"))
                            .unwrap_or_default();
                        parsed_parts.push(ContentPart::File {
                            filename,
                            mime_type,
                            data,
                        });
                    }
                    _ => {}
                }
            }

            if parsed_parts.is_empty() {
                Ok(MessageContent::Text(String::new()))
            } else {
                Ok(MessageContent::Parts(parsed_parts))
            }
        }
        Some(other) => Ok(MessageContent::Text(other.to_string())),
    }
}

fn parse_usage(value: Option<&Value>) -> Result<Option<TokenUsage>, LlmError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }

    let prompt_tokens = u32_field(value, "prompt_tokens").unwrap_or(0);
    let completion_tokens = u32_field(value, "completion_tokens").unwrap_or(0);
    let cached_tokens = value
        .get("prompt_tokens_details")
        .and_then(|details| details.get("cached_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    let total_tokens =
        u32_field(value, "total_tokens").unwrap_or(prompt_tokens.saturating_add(completion_tokens));

    Ok(Some(TokenUsage {
        prompt_tokens,
        completion_tokens,
        cached_tokens,
        total_tokens,
    }))
}

fn parse_finish_reason_value(value: Option<&Value>) -> Result<Option<FinishReason>, LlmError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let reason = value
        .as_str()
        .ok_or_else(|| LlmError::ApiError("finish_reason must be a string".into()))?;
    Ok(Some(parse_finish_reason_str(reason)?))
}

fn parse_finish_reason_str(reason: &str) -> Result<FinishReason, LlmError> {
    match reason {
        "stop" => Ok(FinishReason::Stop),
        "length" => Ok(FinishReason::Length),
        "tool_calls" => Ok(FinishReason::ToolCalls),
        "content_filter" => Ok(FinishReason::ContentFilter),
        "cancelled" | "canceled" => Ok(FinishReason::Cancelled),
        other => Err(LlmError::ApiError(format!(
            "unsupported finish_reason '{other}'"
        ))),
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
    provider_id: &str,
    body_text: String,
) -> LlmError {
    match status.as_u16() {
        401 | 403 => LlmError::AuthenticationFailed {
            provider: provider_id.to_string(),
            reason: body_text,
        },
        404 => LlmError::ModelUnavailable {
            model_id: provider_id.to_string(),
            reason: body_text,
        },
        408 => LlmError::Timeout {
            timeout_ms: retry_after_to_millis(retry_after).unwrap_or(0),
        },
        413 => LlmError::ContextLengthExceeded {
            prompt_tokens: 0,
            max_tokens: 0,
        },
        429 => LlmError::RateLimited {
            provider: provider_id.to_string(),
            retry_after_ms: retry_after_to_millis(retry_after),
        },
        _ => LlmError::ApiError(format!("API returned status {status}: {body_text}")),
    }
}

fn retry_after_to_millis(value: Option<&HeaderValue>) -> Option<u64> {
    value
        .and_then(|header| header.to_str().ok())
        .and_then(|text| text.parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1000))
}

fn role_name(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}

fn role_from_str(role: &str) -> Result<MessageRole, LlmError> {
    match role {
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "tool" => Ok(MessageRole::Tool),
        other => Err(LlmError::ApiError(format!(
            "unsupported message role '{other}'"
        ))),
    }
}

fn image_detail_from_str(detail: &str) -> Result<ImageDetail, LlmError> {
    match detail {
        "low" => Ok(ImageDetail::Low),
        "high" => Ok(ImageDetail::High),
        "auto" => Ok(ImageDetail::Auto),
        other => Err(LlmError::ApiError(format!(
            "unsupported image detail '{other}'"
        ))),
    }
}

fn content_part_to_openai_json(part: &ContentPart) -> Result<Value, LlmError> {
    match part {
        ContentPart::Text { text } => Ok(json!({
            "type": "text",
            "text": text,
        })),
        ContentPart::Image { url, detail } => Ok(json!({
            "type": "image_url",
            "image_url": {
                "url": url,
                "detail": match detail {
                    ImageDetail::Low => "low",
                    ImageDetail::High => "high",
                    ImageDetail::Auto => "auto",
                }
            }
        })),
        ContentPart::File {
            filename,
            mime_type,
            data,
        } => Ok(json!({
            "type": "file",
            "file": {
                "filename": filename,
                "mime_type": mime_type,
                "file_data": data,
            }
        })),
    }
}

fn tool_definition_to_openai_json(tool: &crate::ToolDefinition) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
            "strict": tool.strict,
        }
    })
}

fn tool_call_to_openai_json(tool_call: &ToolCall) -> Value {
    json!({
        "id": tool_call.id,
        "type": "function",
        "function": {
            "name": tool_call.name,
            "arguments": tool_call.arguments,
        }
    })
}

fn response_format_to_openai_json(response_format: &ResponseFormat) -> Value {
    match response_format {
        ResponseFormat::Text => json!({ "type": "text" }),
        ResponseFormat::JsonObject => json!({ "type": "json_object" }),
        ResponseFormat::JsonSchema {
            schema,
            name,
            strict,
        } => json!({
            "type": "json_schema",
            "json_schema": {
                "name": name,
                "schema": schema,
                "strict": strict,
            }
        }),
    }
}

fn cache_control_name(cache_control: CacheControlType) -> &'static str {
    match cache_control {
        CacheControlType::CacheBreakpoint => "breakpoint",
        CacheControlType::Ephemeral => "ephemeral",
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
    format!("req-{sequence}")
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

struct StreamState {
    response_id: String,
    content: String,
    usage: Option<TokenUsage>,
    finish_reason: FinishReason,
    prompt_tokens_estimate: u32,
}

impl StreamState {
    fn new(prompt_tokens_estimate: u32) -> Self {
        Self {
            response_id: String::new(),
            content: String::new(),
            usage: None,
            finish_reason: FinishReason::Stop,
            prompt_tokens_estimate,
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

    use crate::{CacheControlType, ChatMessage, ChatRequest, LlmProvider, MessageRole, ModelInfo};

    use super::OpenAiProvider;

    fn test_model() -> ModelInfo {
        ModelInfo::new("gpt-4o-mini", "gpt-4o-mini", "openai")
    }

    #[test]
    fn build_chat_body_includes_cache_control_extension() {
        let provider = OpenAiProvider::new("https://example.com/v1", "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let mut request = ChatRequest::default();
        let mut message = ChatMessage::text(MessageRole::User, "hello");
        message.cache_control = Some(CacheControlType::Ephemeral);
        request.messages.push(message);

        let body = provider
            .build_chat_body(&request, false)
            .unwrap_or_else(|_| panic!("chat body should build successfully"));

        assert_eq!(body["messages"][0]["cache_control"]["type"], "ephemeral");
    }

    #[tokio::test]
    async fn chat_parses_openai_compatible_response() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "id": "chatcmpl-123",
            "model": "gpt-4o-mini",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "hello back"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 12,
                "completion_tokens": 4,
                "total_tokens": 16,
                "prompt_tokens_details": {
                    "cached_tokens": 2
                }
            }
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = OpenAiProvider::new(server.url("/v1"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));
        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "hello"));

        let response = provider.chat(request, CancellationToken::new()).await;
        assert!(response.is_ok(), "chat request should succeed");
        let response = response.unwrap_or_else(|_| panic!("response should be present"));

        mock.assert_async().await;
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.message.content.to_text(), "hello back");
        assert_eq!(response.usage.cached_tokens, 2);
    }

    #[tokio::test]
    async fn list_models_parses_response() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "data": [
                { "id": "gpt-4o-mini" },
                { "id": "gpt-4o" }
            ]
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/models");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = OpenAiProvider::new(server.url("/v1"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let models = provider.list_models(CancellationToken::new()).await;
        assert!(models.is_ok(), "model listing should succeed");
        let models = models.unwrap_or_default();
        mock.assert_async().await;
        assert_eq!(models.len(), 2);
        assert_eq!(models[1].id, "gpt-4o");
    }

    #[tokio::test]
    async fn cancelled_chat_returns_cancelled_error() {
        let provider = OpenAiProvider::new("https://example.com/v1", "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));
        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "hello"));

        let cancel_token = CancellationToken::new();
        cancel_token.cancel();

        let result = provider.chat(request, cancel_token).await;
        assert!(matches!(result, Err(crate::LlmError::Cancelled { .. })));
    }
}
