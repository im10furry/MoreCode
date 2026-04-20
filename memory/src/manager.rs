use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::error::MemoryError;
use crate::letta::{
    CoreMemory, LruFileCache, ProceduralMemory, SqliteConversationStore, SqliteKnowledgeStore,
};
use crate::preference::{PreferenceManager, RuleBundle, RuleLoader, UserPreferences};
use crate::state::ProjectMemoryState;
use crate::store::{MemoryManager as StoreMemoryManager, MemoryManagerTrait, MemoryUpdate};
use crate::write_queue::MemoryWriteQueue;

pub struct MemorySystem {
    project_root: PathBuf,
    letta_dir: PathBuf,
    store_manager: Arc<StoreMemoryManager>,
    write_queue: MemoryWriteQueue,
    preference_manager: PreferenceManager,
    rule_loader: RuleLoader,
    recall_store: Arc<Mutex<Option<Arc<SqliteConversationStore>>>>,
    archival_store: Arc<Mutex<Option<Arc<SqliteKnowledgeStore>>>>,
    procedural_memory: Arc<RwLock<ProceduralMemory>>,
    core_memory: Arc<RwLock<CoreMemory>>,
    working_memory: Arc<LruFileCache>,
}

impl MemorySystem {
    pub async fn new(project_root: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let project_root = project_root.as_ref().to_path_buf();
        let letta_dir = project_root.join(".assistant-memory").join("letta");
        tokio::fs::create_dir_all(&letta_dir).await?;

        let (sender, receiver) = tokio::sync::mpsc::channel(128);
        let store_manager = Arc::new(StoreMemoryManager::new(&project_root, sender.clone()));
        let write_queue = MemoryWriteQueue::new(sender, receiver, Arc::clone(&store_manager));
        let preference_manager = PreferenceManager::new(&project_root);
        let rule_loader = RuleLoader::new(&project_root);
        let procedural_memory = Arc::new(RwLock::new(
            ProceduralMemory::load_from_path(&letta_dir.join("procedural.json"))
                .await
                .unwrap_or_default(),
        ));

        Ok(Self {
            project_root,
            letta_dir,
            store_manager,
            write_queue,
            preference_manager,
            rule_loader,
            recall_store: Arc::new(Mutex::new(None)),
            archival_store: Arc::new(Mutex::new(None)),
            procedural_memory,
            core_memory: Arc::new(RwLock::new(CoreMemory::new(4_096))),
            working_memory: Arc::new(LruFileCache::default()),
        })
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn store_manager(&self) -> Arc<StoreMemoryManager> {
        Arc::clone(&self.store_manager)
    }

    pub async fn recall_store(&self) -> Result<Arc<SqliteConversationStore>, MemoryError> {
        let mut guard = self.recall_store.lock().await;
        if let Some(store) = guard.as_ref() {
            return Ok(Arc::clone(store));
        }

        let store =
            Arc::new(SqliteConversationStore::new(&self.letta_dir.join("recall.db")).await?);
        *guard = Some(Arc::clone(&store));
        Ok(store)
    }

    pub async fn archival_store(&self) -> Result<Arc<SqliteKnowledgeStore>, MemoryError> {
        let mut guard = self.archival_store.lock().await;
        if let Some(store) = guard.as_ref() {
            return Ok(Arc::clone(store));
        }

        let store = Arc::new(SqliteKnowledgeStore::new(&self.letta_dir).await?);
        *guard = Some(Arc::clone(&store));
        Ok(store)
    }

    pub fn core_memory(&self) -> Arc<RwLock<CoreMemory>> {
        Arc::clone(&self.core_memory)
    }

    pub fn working_memory(&self) -> Arc<LruFileCache> {
        Arc::clone(&self.working_memory)
    }

    pub async fn load_project_memory_state(&self) -> Result<ProjectMemoryState, MemoryError> {
        let memory = self
            .store_manager
            .load_memory()
            .await
            .map_err(|error| MemoryError::Internal(error.to_string()))?;
        Ok(memory.into())
    }

    pub async fn refresh_project_memory(&self) -> Result<ProjectMemoryState, MemoryError> {
        let memory = self
            .store_manager
            .incremental_update()
            .await
            .map_err(|error| MemoryError::Internal(error.to_string()))?;
        Ok(memory.into())
    }

    pub async fn submit_update(&self, update: MemoryUpdate) -> Result<(), MemoryError> {
        self.write_queue.submit(update).await
    }

    pub async fn memory_summary(&self) -> Result<String, MemoryError> {
        self.store_manager
            .get_memory_summary()
            .await
            .map_err(|error| MemoryError::Internal(error.to_string()))
    }

    pub async fn load_user_preferences(&self) -> Result<UserPreferences, MemoryError> {
        UserPreferences::load(&self.project_root).await
    }

    pub async fn load_rule_bundle(&self) -> Result<RuleBundle, MemoryError> {
        self.rule_loader.load().await
    }

    pub async fn load_preference_prompt_block(&self) -> Result<String, MemoryError> {
        self.preference_manager.render_prompt_block().await
    }

    pub async fn procedural_memory(&self) -> ProceduralMemory {
        self.procedural_memory.read().await.clone()
    }

    pub async fn shutdown(&self) {
        self.write_queue.shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use tokio::fs;

    use super::MemorySystem;

    #[tokio::test]
    async fn memory_system_loads_lightweight_facades() {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("src")).await.unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n")
            .await
            .unwrap();

        let system = MemorySystem::new(temp.path()).await.unwrap();
        assert_eq!(system.project_root(), temp.path());
        assert!(system
            .load_preference_prompt_block()
            .await
            .unwrap()
            .is_empty());
        assert!(system.load_rule_bundle().await.unwrap().rules.is_empty());
        assert!(system
            .load_user_preferences()
            .await
            .unwrap()
            .preferences
            .is_empty());

        system.shutdown().await;
    }
}
