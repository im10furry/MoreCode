use std::sync::Arc;

use async_trait::async_trait;
use mc_context::{
    AutoCompactor, ChatMessage, CompactSummary, CompressionCoordinator, ContextCompressor,
    ContextPool, LlmClient, LlmError, LlmRequest, LlmResponse, MessageRole, PlatformInfo,
    SimpleTokenCounter,
};

#[derive(Debug)]
struct MockLlm;

#[async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, _request: LlmRequest) -> std::result::Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: serde_json::to_string(&CompactSummary {
                user_request: "Implement mc-context".into(),
                completed_work: vec!["Built context layers".into()],
                current_state: "Compressing history".into(),
                key_decisions: vec!["Prefer rule-first compaction".into()],
                error_info: vec![],
                continuation_context: "Run cargo test".into(),
            })
            .unwrap(),
        })
    }
}

#[tokio::test]
async fn platform_context_flows_into_pool_and_compression_pipeline() {
    let pool = ContextPool::new(Arc::new(PlatformInfo::default()), 50_000);
    pool.update_project_knowledge("Project knowledge".into())
        .await;
    pool.update_task_context("Task focus".into()).await;
    pool.set_agent_context("coder".into(), "Agent-specific notes".into())
        .await;

    let context = pool.build_context_for_agent("coder", "Coder").await;
    assert!(context.contains("## Platform Environment"));
    assert!(context.contains("## Project Knowledge"));
    assert!(context.contains("## Task Context"));
    assert!(context.contains("## Coder Specific Context"));

    let coordinator = CompressionCoordinator::default().with_l2(
        AutoCompactor::default()
            .with_llm_client(Arc::new(MockLlm))
            .with_max_context_tokens(1_000),
    );
    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: context,
            turn: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: MessageRole::Tool,
            content: "x".repeat(12_000),
            turn: Some(1),
            tool_call_id: Some("call-tool".into()),
        },
        ChatMessage {
            role: MessageRole::User,
            content: "x".repeat(4_000),
            turn: Some(2),
            tool_call_id: None,
        },
    ];

    let result = coordinator
        .compress(&messages, 8, &SimpleTokenCounter, 1_000)
        .await
        .unwrap();

    assert!(result.level >= 1);
    assert!(result
        .messages
        .iter()
        .any(|message| message.content.contains("[Context Compacted]")));
}
