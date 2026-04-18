use std::time::{Duration, Instant};

use crate::ChatResponse;

use super::SemanticCacheNamespace;

#[derive(Debug, Clone)]
pub struct SemanticCacheEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub response: ChatResponse,
    pub created_at: Instant,
    pub last_accessed_at: Instant,
    pub hit_count: u64,
    pub hit_stats: Vec<Instant>,
    pub namespace: SemanticCacheNamespace,
}

impl SemanticCacheEntry {
    pub fn is_expired(&self, max_ttl: Duration) -> bool {
        self.created_at.elapsed() > max_ttl
    }

    pub fn record_hit(&mut self, stats_window_size: usize) {
        let now = Instant::now();
        self.hit_count += 1;
        self.last_accessed_at = now;
        self.hit_stats.push(now);

        let cutoff = now - Duration::from_secs(300);
        self.hit_stats.retain(|instant| *instant > cutoff);
        if self.hit_stats.len() > stats_window_size {
            let keep_from = self.hit_stats.len() - stats_window_size;
            self.hit_stats.drain(..keep_from);
        }
    }
}
