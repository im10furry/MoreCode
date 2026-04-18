use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{
    CacheCapability, ChatRequest, ChatResponse, LlmError, LlmProvider, ModelInfo, StreamEvent,
};

use super::{
    CacheStats, SemanticCacheConfig, SemanticCacheEntry, SemanticCacheNamespace, SemanticCacheStore,
};

pub struct SemanticCacheMiddleware<P: LlmProvider, S: SemanticCacheStore> {
    inner: P,
    store: S,
    config: SemanticCacheConfig,
    embed_fn: fn(&str) -> Vec<f32>,
    stats: RwLock<CacheStats>,
    hit_stats: RwLock<HashMap<String, (u32, u32)>>,
    miss_counters: RwLock<HashMap<String, u32>>,
}

impl<P: LlmProvider, S: SemanticCacheStore> SemanticCacheMiddleware<P, S> {
    pub fn new(inner: P, store: S, config: SemanticCacheConfig) -> Result<Self, LlmError> {
        config.validate()?;
        Ok(Self {
            inner,
            store,
            config,
            embed_fn: |text| text.bytes().map(|byte| byte as f32 / 255.0).collect(),
            stats: RwLock::new(CacheStats::default()),
            hit_stats: RwLock::new(HashMap::new()),
            miss_counters: RwLock::new(HashMap::new()),
        })
    }

    pub fn with_embed_fn(mut self, embed_fn: fn(&str) -> Vec<f32>) -> Self {
        self.embed_fn = embed_fn;
        self
    }

    pub async fn cleanup_stale_stats(&self) {
        let mut stats = self.stats.write().await;
        let total = stats.total_hits + stats.total_misses;
        stats.hit_rate = if total == 0 {
            0.0
        } else {
            stats.total_hits as f64 / total as f64
        };
    }

    fn compute_embedding(&self, request: &ChatRequest) -> Vec<f32> {
        let text = request
            .messages
            .iter()
            .map(|message| message.content.to_text())
            .collect::<Vec<_>>()
            .join("\n");
        (self.embed_fn)(&text)
    }

    fn namespace_for(&self, request: &ChatRequest) -> SemanticCacheNamespace {
        SemanticCacheNamespace {
            name: "default".into(),
            model_id: request
                .model
                .clone()
                .unwrap_or_else(|| self.inner.model_info().id.clone()),
            project_id: request.user_id.clone(),
        }
    }

    fn should_use_cache(&self, request: &ChatRequest) -> bool {
        self.config.enabled
            && (!self.config.disable_for_code_generation || !looks_like_code_generation(request))
    }

    async fn record_hit(&self, namespace: &SemanticCacheNamespace) {
        let namespace_key = namespace.name.clone();
        {
            let mut stats = self.stats.write().await;
            stats.total_hits += 1;
        }
        let mut hit_stats = self.hit_stats.write().await;
        let entry = hit_stats.entry(namespace_key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += 1;
    }

    async fn record_miss(&self, namespace: &SemanticCacheNamespace) {
        let namespace_key = namespace.name.clone();
        {
            let mut stats = self.stats.write().await;
            stats.total_misses += 1;
        }
        {
            let mut hit_stats = self.hit_stats.write().await;
            let entry = hit_stats.entry(namespace_key.clone()).or_insert((0, 0));
            entry.1 += 1;
        }
        let mut miss_counters = self.miss_counters.write().await;
        let counter = miss_counters.entry(namespace_key).or_insert(0);
        *counter += 1;
    }
}

impl<P: LlmProvider, S: SemanticCacheStore> LlmProvider for SemanticCacheMiddleware<P, S> {
    fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    fn model_info(&self) -> &ModelInfo {
        self.inner.model_info()
    }

    fn chat(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, LlmError>> + Send + '_>> {
        Box::pin(async move {
            if !self.should_use_cache(&request) {
                return self.inner.chat(request, cancel_token).await;
            }

            let namespace = self.namespace_for(&request);
            let embedding = self.compute_embedding(&request);

            match self
                .store
                .find_similar(
                    &namespace,
                    &embedding,
                    self.config.similarity_threshold,
                    cancel_token.child_token(),
                )
                .await
            {
                Ok(Some(entry)) if !entry.is_expired(self.config.max_ttl) => {
                    self.record_hit(&namespace).await;
                    return Ok(entry.response);
                }
                Ok(_) => {
                    self.record_miss(&namespace).await;
                }
                Err(LlmError::Cancelled { .. }) => {
                    return Err(LlmError::Cancelled {
                        reason: "semantic cache lookup cancelled".into(),
                    });
                }
                Err(_) => {
                    self.record_miss(&namespace).await;
                }
            }

            let response = self
                .inner
                .chat(request.clone(), cancel_token.child_token())
                .await?;

            if response.message.content.to_text().chars().count() <= self.config.max_response_chars
            {
                let entry = SemanticCacheEntry {
                    id: response.id.clone(),
                    embedding,
                    response: response.clone(),
                    created_at: Instant::now(),
                    last_accessed_at: Instant::now(),
                    hit_count: 0,
                    hit_stats: Vec::new(),
                    namespace,
                };

                let _ = self.store.store(entry, cancel_token.child_token()).await;
            }

            Ok(response)
        })
    }

    fn chat_stream(
        &self,
        request: ChatRequest,
        cancel_token: CancellationToken,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<tokio::sync::mpsc::Receiver<StreamEvent>, LlmError>>
                + Send
                + '_,
        >,
    > {
        self.inner.chat_stream(request, cancel_token)
    }

    fn cache_capability(&self) -> CacheCapability {
        self.inner.cache_capability()
    }

    fn list_models(
        &self,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelInfo>, LlmError>> + Send + '_>> {
        self.inner.list_models(cancel_token)
    }

    fn cancel_request(&self, request_id: &str) -> Result<(), LlmError> {
        self.inner.cancel_request(request_id)
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        self.inner.estimate_tokens(text)
    }
}

fn looks_like_code_generation(request: &ChatRequest) -> bool {
    request.messages.iter().any(|message| {
        let content = message.content.to_text().to_ascii_lowercase();
        content.contains("implement")
            || content.contains("write code")
            || content.contains("refactor")
            || content.contains("```")
    })
}
