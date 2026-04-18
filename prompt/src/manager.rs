use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use mc_llm::{CacheControlType, ChatMessage, MessageRole};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use crate::cache::should_set_cache_breakpoint;
use crate::error::PromptCacheError;
use crate::layer::{PromptLayer, PromptLayerContent, PromptLayers, TurnMessage};
use crate::template::{extract_template_variables, is_valid_variable_name, TemplateRenderer};
use crate::watcher::{start_file_watcher, FileWatcherHandle};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationReason {
    LayerUpdated,
    FileChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInvalidationEvent {
    pub layer: PromptLayer,
    pub version: u64,
    pub global_version: u64,
    pub reason: InvalidationReason,
    pub timestamp: DateTime<Utc>,
    pub path: Option<PathBuf>,
}

pub struct PromptLayerManager {
    layers: Arc<RwLock<PromptLayers>>,
    invalidation_tx: broadcast::Sender<CacheInvalidationEvent>,
    global_version: Arc<AtomicU64>,
    file_watcher: Mutex<Option<FileWatcherHandle>>,
}

impl PromptLayerManager {
    pub fn new() -> Self {
        let (invalidation_tx, _) = broadcast::channel(256);
        Self {
            layers: Arc::new(RwLock::new(PromptLayers::new())),
            invalidation_tx,
            global_version: Arc::new(AtomicU64::new(0)),
            file_watcher: Mutex::new(None),
        }
    }

    pub async fn update_layer(
        &self,
        layer: PromptLayer,
        system_prompt: impl Into<String>,
        variables: HashMap<String, String>,
    ) -> Result<u64, PromptCacheError> {
        if layer == PromptLayer::Turn {
            return Err(PromptCacheError::TurnLayerNotUpdatable);
        }

        let system_prompt = system_prompt.into();
        let _ = extract_template_variables(&system_prompt)?;
        validate_variable_map(&variables)?;

        let mut layers = self.layers.write().await;
        let next_version = layers
            .get(layer)
            .map(|content| content.version() + 1)
            .unwrap_or(1);
        let content = PromptLayerContent::from_parts(layer, system_prompt, variables, next_version);
        layers.set(content);
        drop(layers);

        let global_version = self.global_version.fetch_add(1, Ordering::AcqRel) + 1;
        let event = CacheInvalidationEvent {
            layer,
            version: next_version,
            global_version,
            reason: InvalidationReason::LayerUpdated,
            timestamp: Utc::now(),
            path: None,
        };
        let _ = self.invalidation_tx.send(event);

        Ok(next_version)
    }

    pub async fn append_turn_message(&self, message: TurnMessage) {
        let mut layers = self.layers.write().await;
        layers.append_turn_message(message);
    }

    pub async fn replace_turn_history(&self, messages: Vec<TurnMessage>) {
        let mut layers = self.layers.write().await;
        layers.replace_turn_history(messages);
    }

    pub async fn get_version(&self, layer: PromptLayer) -> u64 {
        let layers = self.layers.read().await;
        layers
            .get(layer)
            .map(|content| content.version())
            .unwrap_or(0)
    }

    pub fn get_global_version(&self) -> u64 {
        self.global_version.load(Ordering::Acquire)
    }

    pub async fn build_messages(
        &self,
        turn_messages: &[TurnMessage],
        template_vars: &HashMap<String, String>,
    ) -> Result<Vec<ChatMessage>, PromptCacheError> {
        let layers = self.layers.read().await;
        let mut merged_variables = layers.merge_variables();
        for (key, value) in template_vars {
            merged_variables.insert(key.clone(), value.clone());
        }

        let renderer = TemplateRenderer::strict();
        let mut messages = Vec::new();
        let mut last_cacheable_index = None;

        for content in layers.sorted_layers() {
            let rendered = renderer.render(content.system_prompt(), &merged_variables)?;
            if rendered.trim().is_empty() {
                continue;
            }

            if content.layer().should_cache() {
                last_cacheable_index = Some(messages.len());
            }

            messages.push(ChatMessage::text(MessageRole::System, rendered));
        }

        if should_set_cache_breakpoint(turn_messages) {
            if let Some(index) = last_cacheable_index {
                if let Some(message) = messages.get_mut(index) {
                    message.cache_control = Some(CacheControlType::CacheBreakpoint);
                }
            }
        }

        for turn in turn_messages {
            messages.push(ChatMessage::text(turn.role, turn.content.clone()));
        }

        Ok(messages)
    }

    pub fn subscribe_invalidation(&self) -> broadcast::Receiver<CacheInvalidationEvent> {
        self.invalidation_tx.subscribe()
    }

    pub async fn on_invalidation(
        &self,
        event: CacheInvalidationEvent,
    ) -> Result<(), PromptCacheError> {
        let current_version = self.get_version(event.layer).await;
        if event.version < current_version {
            return Err(PromptCacheError::StaleEvent {
                layer: event.layer,
                event_version: event.version,
                current_version,
            });
        }

        Ok(())
    }

    pub fn start_file_watcher(&self, watch_paths: &[PathBuf]) -> Result<(), PromptCacheError> {
        let handle = start_file_watcher(
            watch_paths.to_vec(),
            self.invalidation_tx.clone(),
            Arc::clone(&self.global_version),
        )?;

        let mut guard = self.file_watcher.lock().map_err(|_| {
            PromptCacheError::FileWatcherError("file watcher mutex poisoned".to_string())
        })?;
        *guard = Some(handle);
        Ok(())
    }
}

impl Default for PromptLayerManager {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_variable_map(variables: &HashMap<String, String>) -> Result<(), PromptCacheError> {
    for key in variables.keys() {
        if !is_valid_variable_name(key) {
            return Err(PromptCacheError::InvalidVariableName(key.clone()));
        }
    }

    Ok(())
}
