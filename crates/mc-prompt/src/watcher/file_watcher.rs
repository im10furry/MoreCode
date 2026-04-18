use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{broadcast, mpsc};

use crate::error::PromptCacheError;
use crate::manager::{CacheInvalidationEvent, InvalidationReason};
use crate::watcher::file_mapping::{infer_layer_from_path, is_supported_prompt_file};

pub struct FileWatcherHandle {
    _watcher: RecommendedWatcher,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for FileWatcherHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub fn start_file_watcher(
    watch_paths: Vec<PathBuf>,
    tx: broadcast::Sender<CacheInvalidationEvent>,
    global_version: Arc<AtomicU64>,
) -> Result<FileWatcherHandle, PromptCacheError> {
    let (notify_tx, mut notify_rx) = mpsc::channel::<Result<Event, notify::Error>>(128);
    let mut watcher = recommended_watcher(move |result| {
        let _ = notify_tx.blocking_send(result);
    })
    .map_err(|error| PromptCacheError::FileWatcherError(error.to_string()))?;

    for path in &watch_paths {
        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|error| {
                PromptCacheError::FileWatcherError(format!(
                    "failed to watch '{}': {error}",
                    path.display()
                ))
            })?;
    }

    let task = tokio::spawn(async move {
        while let Some(result) = notify_rx.recv().await {
            let event = match result {
                Ok(event) => event,
                Err(_) => continue,
            };

            let reason = match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    InvalidationReason::FileChanged
                }
                _ => continue,
            };

            for path in event.paths {
                if !is_supported_prompt_file(&path) {
                    continue;
                }

                let version = global_version.fetch_add(1, Ordering::AcqRel) + 1;
                let _ = tx.send(CacheInvalidationEvent {
                    layer: infer_layer_from_path(&path),
                    version: 0,
                    global_version: version,
                    reason: reason.clone(),
                    timestamp: Utc::now(),
                    path: Some(path),
                });
            }
        }
    });

    Ok(FileWatcherHandle {
        _watcher: watcher,
        task,
    })
}
