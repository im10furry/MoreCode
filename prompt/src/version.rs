use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::layer::PromptLayer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedLayer {
    pub layer: PromptLayer,
    pub version: u64,
    pub updated_at: DateTime<Utc>,
}

pub struct CacheVersionTracker {
    global_version: AtomicU64,
    layer_versions: Arc<RwLock<HashMap<PromptLayer, VersionedLayer>>>,
}

impl CacheVersionTracker {
    pub fn new() -> Self {
        let mut layer_versions = HashMap::new();
        for layer in PromptLayer::all() {
            layer_versions.insert(
                layer,
                VersionedLayer {
                    layer,
                    version: 0,
                    updated_at: Utc::now(),
                },
            );
        }

        Self {
            global_version: AtomicU64::new(0),
            layer_versions: Arc::new(RwLock::new(layer_versions)),
        }
    }

    pub fn global_version(&self) -> u64 {
        self.global_version.load(Ordering::Acquire)
    }

    pub async fn layer_version(&self, layer: PromptLayer) -> u64 {
        self.layer_versions
            .read()
            .await
            .get(&layer)
            .map(|entry| entry.version)
            .unwrap_or(0)
    }

    pub async fn bump_layer(&self, layer: PromptLayer) -> VersionedLayer {
        let global = self.global_version.fetch_add(1, Ordering::AcqRel) + 1;
        let mut guard = self.layer_versions.write().await;
        let entry = guard.entry(layer).or_insert_with(|| VersionedLayer {
            layer,
            version: 0,
            updated_at: Utc::now(),
        });
        entry.version = entry.version.saturating_add(1);
        entry.updated_at = Utc::now();

        let updated = entry.clone();
        debug_assert!(global >= updated.version);
        updated
    }

    pub async fn snapshot(&self) -> HashMap<PromptLayer, VersionedLayer> {
        self.layer_versions.read().await.clone()
    }

    pub async fn reset(&self) {
        self.global_version.store(0, Ordering::Release);
        let mut guard = self.layer_versions.write().await;
        for entry in guard.values_mut() {
            entry.version = 0;
            entry.updated_at = Utc::now();
        }
    }
}

impl Default for CacheVersionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::layer::PromptLayer;

    use super::CacheVersionTracker;

    #[tokio::test]
    async fn tracker_bumps_layer_and_global_versions() {
        let tracker = CacheVersionTracker::new();
        assert_eq!(tracker.global_version(), 0);
        assert_eq!(tracker.layer_version(PromptLayer::Global).await, 0);

        let first = tracker.bump_layer(PromptLayer::Global).await;
        let second = tracker.bump_layer(PromptLayer::Project).await;

        assert_eq!(first.version, 1);
        assert_eq!(second.version, 1);
        assert_eq!(tracker.global_version(), 2);
        assert_eq!(tracker.layer_version(PromptLayer::Global).await, 1);
        assert_eq!(tracker.layer_version(PromptLayer::Project).await, 1);
    }
}
