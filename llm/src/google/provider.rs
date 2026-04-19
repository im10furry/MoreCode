use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::header::{HeaderValue, RETRY_AFTER};
use reqwest::{Client, Response, StatusCode};
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::{
    CacheCapability, ChatMessage, ChatRequest, ChatResponse, ContentPart, FinishReason,
    ImageDetail, LlmError, LlmProvider, MessageContent, MessageRole, ModelInfo, ResponseFormat,
    StreamEvent, TokenUsage, ToolCall, ToolDefinition,
};

use super::GoogleProviderConfig;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 8_192;

pub struct GoogleProvider {
    client: Client,
    base_url: String,
    api_key: Arc<str>,
    model: ModelInfo,
    default_headers: HashMap<String, String>,
    cache_capability: CacheCapability,
    active_requests: Arc<RwLock<HashMap<String, CancellationToken>>>,
    request_timeout: Duration,
    stream_buffer_size: usize,
    default_max_output_tokens: u32,
}

impl GoogleProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: ModelInfo,
    ) -> Result<Self, LlmError> {
        let default_max_output_tokens = if model.max_output_tokens > 0 {
            model.max_output_tokens
        } else {
            DEFAULT_MAX_OUTPUT_TOKENS
        };

        Self::from_config(GoogleProviderConfig {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model,
            default_headers: HashMap::new(),
            request_timeout: Duration::from_secs(120),
            stream_buffer_size: 64,
            default_max_output_tokens,
        })
    }

    pub fn from_config(config: GoogleProviderConfig) -> Result<Self, LlmError> {
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
            cache_capability: CacheCapability::default(),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: config.request_timeout,
            stream_buffer_size: config.stream_buffer_size,
            default_max_output_tokens: config.default_max_output_tokens,
        })
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    fn build_generate_content_body(&self, request: &ChatRequest) -> Result<Value, LlmError> {
        self.validate_request_capabilities(request)?;

        let mut system_parts = Vec::new();
        let mut contents = Vec::new();

        for message in &request.messages {
            match message.role {
                MessageRole::System => {
                    if !message.tool_calls.is_empty() || message.tool_call_id.is_some() {
                        return Err(LlmError::ApiError(
                            "google system messages cannot contain tool calls".into(),
                        ));
                    }
                    system_parts.extend(self.message_content_to_google_parts(&message.content)?);
                }
                MessageRole::User => {
                    if !message.tool_calls.is_empty() || message.tool_call_id.is_some() {
                        return Err(LlmError::ApiError(
                            "google user messages cannot contain tool calls".into(),
                        ));
                    }
                    contents.push(json!({
                        "role": "user",
                        "parts": self.message_content_to_google_parts(&message.content)?,
                    }));
                }
                MessageRole::Assistant => {
                    let parts = self.assistant_message_to_google_parts(message)?;
                    contents.push(json!({
                        "role": "model",
                        "parts": parts,
                    }));
                }
                MessageRole::Tool => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [self.tool_message_to_google_part(message)?],
                    }));
                }
            }
        }

        if contents.is_empty() {
            return Err(LlmError::ApiError(
                "google requests require at least one non-system message".into(),
            ));
        }

        let mut body = json!({
            "contents": contents,
        });

        if !system_parts.is_empty() {
            body["systemInstruction"] = json!({
                "parts": system_parts,
            });
        }

        if !request.tools.is_empty() {
            body["tools"] = json!([{
                "functionDeclarations": request
                    .tools
                    .iter()
                    .map(tool_definition_to_google_json)
                    .collect::<Vec<_>>(),
            }]);
        }

        let generation_config = self.build_generation_config(request)?;
        if !generation_config.is_null() {
            body["generationConfig"] = generation_config;
        }

        Ok(body)
    }

    fn build_generation_config(&self, request: &ChatRequest) -> Result<Value, LlmError> {
        let mut config = Map::new();
        config.insert("temperature".into(), json!(request.temperature));
        config.insert(
            "maxOutputTokens".into(),
            json!(request.max_tokens.unwrap_or(self.default_max_output_tokens)),
        );

        if let Some(top_p) = request.top_p {
            config.insert("topP".into(), json!(top_p));
        }
        if !request.stop_sequences.is_empty() {
            config.insert("stopSequences".into(), json!(request.stop_sequences));
        }

        if let Some(response_format) = &request.response_format {
            match response_format {
                ResponseFormat::Text => {
                    config.insert("responseMimeType".into(), json!("text/plain"));
                }
                ResponseFormat::JsonObject => {
                    config.insert("responseMimeType".into(), json!("application/json"));
                }
                ResponseFormat::JsonSchema { schema, .. } => {
                    config.insert("responseMimeType".into(), json!("application/json"));
                    config.insert("responseJsonSchema".into(), schema.clone());
                }
            }
        }

        if config.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(Value::Object(config))
        }
    }

    fn validate_request_capabilities(&self, request: &ChatRequest) -> Result<(), LlmError> {
        if request.messages.is_empty() {
            return Err(LlmError::ApiError(
                "google requests require at least one message".into(),
            ));
        }

        if !request.cache_control_points.is_empty()
            || request
                .messages
                .iter()
                .any(|message| message.cache_control.is_some())
        {
            return Err(LlmError::ApiError(
                "google provider draft does not yet map prompt cache control points".into(),
            ));
        }

        Ok(())
    }

    fn assistant_message_to_google_parts(
        &self,
        message: &ChatMessage,
    ) -> Result<Vec<Value>, LlmError> {
        if message.tool_call_id.is_some() {
            return Err(LlmError::ApiError(
                "google assistant messages cannot contain tool_call_id".into(),
            ));
        }

        let mut parts = self.message_content_to_google_parts(&message.content)?;
        for tool_call in &message.tool_calls {
            parts.push(tool_call_to_google_part(tool_call)?);
        }

        if parts.is_empty() {
            parts.push(json!({ "text": "" }));
        }

        Ok(parts)
    }

    fn tool_message_to_google_part(&self, message: &ChatMessage) -> Result<Value, LlmError> {
        let name = message.name.clone().unwrap_or_else(|| "tool_result".into());
        let response = tool_message_response_value(&message.content);
        let mut function_response = json!({
            "name": name,
            "response": response,
        });

        if let Some(id) = &message.tool_call_id {
            function_response["id"] = Value::String(id.clone());
        }

        let media_parts = tool_message_media_parts(&message.content)?;
        if !media_parts.is_empty() {
            function_response["parts"] = Value::Array(media_parts);
        }

        Ok(json!({
            "functionResponse": function_response,
        }))
    }

    fn message_content_to_google_parts(
        &self,
        content: &MessageContent,
    ) -> Result<Vec<Value>, LlmError> {
        match content {
            MessageContent::Text(text) => Ok(vec![json!({ "text": text })]),
            MessageContent::Parts(parts) => parts.iter().map(content_part_to_google_part).collect(),
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
        let body = self.build_generate_content_body(request)?;
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.model.id.clone());
        let url = if stream {
            format!(
                "{}/{}:streamGenerateContent?alt=sse",
                self.base_url,
                normalize_model_path(&model)
            )
        } else {
            format!(
                "{}/{}:generateContent",
                self.base_url,
                normalize_model_path(&model)
            )
        };
        let timeout = request.timeout.unwrap_or(self.request_timeout);
        let timeout_ms = duration_to_millis(timeout);

        let mut builder = self
            .client
            .post(url)
            .timeout(timeout)
            .header("x-goog-api-key", self.api_key.as_ref())
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
        let candidates = value
            .get("candidates")
            .and_then(Value::as_array)
            .ok_or_else(|| LlmError::ApiError("google response missing candidates array".into()))?;
        let candidate = candidates.first().ok_or_else(|| {
            prompt_feedback_error(&value).unwrap_or_else(|| {
                LlmError::ApiError("google response does not contain any candidates".into())
            })
        })?;

        let message = parse_chat_message(candidate)?;
        let usage = parse_usage(value.get("usageMetadata"))?.unwrap_or_default();
        let mut finish_reason = candidate
            .get("finishReason")
            .map(parse_finish_reason_value)
            .unwrap_or(FinishReason::Stop);
        if matches!(finish_reason, FinishReason::Stop) && !message.tool_calls.is_empty() {
            finish_reason = FinishReason::ToolCalls;
        }

        Ok(ChatResponse {
            id: string_field(&value, "responseId").unwrap_or_else(|| "unknown".into()),
            model: string_field(&value, "modelVersion").unwrap_or_else(|| self.model.id.clone()),
            message,
            usage,
            finish_reason,
            latency_ms,
            raw_response: Some(value),
        })
    }

    fn parse_models_response(&self, value: Value) -> Vec<ModelInfo> {
        value
            .get("models")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| supports_generate_content(item))
            .map(|item| {
                let id = string_field(&item, "baseModelId")
                    .or_else(|| string_field(&item, "name").map(strip_model_prefix))
                    .unwrap_or_else(|| self.model.id.clone());
                let mut model = self.model.clone();
                model.id = id.clone();
                model.display_name =
                    string_field(&item, "displayName").unwrap_or_else(|| id.clone());
                model.provider_id = self.model.provider_id.clone();
                model.max_output_tokens =
                    u32_field(&item, "outputTokenLimit").unwrap_or(model.max_output_tokens);
                model.max_context_tokens =
                    u32_field(&item, "inputTokenLimit").unwrap_or(model.max_context_tokens);
                model
            })
            .collect()
    }
}

impl LlmProvider for GoogleProvider {
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
                            "failed to parse google chat response JSON: {error}"
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
                let result =
                    pump_sse_stream(response, tx.clone(), request_cancel, prompt_tokens_estimate)
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
                .header("x-goog-api-key", self.api_key.as_ref());

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
                    "failed to parse google models response JSON: {error}"
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
                            process_sse_frame(&frame, &mut state, &tx).await?;
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
        process_sse_frame(&buffer, &mut state, &tx).await?;
    }

    send_stream_event(&tx, state.finish_event()).await?;
    Ok(())
}

async fn process_sse_frame(
    frame: &str,
    state: &mut StreamState,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), LlmError> {
    let data = frame
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");

    if data.is_empty() {
        return Ok(());
    }

    let value: Value = serde_json::from_str(&data)
        .map_err(|error| LlmError::StreamError(format!("failed to parse SSE payload: {error}")))?;

    if state.response_id.is_empty() {
        state.response_id = string_field(&value, "responseId").unwrap_or_default();
    }
    if let Some(usage) = parse_usage(value.get("usageMetadata"))? {
        state.usage = Some(usage);
    }

    if let Some(error) = prompt_feedback_error(&value) {
        return Err(error);
    }

    if let Some(candidates) = value.get("candidates").and_then(Value::as_array) {
        for candidate in candidates {
            if let Some(finish_reason) =
                candidate.get("finishReason").map(parse_finish_reason_value)
            {
                state.finish_reason = finish_reason;
            }

            if let Some(parts) = candidate
                .get("content")
                .and_then(|content| content.get("parts"))
                .and_then(Value::as_array)
            {
                for part in parts {
                    if let Some(text) = string_field(part, "text") {
                        if !text.is_empty() {
                            state.content.push_str(&text);
                            send_stream_event(
                                tx,
                                StreamEvent::Delta {
                                    content: text,
                                    cumulative_tokens: Some(crate::estimate_text_tokens(
                                        &state.content,
                                    )
                                        as u32),
                                },
                            )
                            .await?;
                        }
                    }

                    if let Some(function_call) = part.get("functionCall") {
                        let id = string_field(function_call, "id");
                        let name = string_field(function_call, "name");
                        let index = state
                            .resolve_tool_index(id.as_deref(), u32_field(function_call, "index"));
                        let arguments = serialize_args(function_call.get("args"))?;
                        let delta = state.update_tool_arguments(index, &arguments);
                        if !delta.is_empty() {
                            state.finish_reason = FinishReason::ToolCalls;
                            send_stream_event(
                                tx,
                                StreamEvent::ToolCallDelta {
                                    index,
                                    id,
                                    name,
                                    arguments_delta: delta,
                                },
                            )
                            .await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
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

fn parse_chat_message(candidate: &Value) -> Result<ChatMessage, LlmError> {
    let content = candidate
        .get("content")
        .ok_or_else(|| LlmError::ApiError("google candidate missing content".into()))?;
    let parts = content
        .get("parts")
        .and_then(Value::as_array)
        .ok_or_else(|| LlmError::ApiError("google candidate content missing parts".into()))?;
    let role = content
        .get("role")
        .and_then(Value::as_str)
        .map(role_from_str)
        .transpose()?
        .unwrap_or(MessageRole::Assistant);

    let mut text_segments = Vec::new();
    let mut tool_calls = Vec::new();

    for part in parts {
        if let Some(text) = string_field(part, "text") {
            text_segments.push(text);
        }

        if let Some(function_call) = part.get("functionCall") {
            let name = string_field(function_call, "name")
                .ok_or_else(|| LlmError::ApiError("functionCall missing name".into()))?;
            let arguments = serialize_args(function_call.get("args"))?;
            tool_calls.push(ToolCall {
                id: string_field(function_call, "id").unwrap_or_else(|| "function_call".into()),
                name,
                arguments,
            });
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

    let prompt_tokens = u32_field(value, "promptTokenCount").unwrap_or(0);
    let completion_tokens = u32_field(value, "candidatesTokenCount").unwrap_or(0);
    let cached_tokens = u32_field(value, "cachedContentTokenCount").unwrap_or(0);
    let total_tokens = u32_field(value, "totalTokenCount")
        .unwrap_or(prompt_tokens.saturating_add(completion_tokens));

    Ok(Some(TokenUsage {
        prompt_tokens,
        completion_tokens,
        cached_tokens,
        total_tokens,
    }))
}

fn prompt_feedback_error(value: &Value) -> Option<LlmError> {
    let prompt_feedback = value.get("promptFeedback")?;
    let block_reason = string_field(prompt_feedback, "blockReason")?;
    Some(LlmError::ApiError(format!(
        "google prompt blocked: {block_reason}"
    )))
}

fn parse_finish_reason_value(value: &Value) -> FinishReason {
    match value.as_str().unwrap_or_default() {
        "MAX_TOKENS" => FinishReason::Length,
        "SAFETY" | "RECITATION" | "SPII" | "PROHIBITED_CONTENT" => FinishReason::ContentFilter,
        "MALFORMED_FUNCTION_CALL" => FinishReason::ToolCalls,
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
    let error_message = google_error_message(&body_text);
    let error_status = google_error_status(&body_text);

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
        429 => LlmError::RateLimited {
            provider: model.provider_id.clone(),
            retry_after_ms: retry_after_to_millis(retry_after),
        },
        _ if matches!(error_status.as_deref(), Some("RESOURCE_EXHAUSTED")) => {
            LlmError::RateLimited {
                provider: model.provider_id.clone(),
                retry_after_ms: retry_after_to_millis(retry_after),
            }
        }
        _ => LlmError::ApiError(format!("API returned status {status}: {error_message}")),
    }
}

fn google_error_message(body_text: &str) -> String {
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

fn google_error_status(body_text: &str) -> Option<String> {
    serde_json::from_str::<Value>(body_text)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("status"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn looks_like_context_limit(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("too many tokens")
        || lower.contains("context length")
        || lower.contains("token limit")
        || lower.contains("request is too large")
}

fn supports_generate_content(item: &Value) -> bool {
    item.get("supportedGenerationMethods")
        .and_then(Value::as_array)
        .map(|methods| {
            methods.iter().any(|method| {
                method
                    .as_str()
                    .map(|value| value.eq_ignore_ascii_case("generateContent"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(true)
}

fn role_from_str(role: &str) -> Result<MessageRole, LlmError> {
    match role {
        "model" => Ok(MessageRole::Assistant),
        "user" => Ok(MessageRole::User),
        other => Err(LlmError::ApiError(format!(
            "unsupported google message role '{other}'"
        ))),
    }
}

fn content_part_to_google_part(part: &ContentPart) -> Result<Value, LlmError> {
    match part {
        ContentPart::Text { text } => Ok(json!({ "text": text })),
        ContentPart::Image { url, detail } => image_url_to_google_part(url, *detail),
        ContentPart::File {
            filename,
            mime_type,
            data,
        } => file_data_to_google_part(filename, mime_type, data),
    }
}

fn image_url_to_google_part(url: &str, detail: ImageDetail) -> Result<Value, LlmError> {
    let (mime_type, data) = parse_data_url(url).ok_or_else(|| {
        LlmError::ApiError(
            "google provider only supports image data URLs or File parts for multimodal input"
                .into(),
        )
    })?;

    let mut inline_data = json!({
        "mimeType": mime_type,
        "data": data,
    });

    inline_data["displayName"] = Value::String(match detail {
        ImageDetail::Low => "image-low".into(),
        ImageDetail::High => "image-high".into(),
        ImageDetail::Auto => "image-auto".into(),
    });

    Ok(json!({ "inlineData": inline_data }))
}

fn file_data_to_google_part(
    filename: &str,
    mime_type: &str,
    data: &str,
) -> Result<Value, LlmError> {
    if mime_type.trim().is_empty() {
        return Err(LlmError::ApiError(
            "google file parts require a mime_type".into(),
        ));
    }

    let mut inline_data = json!({
        "mimeType": mime_type,
        "data": normalize_file_payload(data),
    });

    if !filename.is_empty() {
        inline_data["displayName"] = Value::String(filename.to_string());
    }

    Ok(json!({ "inlineData": inline_data }))
}

fn tool_message_response_value(content: &MessageContent) -> Value {
    let text = content.to_text();
    if text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&text)
            .ok()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({ "result": text }))
    }
}

fn tool_message_media_parts(content: &MessageContent) -> Result<Vec<Value>, LlmError> {
    match content {
        MessageContent::Text(_) => Ok(Vec::new()),
        MessageContent::Parts(parts) => parts
            .iter()
            .filter(|part| !matches!(part, ContentPart::Text { .. }))
            .map(content_part_to_google_part)
            .collect(),
    }
}

fn tool_definition_to_google_json(tool: &ToolDefinition) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters,
    })
}

fn tool_call_to_google_part(tool_call: &ToolCall) -> Result<Value, LlmError> {
    let parsed_arguments = if tool_call.arguments.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str::<Value>(&tool_call.arguments).map_err(|error| {
            LlmError::ApiError(format!(
                "google tool calls require JSON object arguments: {error}"
            ))
        })?
    };

    if !parsed_arguments.is_object() {
        return Err(LlmError::ApiError(
            "google tool call arguments must decode to a JSON object".into(),
        ));
    }

    let mut function_call = json!({
        "name": tool_call.name,
        "args": parsed_arguments,
    });

    if !tool_call.id.is_empty() {
        function_call["id"] = Value::String(tool_call.id.clone());
    }

    Ok(json!({ "functionCall": function_call }))
}

fn normalize_model_path(model: &str) -> String {
    if model.starts_with("models/") {
        model.to_string()
    } else {
        format!("models/{model}")
    }
}

fn strip_model_prefix(model: String) -> String {
    model
        .strip_prefix("models/")
        .map(ToOwned::to_owned)
        .unwrap_or(model)
}

fn parse_data_url(url: &str) -> Option<(String, String)> {
    let data_url = url.strip_prefix("data:")?;
    let (meta, data) = data_url.split_once(',')?;
    let mime_type = meta
        .split(';')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream");

    Some((mime_type.to_string(), data.to_string()))
}

fn normalize_file_payload(data: &str) -> String {
    parse_data_url(data)
        .map(|(_, payload)| payload)
        .unwrap_or_else(|| data.to_string())
}

fn serialize_args(value: Option<&Value>) -> Result<String, LlmError> {
    let value = value.cloned().unwrap_or_else(|| json!({}));
    serde_json::to_string(&value)
        .map_err(|error| LlmError::ApiError(format!("failed to serialize function args: {error}")))
}

fn shared_prefix_len(left: &str, right: &str) -> usize {
    let mut prefix = 0usize;
    for ((left_index, left_char), (right_index, right_char)) in
        left.char_indices().zip(right.char_indices())
    {
        if left_char != right_char {
            break;
        }
        prefix = left_index + left_char.len_utf8();
        if left_index != right_index {
            break;
        }
    }
    prefix
}

fn request_id_for(request: &ChatRequest) -> String {
    request
        .request_id
        .clone()
        .unwrap_or_else(generate_request_id)
}

fn generate_request_id() -> String {
    let sequence = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("google-req-{sequence}")
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

struct StreamState {
    response_id: String,
    content: String,
    usage: Option<TokenUsage>,
    finish_reason: FinishReason,
    prompt_tokens_estimate: u32,
    tool_arguments: HashMap<u32, String>,
    tool_ids: HashMap<String, u32>,
    next_tool_index: u32,
}

impl StreamState {
    fn new(prompt_tokens_estimate: u32) -> Self {
        Self {
            response_id: String::new(),
            content: String::new(),
            usage: None,
            finish_reason: FinishReason::Stop,
            prompt_tokens_estimate,
            tool_arguments: HashMap::new(),
            tool_ids: HashMap::new(),
            next_tool_index: 0,
        }
    }

    fn resolve_tool_index(&mut self, id: Option<&str>, explicit_index: Option<u32>) -> u32 {
        if let Some(index) = explicit_index {
            if let Some(id) = id {
                self.tool_ids.insert(id.to_string(), index);
            }
            self.next_tool_index = self.next_tool_index.max(index.saturating_add(1));
            return index;
        }

        if let Some(id) = id {
            if let Some(index) = self.tool_ids.get(id).copied() {
                return index;
            }
        }

        let index = self.next_tool_index;
        self.next_tool_index = self.next_tool_index.saturating_add(1);
        if let Some(id) = id {
            self.tool_ids.insert(id.to_string(), index);
        }
        index
    }

    fn update_tool_arguments(&mut self, index: u32, current: &str) -> String {
        let previous = self.tool_arguments.entry(index).or_default();
        if previous == current {
            return String::new();
        }

        let prefix_len = shared_prefix_len(previous, current);
        let delta = current[prefix_len..].to_string();
        *previous = current.to_string();
        delta
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

    use super::GoogleProvider;

    fn test_model() -> ModelInfo {
        let mut model = ModelInfo::new("gemini-2.5-flash", "Gemini 2.5 Flash", "google");
        model.max_context_tokens = 1_048_576;
        model.max_output_tokens = 8_192;
        model.supports_json_mode = true;
        model.supports_tools = true;
        model.supports_vision = true;
        model
    }

    #[test]
    fn build_generate_content_body_maps_system_tools_and_json_schema() {
        let provider = GoogleProvider::new("https://example.com/v1beta", "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::System, "Be concise."));
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "What's the weather?"));
        request.tools.push(crate::ToolDefinition {
            name: "get_weather".into(),
            description: "Fetch weather".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "city": { "type": "string" }
                }
            }),
            strict: true,
        });
        request.response_format = Some(crate::ResponseFormat::JsonSchema {
            schema: json!({
                "type": "object",
                "properties": {
                    "answer": { "type": "string" }
                }
            }),
            name: "weather_response".into(),
            strict: true,
        });

        let body = provider
            .build_generate_content_body(&request)
            .unwrap_or_else(|_| panic!("body build should succeed"));

        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "Be concise.");
        assert_eq!(
            body["tools"][0]["functionDeclarations"][0]["name"],
            "get_weather"
        );
        assert_eq!(
            body["generationConfig"]["responseMimeType"],
            "application/json"
        );
        assert_eq!(
            body["generationConfig"]["responseJsonSchema"]["type"],
            "object"
        );
    }

    #[tokio::test]
    async fn chat_parses_google_response() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "responseId": "resp_123",
            "modelVersion": "gemini-2.5-flash",
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        { "text": "Calling tool" },
                        {
                            "functionCall": {
                                "id": "fc_1",
                                "name": "get_weather",
                                "args": { "city": "Shanghai" }
                            }
                        }
                    ]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "cachedContentTokenCount": 2,
                "candidatesTokenCount": 5,
                "totalTokenCount": 15
            }
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/v1beta/models/gemini-2.5-flash:generateContent");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = GoogleProvider::new(server.url("/v1beta"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));
        let mut request = ChatRequest::default();
        request
            .messages
            .push(ChatMessage::text(MessageRole::User, "hello"));

        let response = provider.chat(request, CancellationToken::new()).await;
        assert!(response.is_ok(), "chat request should succeed");
        let response = response.unwrap_or_else(|_| panic!("response should be present"));

        mock.assert_async().await;
        assert_eq!(response.id, "resp_123");
        assert_eq!(response.model, "gemini-2.5-flash");
        assert_eq!(response.message.content.to_text(), "Calling tool");
        assert_eq!(response.finish_reason, FinishReason::ToolCalls);
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.message.tool_calls[0].name, "get_weather");
    }

    #[tokio::test]
    async fn stream_parses_text_and_tool_call_updates() {
        let server = MockServer::start_async().await;
        let sse_body = concat!(
            "data: {\"responseId\":\"resp_stream\",\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"Hel\"}]}}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"lo\"}]}}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"functionCall\":{\"id\":\"fc_1\",\"name\":\"get_weather\",\"args\":{\"city\":\"Shanghai\"}}}],\"role\":\"model\"},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":7,\"cachedContentTokenCount\":1,\"candidatesTokenCount\":4,\"totalTokenCount\":11}}\n\n"
        );

        let mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/v1beta/models/gemini-2.5-flash:streamGenerateContent")
                    .query_param("alt", "sse");
                then.status(200)
                    .header("content-type", "text/event-stream")
                    .body(sse_body);
            })
            .await;

        let provider = GoogleProvider::new(server.url("/v1beta"), "test-key", test_model())
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
        assert_eq!(usage.prompt_tokens, 7);
        assert_eq!(usage.total_tokens, 11);
    }

    #[tokio::test]
    async fn list_models_filters_to_generate_content_models() {
        let server = MockServer::start_async().await;
        let response_body = json!({
            "models": [
                {
                    "name": "models/gemini-2.5-flash",
                    "baseModelId": "gemini-2.5-flash",
                    "displayName": "Gemini 2.5 Flash",
                    "inputTokenLimit": 1048576,
                    "outputTokenLimit": 8192,
                    "supportedGenerationMethods": ["generateContent"]
                },
                {
                    "name": "models/embedding-001",
                    "displayName": "Embedding",
                    "supportedGenerationMethods": ["embedContent"]
                }
            ]
        });

        let mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1beta/models");
                then.status(200).json_body(response_body);
            })
            .await;

        let provider = GoogleProvider::new(server.url("/v1beta"), "test-key", test_model())
            .unwrap_or_else(|_| panic!("provider construction should succeed"));

        let models = provider.list_models(CancellationToken::new()).await;
        assert!(models.is_ok(), "model listing should succeed");
        let models = models.unwrap_or_default();
        mock.assert_async().await;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gemini-2.5-flash");
        assert_eq!(models[0].display_name, "Gemini 2.5 Flash");
    }
}
