use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Default)]
struct HandoffData {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

#[derive(Clone)]
pub struct AgentHandoff {
    inner: Arc<RwLock<HandoffData>>,
    parent: Option<Arc<AgentHandoff>>,
}

impl AgentHandoff {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HandoffData::default())),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<AgentHandoff>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HandoffData::default())),
            parent: Some(parent),
        }
    }

    pub async fn put<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        let mut guard = self.inner.write().await;
        guard.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let guard = self.inner.read().await;
        guard
            .data
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
    }

    pub async fn get_with_fallback<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        if let Some(value) = self.get::<T>().await {
            return Some(value);
        }

        let mut parent = self.parent.clone();
        while let Some(current) = parent {
            if let Some(value) = current.get::<T>().await {
                return Some(value);
            }
            parent = current.parent.clone();
        }

        None
    }

    pub async fn clear(&self) {
        self.inner.write().await.data.clear();
    }
}

impl Default for AgentHandoff {
    fn default() -> Self {
        Self::new()
    }
}
