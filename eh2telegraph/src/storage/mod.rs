use futures::Future;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

pub mod cloudflare_kv;
pub mod lru;

pub trait KVStorage<V> {
    fn get(&self, key: &str) -> impl Future<Output = anyhow::Result<Option<V>>> + Send;
    fn set(
        &self,
        key: String,
        value: V,
        expire_ttl: Option<usize>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn delete(&self, key: &str) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Clone, Debug)]
pub struct SimpleMemStorage<T>(Arc<RwLock<HashMap<String, T>>>);

impl<T> Default for SimpleMemStorage<T> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl<T> SimpleMemStorage<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Arc::new(RwLock::new(HashMap::with_capacity(capacity))))
    }
}

impl<T> KVStorage<T> for SimpleMemStorage<T>
where
    T: Clone + Send + Sync,
{
    async fn get(&self, key: &str) -> anyhow::Result<Option<T>> {
        let v = self.0.read().get(key).cloned();
        Ok(v)
    }

    async fn set(&self, key: String, value: T, _expire_ttl: Option<usize>) -> anyhow::Result<()> {
        self.0.write().insert(key, value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.0.write().remove(key);
        Ok(())
    }
}
