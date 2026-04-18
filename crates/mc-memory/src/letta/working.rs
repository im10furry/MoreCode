use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use lru::LruCache;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt},
    sync::RwLock,
};

use crate::error::MemoryError;

#[derive(Debug, Clone)]
pub struct CachedFile {
    pub path: PathBuf,
    pub content: String,
    pub original_size: u64,
    pub cached_size: u64,
    pub is_summary: bool,
    pub modified_time: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CacheStats {
    pub file_count: usize,
    pub total_bytes: u64,
    pub max_files: usize,
    pub max_bytes: u64,
    pub utilization_ratio: f64,
}

#[derive(Debug)]
pub struct LruFileCache {
    cache: RwLock<LruCache<PathBuf, Arc<CachedFile>>>,
    max_files: usize,
    max_bytes: u64,
    // Lock order rule: always acquire `cache` first, then `current_bytes`.
    current_bytes: RwLock<u64>,
}

impl Default for LruFileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LruFileCache {
    pub fn new() -> Self {
        Self::with_capacity(100, 25 * 1024 * 1024)
    }

    pub fn with_capacity(max_files: usize, max_bytes: u64) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(max_files).expect("max_files must be > 0"),
            )),
            max_files,
            max_bytes,
            current_bytes: RwLock::new(0),
        }
    }

    pub async fn get_or_read(&self, path: &Path) -> Result<Arc<CachedFile>, MemoryError> {
        let path_buf = path.to_path_buf();

        {
            let cache = self.cache.read().await;
            if cache.contains(&path_buf) {
                drop(cache);
                let mut cache = self.cache.write().await;
                if let Some(existing) = cache.get(&path_buf) {
                    return Ok(Arc::clone(existing));
                }
            }
        }

        let metadata = fs::metadata(path).await?;
        let file_size = metadata.len();
        let modified_time = metadata.modified()?;

        let content = if file_size > 50_000 {
            self.read_with_smart_truncation(path, file_size).await?
        } else {
            let bytes = fs::read(path).await?;
            String::from_utf8_lossy(&bytes).to_string()
        };

        let cached_size = content.len() as u64;
        if cached_size > self.max_bytes {
            return Err(MemoryError::CacheCapacityExceeded);
        }

        let cached_file = Arc::new(CachedFile {
            path: path_buf.clone(),
            content,
            original_size: file_size,
            cached_size,
            is_summary: cached_size < file_size,
            modified_time,
        });

        let mut cache = self.cache.write().await;
        if let Some(existing) = cache.get(&path_buf) {
            if existing.modified_time == modified_time {
                return Ok(Arc::clone(existing));
            }
        }

        let mut bytes = self.current_bytes.write().await;
        if let Some(previous) = cache.put(path_buf, Arc::clone(&cached_file)) {
            *bytes = bytes.saturating_sub(previous.cached_size);
        }
        *bytes += cached_file.cached_size;

        while cache.len() > self.max_files || *bytes > self.max_bytes {
            if let Some((_path, removed)) = cache.pop_lru() {
                *bytes = bytes.saturating_sub(removed.cached_size);
            } else {
                break;
            }
        }

        Ok(cached_file)
    }

    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let bytes = self.current_bytes.read().await;
        CacheStats {
            file_count: cache.len(),
            total_bytes: *bytes,
            max_files: self.max_files,
            max_bytes: self.max_bytes,
            utilization_ratio: if self.max_bytes == 0 {
                0.0
            } else {
                *bytes as f64 / self.max_bytes as f64
            },
        }
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut bytes = self.current_bytes.write().await;
        cache.clear();
        *bytes = 0;
    }

    async fn read_with_smart_truncation(
        &self,
        path: &Path,
        file_size: u64,
    ) -> Result<String, MemoryError> {
        let head_size = 15_000usize;
        let tail_size = 15_000usize;
        let mut file = fs::File::open(path).await?;

        let head_len = head_size.min(file_size as usize);
        let mut head = vec![0u8; head_len];
        file.read_exact(&mut head).await?;

        let remaining = file_size as usize - head_len;
        let tail_len = tail_size.min(remaining);
        let mut tail = vec![0u8; tail_len];
        if tail_len > 0 {
            file.seek(std::io::SeekFrom::End(-(tail_len as i64)))
                .await?;
            file.read_exact(&mut tail).await?;
        }

        let head = String::from_utf8_lossy(&head);
        let tail = String::from_utf8_lossy(&tail);

        if tail_len == 0 {
            return Ok(head.to_string());
        }

        Ok(format!(
            "{head}\n\n... [文件过大，已截断: 原始 {file_size} 字节，显示首尾各 ~15KB] ...\n\n{tail}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_clear_resets_stats() {
        let cache = LruFileCache::with_capacity(2, 1024);
        cache.clear().await;
        assert_eq!(cache.stats().await.file_count, 0);
    }
}
