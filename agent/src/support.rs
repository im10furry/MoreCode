use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use mc_llm::{ChatMessage, ChatRequest, FinishReason, LlmProvider, MessageRole, ResponseFormat};

use crate::AgentError;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn complete_json<T>(
    provider: &dyn LlmProvider,
    model_id: &str,
    system_prompt: &str,
    user_prompt: &str,
    schema_name: &str,
    schema: Value,
    temperature: f32,
    max_tokens: u32,
    cancel_token: CancellationToken,
) -> Result<(T, u32), AgentError>
where
    T: DeserializeOwned,
{
    let request = ChatRequest {
        model: Some(model_id.to_string()),
        temperature,
        max_tokens: Some(max_tokens),
        response_format: Some(ResponseFormat::JsonSchema {
            schema,
            name: schema_name.to_string(),
            strict: true,
        }),
        messages: vec![
            ChatMessage::text(MessageRole::System, system_prompt),
            ChatMessage::text(MessageRole::User, user_prompt),
        ],
        ..ChatRequest::default()
    };

    let response = provider
        .chat(request, cancel_token)
        .await
        .map_err(|error| AgentError::LlmError {
            message: error.to_string(),
        })?;

    if response.finish_reason != FinishReason::Stop {
        return Err(AgentError::LlmError {
            message: format!("unexpected finish reason: {:?}", response.finish_reason),
        });
    }

    let content = response.message.content.to_text();
    let parsed =
        serde_json::from_str::<T>(&content).map_err(|error| AgentError::LlmParseError {
            message: error.to_string(),
        })?;

    Ok((parsed, response.usage.total_tokens))
}
