use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mc_memory::{
    compact_core_memory, ConversationEntry, ConversationRole, ConversationStore, CoreMemory,
    CoreMemoryManager, KnowledgeEntry, KnowledgeSource, KnowledgeStore, MemoryBlock,
    MemoryCategory, MemoryError, MemoryManager, MemoryManagerTrait, MemoryUpdate,
    PreferenceManager, PreferenceObservation, RiskAreas, RuleEnforcer, RuleScope, RuleSource,
    RuleType, SearchQuery, SqliteConversationStore, SqliteKnowledgeStore, UserRule,
};
use tempfile::TempDir;
use tokio::{
    fs,
    sync::{Mutex, Notify, RwLock},
    time::{timeout, Duration},
};

#[derive(Default)]
struct MockKnowledgeStore {
    started: Notify,
    release: Notify,
    stored_ids: Mutex<Vec<String>>,
}

#[async_trait]
impl KnowledgeStore for MockKnowledgeStore {
    async fn store(&self, entry: KnowledgeEntry) -> Result<String, MemoryError> {
        self.started.notify_waiters();
        self.release.notified().await;
        self.stored_ids.lock().await.push(entry.id.clone());
        Ok(entry.id)
    }

    async fn retrieve(&self, _id: &str) -> Result<Option<KnowledgeEntry>, MemoryError> {
        Ok(None)
    }

    async fn search(
        &self,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<KnowledgeEntry>, MemoryError> {
        Ok(Vec::new())
    }

    async fn delete(&self, _id: &str) -> Result<bool, MemoryError> {
        Ok(false)
    }
}

#[tokio::test]
async fn agent_updates_are_serialized_through_single_writer_queue() {
    let temp = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp.path().join("src"))
        .await
        .expect("create src");
    fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n")
        .await
        .expect("seed project");

    let (manager, rx) = MemoryManager::with_channel(temp.path(), 64);
    let manager = Arc::new(manager);
    let write_loop = tokio::spawn(manager.clone().run_write_loop(rx));

    let mut tasks = Vec::new();
    for index in 0..8 {
        let manager = manager.clone();
        tasks.push(tokio::spawn(async move {
            manager
                .agent_update(MemoryUpdate::RiskDiscovered {
                    area: format!("src/lib.rs:{index}"),
                    r#type: "consistency".to_string(),
                    description: format!("issue-{index}"),
                    severity: "medium".to_string(),
                })
                .await
        }));
    }

    for task in tasks {
        task.await.expect("join").expect("queue write");
    }

    let risks_path = temp
        .path()
        .join(".assistant-memory")
        .join("risk-areas.json");
    let risks: RiskAreas = serde_json::from_str(
        &fs::read_to_string(risks_path)
            .await
            .expect("read serialized risks"),
    )
    .expect("deserialize risks");
    assert_eq!(risks.risks.len(), 8);

    write_loop.abort();
}

#[tokio::test]
async fn compact_core_memory_releases_lock_before_archival_write() {
    let mut core = CoreMemory::new(200);
    for (suffix, priority) in [("low-a", 9), ("low-b", 8), ("task", 1), ("ctx", 5)] {
        let mut block = MemoryBlock::new(
            format!("block:{suffix}"),
            MemoryCategory::Context,
            "x".repeat(160),
            "tester",
        );
        block.priority = priority;
        core.upsert_block(block).expect("insert block");
    }

    let core = Arc::new(RwLock::new(core));
    let store = Arc::new(MockKnowledgeStore::default());
    let store_trait: Arc<dyn KnowledgeStore> = store.clone();

    let compact_task = tokio::spawn({
        let core = core.clone();
        let store = store_trait.clone();
        async move { compact_core_memory(&core, &store).await }
    });

    store.started.notified().await;
    let read_guard = timeout(Duration::from_millis(100), core.read())
        .await
        .expect("core lock should already be released");
    assert!(read_guard.current_tokens() <= 120);
    drop(read_guard);

    store.release.notify_waiters();
    let compacted = compact_task.await.expect("join").expect("compact");
    assert_eq!(compacted, 1);
    assert_eq!(store.stored_ids.lock().await.len(), 1);
}

#[tokio::test]
async fn recall_memory_searches_via_fts_and_filters() {
    let temp = TempDir::new().expect("temp dir");
    let store = SqliteConversationStore::new(&temp.path().join("recall.db"))
        .await
        .expect("create recall store");

    store
        .append(ConversationEntry {
            id: "1".to_string(),
            session_id: "alpha".to_string(),
            role: ConversationRole::User,
            content: "rust async memory".to_string(),
            tokens_used: 12,
            timestamp: Utc::now(),
            tags: vec!["memory".to_string()],
        })
        .await
        .expect("append entry");
    store
        .append(ConversationEntry {
            id: "2".to_string(),
            session_id: "beta".to_string(),
            role: ConversationRole::Tool,
            content: "python script".to_string(),
            tokens_used: 6,
            timestamp: Utc::now(),
            tags: vec!["tool".to_string()],
        })
        .await
        .expect("append second entry");

    let results = store
        .search(&SearchQuery {
            keyword: Some("memory".to_string()),
            session_id: Some("alpha".to_string()),
            role: Some(ConversationRole::User),
            start_time: None,
            end_time: None,
            limit: 10,
            offset: 0,
        })
        .await
        .expect("fts search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "1");
}

#[tokio::test]
async fn archival_store_round_trips_search_rules_and_preferences() {
    let temp = TempDir::new().expect("temp dir");
    let store = SqliteKnowledgeStore::new(&temp.path().join("letta"))
        .await
        .expect("create archival store");

    store
        .store(KnowledgeEntry {
            id: "entry-1".to_string(),
            title: "Memory design".to_string(),
            content: "Rust agent memory with tantivy search".to_string(),
            category: "design".to_string(),
            source: KnowledgeSource::Manual,
            tags: vec!["memory".to_string(), "tantivy".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            embedding: None,
            entities: vec!["Rust".to_string()],
        })
        .await
        .expect("store entry");

    let hits = store.search("tantivy", 5).await.expect("search entry");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "entry-1");

    let preferences = PreferenceManager::new(temp.path());
    let profile = preferences
        .record_sideband_observations(&[PreferenceObservation {
            key: "output_format".to_string(),
            value: "markdown".to_string(),
            confidence_delta: 0.8,
            evidence: "user selected markdown response".to_string(),
            observed_at: Utc::now(),
        }])
        .await
        .expect("record sideband preference");
    assert_eq!(profile.preferences.len(), 1);
    assert!(preferences
        .render_prompt_block()
        .await
        .expect("render preference block")
        .contains("output_format = markdown"));

    let rules = vec![UserRule {
        id: "rule-1".to_string(),
        description: "禁止出现 forbidden".to_string(),
        rule_type: RuleType::ForbiddenWords {
            words: vec!["forbidden".to_string()],
            case_sensitive: false,
        },
        scope: RuleScope::Project,
        created_at: Utc::now(),
        source: RuleSource::Manual,
        enabled: true,
    }];
    let (prompt, validation) =
        RuleEnforcer::apply_dual_guard("base system prompt", "contains forbidden token", &rules);
    assert!(prompt.contains("用户规则"));
    assert!(!validation.passed);
    assert_eq!(validation.violations.len(), 1);
}
