use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use mc_llm::{CacheControlType, MessageRole, ModelInfo, TokenUsage};
use mc_prompt::{
    calculate_cache_savings, should_set_cache_breakpoint, CacheInvalidationEvent,
    InvalidationReason, PromptCacheError, PromptLayer, PromptLayerManager, TemplateRenderer,
    TurnMessage,
};
use tempfile::tempdir;
use tokio::time::{sleep, timeout};

#[test]
fn prompt_layer_order_matches_priority() {
    assert_eq!(
        PromptLayer::all(),
        [
            PromptLayer::Global,
            PromptLayer::Organization,
            PromptLayer::Project,
            PromptLayer::Session,
            PromptLayer::Turn,
        ]
    );
    assert_eq!(PromptLayer::Global.depth(), 0);
    assert_eq!(PromptLayer::Turn.depth(), 4);
    assert_eq!(PromptLayer::Project.name(), "project");
}

#[tokio::test]
async fn update_layer_broadcasts_versions_and_builds_ordered_messages() {
    let manager = PromptLayerManager::new();
    let mut receiver = manager.subscribe_invalidation();

    let global_version = manager
        .update_layer(
            PromptLayer::Global,
            "global {{language}}",
            HashMap::from([("language".to_string(), "Rust".to_string())]),
        )
        .await
        .expect("global update should succeed");
    assert_eq!(global_version, 1);

    let event = receiver.recv().await.expect("should receive invalidation");
    assert_eq!(event.layer, PromptLayer::Global);
    assert_eq!(event.reason, InvalidationReason::LayerUpdated);
    assert_eq!(event.version, 1);

    manager
        .update_layer(
            PromptLayer::Organization,
            "org {{language}}",
            HashMap::from([("language".to_string(), "Go".to_string())]),
        )
        .await
        .expect("organization update should succeed");
    manager
        .update_layer(PromptLayer::Project, "project", HashMap::new())
        .await
        .expect("project update should succeed");
    manager
        .update_layer(PromptLayer::Session, "session", HashMap::new())
        .await
        .expect("session update should succeed");

    let turn_messages = vec![
        TurnMessage::user("one"),
        TurnMessage::assistant("two"),
        TurnMessage::user("three"),
    ];
    let built = manager
        .build_messages(&turn_messages, &HashMap::new())
        .await
        .expect("message build should succeed");

    assert_eq!(built.len(), 7);
    assert_eq!(built[0].content.to_text(), "global Go");
    assert_eq!(built[1].content.to_text(), "org Go");
    assert_eq!(built[2].content.to_text(), "project");
    assert_eq!(built[3].content.to_text(), "session");
    assert_eq!(built[4].role, MessageRole::User);
    assert_eq!(built[5].role, MessageRole::Assistant);
    assert_eq!(built[6].role, MessageRole::User);
    assert_eq!(
        built[2].cache_control,
        Some(CacheControlType::CacheBreakpoint)
    );
    assert_eq!(built[3].cache_control, None);
}

#[tokio::test]
async fn stale_invalidation_events_are_rejected() {
    let manager = PromptLayerManager::new();
    manager
        .update_layer(PromptLayer::Global, "v1", HashMap::new())
        .await
        .expect("first update should succeed");
    manager
        .update_layer(PromptLayer::Global, "v2", HashMap::new())
        .await
        .expect("second update should succeed");

    let error = manager
        .on_invalidation(CacheInvalidationEvent {
            layer: PromptLayer::Global,
            version: 1,
            global_version: 1,
            reason: InvalidationReason::LayerUpdated,
            timestamp: chrono::Utc::now(),
            path: None,
        })
        .await
        .expect_err("older event should be rejected");

    assert!(matches!(
        error,
        PromptCacheError::StaleEvent {
            layer: PromptLayer::Global,
            event_version: 1,
            current_version: 2,
        }
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_updates_do_not_deadlock() {
    let manager = PromptLayerManager::new();

    let update_one = manager.update_layer(PromptLayer::Organization, "org", HashMap::new());
    let update_two = manager.update_layer(PromptLayer::Project, "project", HashMap::new());
    let update_three = manager.update_layer(PromptLayer::Session, "session", HashMap::new());

    let (org_version, project_version, session_version) =
        tokio::join!(update_one, update_two, update_three);

    assert_eq!(org_version.expect("org update should succeed"), 1);
    assert_eq!(project_version.expect("project update should succeed"), 1);
    assert_eq!(session_version.expect("session update should succeed"), 1);
}

#[test]
fn template_renderer_supports_strict_and_non_strict_modes() {
    let renderer = TemplateRenderer::new();
    let rendered = renderer
        .render(
            "hello {{name}} from {{city}}",
            &HashMap::from([("name".to_string(), "Alice".to_string())]),
        )
        .expect("non-strict render should succeed");
    assert_eq!(rendered, "hello Alice from {{city}}");

    let strict_error = TemplateRenderer::strict()
        .render("hello {{name}}", &HashMap::new())
        .expect_err("strict render should fail");
    assert!(matches!(
        strict_error,
        PromptCacheError::TemplateRenderError(message) if message.contains("name")
    ));
}

#[test]
fn template_renderer_ignores_nested_placeholders() {
    let rendered = TemplateRenderer::strict()
        .render(
            "{{{{name}}}}",
            &HashMap::from([("name".to_string(), "Alice".to_string())]),
        )
        .expect("nested placeholders should be left untouched");
    assert_eq!(rendered, "{{{{name}}}}");
}

#[test]
fn cache_breakpoint_only_depends_on_turn_count() {
    let two_turns = vec![TurnMessage::user("one"), TurnMessage::assistant("two")];
    let three_turns = vec![
        TurnMessage::user("one"),
        TurnMessage::assistant("two"),
        TurnMessage::user("three"),
    ];

    assert!(!should_set_cache_breakpoint(&two_turns));
    assert!(should_set_cache_breakpoint(&three_turns));
}

#[test]
fn cache_savings_uses_ttl_and_hit_rate() {
    let usage = TokenUsage {
        prompt_tokens: 1000,
        completion_tokens: 100,
        cached_tokens: 500,
        total_tokens: 1100,
    };
    let mut model = ModelInfo::new("test-model", "Test Model", "test-provider");
    model.input_price_per_1k = 1.0;
    model.cache_read_price_per_1k = 0.1;

    let short_ttl = calculate_cache_savings(&usage, &model, 300, 5);
    let long_ttl = calculate_cache_savings(&usage, &model, 3600, 5);
    let zero_hit_rate = calculate_cache_savings(
        &TokenUsage {
            cached_tokens: 0,
            ..usage
        },
        &model,
        3600,
        5,
    );

    assert!(short_ttl.0 > 0);
    assert!(long_ttl.0 > short_ttl.0);
    assert_eq!(zero_hit_rate, (0, 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn file_watcher_handles_supported_extensions_and_ignores_txt() {
    let manager = PromptLayerManager::new();
    let temp_dir = tempdir().expect("temporary directory should exist");
    let mut receiver = manager.subscribe_invalidation();

    manager
        .start_file_watcher(&[temp_dir.path().to_path_buf()])
        .expect("watcher should start");

    let markdown_path = temp_dir.path().join("prompt.md");
    let event = write_until_event(&markdown_path, "hello", &mut receiver)
        .await
        .expect("markdown change should broadcast");
    assert_eq!(event.reason, InvalidationReason::FileChanged);
    assert_eq!(event.layer, PromptLayer::Project);

    sleep(Duration::from_millis(200)).await;
    while receiver.try_recv().is_ok() {}

    let ignored_path = temp_dir.path().join("ignore.txt");
    tokio::fs::write(&ignored_path, "ignored")
        .await
        .expect("write should succeed");

    let ignored_event =
        next_file_event_within(&mut receiver, &ignored_path, Duration::from_millis(800)).await;
    assert!(
        ignored_event.is_none(),
        "txt updates should not broadcast, got {ignored_event:?}"
    );
}

async fn next_file_event_within(
    receiver: &mut tokio::sync::broadcast::Receiver<CacheInvalidationEvent>,
    path: &Path,
    deadline: Duration,
) -> Option<CacheInvalidationEvent> {
    timeout(deadline, async move {
        loop {
            match receiver.recv().await {
                Ok(event) if event.path.as_deref() == Some(path) => return Some(event),
                Ok(_) => continue,
                Err(_) => return None,
            }
        }
    })
    .await
    .ok()
    .flatten()
}

async fn write_until_event(
    path: &Path,
    contents: &str,
    receiver: &mut tokio::sync::broadcast::Receiver<CacheInvalidationEvent>,
) -> Option<CacheInvalidationEvent> {
    for attempt in 0..5 {
        sleep(Duration::from_millis(250)).await;
        tokio::fs::write(path, format!("{contents}-{attempt}"))
            .await
            .expect("write should succeed");

        if let Some(event) = next_file_event_within(receiver, path, Duration::from_secs(2)).await {
            return Some(event);
        }
    }

    None
}
