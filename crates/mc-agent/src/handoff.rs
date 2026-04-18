use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AgentHandoff {
    inner: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    parent: Option<Arc<AgentHandoff>>,
}

impl AgentHandoff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: Arc<AgentHandoff>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            parent: Some(parent),
        }
    }

    pub async fn put<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .write()
            .await
            .insert(TypeId::of::<T>(), Box::new(value));
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.inner
            .read()
            .await
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

        let mut current = self.parent.clone();
        while let Some(parent) = current {
            if let Some(value) = parent.get::<T>().await {
                return Some(value);
            }
            current = parent.parent.clone();
        }

        None
    }
}
