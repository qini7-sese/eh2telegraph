use std::{sync::Arc, time::Duration};

use cloudflare_kv_proxy::{Client, ClientError, NotFoundMapping};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::config;

use super::{KVStorage, SimpleMemStorage};

const CONFIG_KEY: &str = "worker_kv";
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Deserialize)]
pub struct CFConfig {
    pub endpoint: String,
    pub token: String,
    pub cache_size: usize,
    pub expire_sec: u64,
}

#[derive(Clone, Debug)]
pub struct CFStorage(Arc<Client>);

impl CFStorage {
    pub fn new<T: Into<String>, E: Into<String>>(
        endpoint: E,
        token: T,
        cache_size: usize,
        expire: Duration,
    ) -> Result<Self, ClientError> {
        Ok(Self(Arc::new(Client::new(
            endpoint, token, TIMEOUT, cache_size, expire,
        )?)))
    }

    pub fn new_from_config() -> anyhow::Result<Self> {
        let config: CFConfig = config::parse(CONFIG_KEY)?
            .ok_or_else(|| anyhow::anyhow!("cloudflare worker config(key: worker_kv) not found"))?;
        Self::new(
            config.endpoint,
            config.token,
            config.cache_size,
            Duration::from_secs(config.expire_sec),
        )
        .map_err(Into::into)
    }
}

impl<T> KVStorage<T> for CFStorage
where
    T: DeserializeOwned + Serialize + Send + Sync,
{
    async fn get(&self, key: &str) -> anyhow::Result<Option<T>> {
        self.0
            .get(key)
            .await
            .map_not_found_to_option()
            .map_err(Into::into)
    }

    async fn set(&self, key: String, value: T, _expire_ttl: Option<usize>) -> anyhow::Result<()> {
        self.0.put(&key, &value).await.map_err(Into::into)
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.0.delete(key).await.map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub enum CFOrMemStorage<T> {
    Mem(SimpleMemStorage<T>),
    CF(CFStorage),
}

impl<T> CFOrMemStorage<T> {
    pub fn new_from_config() -> Self {
        match CFStorage::new_from_config() {
            Ok(s) => CFOrMemStorage::CF(s),
            Err(e) => {
                tracing::error!(
                    "unable to read cloudflare cache settings, will use in memory cache: {e:?}"
                );
                CFOrMemStorage::Mem(SimpleMemStorage::<T>::default())
            }
        }
    }
}

impl<T> KVStorage<T> for CFOrMemStorage<T>
where
    T: Clone + Send + Sync,
    CFStorage: KVStorage<T>,
{
    async fn get(&self, key: &str) -> anyhow::Result<Option<T>> {
        match self {
            CFOrMemStorage::Mem(inner) => inner.get(key).await,
            CFOrMemStorage::CF(inner) => inner.get(key).await,
        }
    }

    async fn set(&self, key: String, value: T, expire_ttl: Option<usize>) -> anyhow::Result<()> {
        match self {
            CFOrMemStorage::Mem(inner) => inner.set(key, value, expire_ttl).await,
            CFOrMemStorage::CF(inner) => inner.set(key, value, expire_ttl).await,
        }
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        match self {
            CFOrMemStorage::Mem(inner) => inner.delete(key).await,
            CFOrMemStorage::CF(inner) => inner.delete(key).await,
        }
    }
}
