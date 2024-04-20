use std::sync::Arc;

use hashlink::LruCache;
use parking_lot::Mutex;

use super::KVStorage;

#[derive(Clone, Debug)]
pub struct LruStorage(Arc<Mutex<LruCache<String, String>>>);

impl LruStorage {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(Mutex::new(LruCache::new(capacity))))
    }
}

impl KVStorage<String> for LruStorage {
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let v = self.0.lock().get(key).cloned();
        Ok(v)
    }

    async fn set(
        &self,
        key: String,
        value: String,
        _expire_ttl: Option<usize>,
    ) -> anyhow::Result<()> {
        self.0.lock().insert(key, value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.0.lock().remove(key);
        Ok(())
    }
}
