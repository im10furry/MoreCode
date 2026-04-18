use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;

use crate::error::MemoryError;

use super::{CoreMemory, CoreMemoryManager, KnowledgeEntry, KnowledgeSource, KnowledgeStore};

pub async fn compact_core_memory(
    core_memory: &Arc<RwLock<CoreMemory>>,
    knowledge_store: &Arc<dyn KnowledgeStore>,
) -> Result<usize, MemoryError> {
    let evicted = {
        let mut memory = core_memory.write().await;
        let target = memory.max_token_budget() * 70 / 100;
        memory.compact(target)
    };

    let evicted_count = evicted.len();
    for block in evicted {
        let entry = KnowledgeEntry {
            id: format!("demoted:{}", block.id),
            title: format!("归档记忆: {}", block.id),
            content: block.content,
            category: format!("demoted_{}", block.category.display_name()),
            source: KnowledgeSource::CoreMemoryDemotion {
                block_id: block.id.clone(),
            },
            tags: vec!["auto-demoted".into(), "core-memory".into()],
            created_at: block.created_at,
            updated_at: Utc::now(),
            embedding: None,
            entities: Vec::new(),
        };
        knowledge_store.store(entry).await?;
    }

    Ok(evicted_count)
}
