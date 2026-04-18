use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::LlmError;

use super::{SemanticCacheEntry, SemanticCacheNamespace};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub avg_similarity: f64,
}

pub trait SemanticCacheStore: Send + Sync {
    fn find_similar(
        &self,
        namespace: &SemanticCacheNamespace,
        embedding: &[f32],
        threshold: f64,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Option<SemanticCacheEntry>, LlmError>> + Send + '_>>;

    fn store(
        &self,
        entry: SemanticCacheEntry,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>>;

    fn invalidate_namespace(
        &self,
        namespace: &SemanticCacheNamespace,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<usize, LlmError>> + Send + '_>>;

    fn stats(
        &self,
        namespace: &SemanticCacheNamespace,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<CacheStats, LlmError>> + Send + '_>>;
}

#[derive(Default)]
pub struct InMemorySemanticCacheStore {
    entries: Mutex<HashMap<SemanticCacheNamespace, Vec<SemanticCacheEntry>>>,
}

impl InMemorySemanticCacheStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SemanticCacheStore for InMemorySemanticCacheStore {
    fn find_similar(
        &self,
        namespace: &SemanticCacheNamespace,
        embedding: &[f32],
        threshold: f64,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<Option<SemanticCacheEntry>, LlmError>> + Send + '_>>
    {
        let namespace = namespace.clone();
        let embedding = embedding.to_vec();
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "semantic cache lookup cancelled".into(),
                });
            }

            let mut entries = self
                .entries
                .lock()
                .map_err(|_| LlmError::Internal("semantic cache store lock poisoned".into()))?;

            let namespace_entries = entries.entry(namespace).or_default();
            let mut best_index = None;
            let mut best_similarity = threshold;

            for (index, entry) in namespace_entries.iter().enumerate() {
                if entry.is_expired(std::time::Duration::from_secs(u64::MAX / 2)) {
                    continue;
                }

                let similarity = cosine_similarity(&entry.embedding, &embedding);
                if similarity >= best_similarity {
                    best_similarity = similarity;
                    best_index = Some(index);
                }
            }

            let hit = best_index.and_then(|index| {
                let entry = namespace_entries.get_mut(index)?;
                entry.record_hit(100);
                Some(entry.clone())
            });

            Ok(hit)
        })
    }

    fn store(
        &self,
        entry: SemanticCacheEntry,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<(), LlmError>> + Send + '_>> {
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "semantic cache store cancelled".into(),
                });
            }

            let mut entries = self
                .entries
                .lock()
                .map_err(|_| LlmError::Internal("semantic cache store lock poisoned".into()))?;
            entries
                .entry(entry.namespace.clone())
                .or_default()
                .push(entry);
            Ok(())
        })
    }

    fn invalidate_namespace(
        &self,
        namespace: &SemanticCacheNamespace,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<usize, LlmError>> + Send + '_>> {
        let namespace = namespace.clone();
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "semantic cache invalidation cancelled".into(),
                });
            }

            let mut entries = self
                .entries
                .lock()
                .map_err(|_| LlmError::Internal("semantic cache store lock poisoned".into()))?;
            let removed = entries
                .remove(&namespace)
                .map(|items| items.len())
                .unwrap_or(0);
            Ok(removed)
        })
    }

    fn stats(
        &self,
        namespace: &SemanticCacheNamespace,
        cancel_token: CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Result<CacheStats, LlmError>> + Send + '_>> {
        let namespace = namespace.clone();
        Box::pin(async move {
            if cancel_token.is_cancelled() {
                return Err(LlmError::Cancelled {
                    reason: "semantic cache stats cancelled".into(),
                });
            }

            let entries = self
                .entries
                .lock()
                .map_err(|_| LlmError::Internal("semantic cache store lock poisoned".into()))?;
            let Some(namespace_entries) = entries.get(&namespace) else {
                return Ok(CacheStats::default());
            };

            let total_hits = namespace_entries
                .iter()
                .map(|entry| entry.hit_count)
                .sum::<u64>();
            let total_entries = namespace_entries.len();
            let total_misses = 0u64;
            let hit_rate = if total_hits + total_misses == 0 {
                0.0
            } else {
                total_hits as f64 / (total_hits + total_misses) as f64
            };

            Ok(CacheStats {
                total_entries,
                total_hits,
                total_misses,
                hit_rate,
                avg_similarity: 0.0,
            })
        })
    }
}

fn cosine_similarity(lhs: &[f32], rhs: &[f32]) -> f64 {
    if lhs.is_empty() || rhs.is_empty() {
        return 0.0;
    }

    let len = lhs.len().min(rhs.len());
    let mut dot = 0.0f64;
    let mut lhs_norm = 0.0f64;
    let mut rhs_norm = 0.0f64;

    for index in 0..len {
        let l = lhs[index] as f64;
        let r = rhs[index] as f64;
        dot += l * r;
        lhs_norm += l * l;
        rhs_norm += r * r;
    }

    if lhs_norm == 0.0 || rhs_norm == 0.0 {
        0.0
    } else {
        dot / (lhs_norm.sqrt() * rhs_norm.sqrt())
    }
}
