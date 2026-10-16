//! Soft-degrading cache. When Redis is unreachable every call is a no-op.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use redis::aio::ConnectionManager;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str, ttl_secs: u64);
    async fn delete(&self, key: &str);
    /// Delete every key that starts with `prefix`.
    async fn delete_prefix(&self, prefix: &str);
}

struct PoolInner {
    conn: Option<ConnectionManager>,
    last_failure: Option<Instant>,
}

/// Lazily connected, shared Redis connection (used by the cache and the job queue).
pub struct RedisPool {
    client: redis::Client,
    inner: tokio::sync::Mutex<PoolInner>,
}

impl RedisPool {
    pub fn new(url: &str) -> Result<RedisPool, redis::RedisError> {
        let client = redis::Client::open(url)?;
        Ok(RedisPool {
            client,
            inner: tokio::sync::Mutex::new(PoolInner {
                conn: None,
                last_failure: None,
            }),
        })
    }

    /// A cloned connection manager, or an error message when Redis is down.
    /// After a failure, reconnects are not attempted again for a few seconds so
    /// that a dead Redis does not slow every request down.
    pub async fn connection(&self) -> Result<ConnectionManager, String> {
        let mut guard = self.inner.lock().await;
        if let Some(conn) = guard.conn.as_ref() {
            return Ok(conn.clone());
        }
        if let Some(at) = guard.last_failure {
            if at.elapsed() < Duration::from_secs(3) {
                return Err("connection refused (retrying shortly)".to_string());
            }
        }
        let attempt = tokio::time::timeout(
            Duration::from_secs(2),
            ConnectionManager::new(self.client.clone()),
        )
        .await;
        match attempt {
            Ok(Ok(conn)) => {
                guard.conn = Some(conn.clone());
                guard.last_failure = None;
                Ok(conn)
            }
            Ok(Err(err)) => {
                guard.last_failure = Some(Instant::now());
                Err(err.to_string())
            }
            Err(_) => {
                guard.last_failure = Some(Instant::now());
                Err("connection timed out".to_string())
            }
        }
    }
}

pub struct RedisCache {
    pool: std::sync::Arc<RedisPool>,
}

impl RedisCache {
    pub fn new(pool: std::sync::Arc<RedisPool>) -> Self {
        RedisCache { pool }
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.pool.connection().await.ok()?;
        let result: redis::RedisResult<Option<String>> =
            redis::cmd("GET").arg(key).query_async(&mut conn).await;
        match result {
            Ok(value) => value,
            Err(err) => {
                tracing::debug!("cache get failed: {}", err);
                None
            }
        }
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) {
        let mut conn = match self.pool.connection().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let result: redis::RedisResult<()> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await;
        if let Err(err) = result {
            tracing::debug!("cache set failed: {}", err);
        }
    }

    async fn delete(&self, key: &str) {
        let mut conn = match self.pool.connection().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let result: redis::RedisResult<()> =
            redis::cmd("DEL").arg(key).query_async(&mut conn).await;
        if let Err(err) = result {
            tracing::debug!("cache delete failed: {}", err);
        }
    }

    async fn delete_prefix(&self, prefix: &str) {
        let mut conn = match self.pool.connection().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let pattern = format!("{}*", prefix);
        let mut cursor: u64 = 0;
        loop {
            let scanned: redis::RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(200)
                .query_async(&mut conn)
                .await;
            let (next, keys) = match scanned {
                Ok(pair) => pair,
                Err(err) => {
                    tracing::debug!("cache scan failed: {}", err);
                    return;
                }
            };
            if !keys.is_empty() {
                let mut del = redis::cmd("DEL");
                for key in &keys {
                    del.arg(key);
                }
                let deleted: redis::RedisResult<()> = del.query_async(&mut conn).await;
                if let Err(err) = deleted {
                    tracing::debug!("cache delete failed: {}", err);
                    return;
                }
            }
            if next == 0 {
                break;
            }
            cursor = next;
        }
    }
}

/// Cache that stores nothing. Used when Redis cannot be configured at all.
pub struct NoopCache;

#[async_trait]
impl Cache for NoopCache {
    async fn get(&self, _key: &str) -> Option<String> {
        None
    }
    async fn set(&self, _key: &str, _value: &str, _ttl_secs: u64) {}
    async fn delete(&self, _key: &str) {}
    async fn delete_prefix(&self, _prefix: &str) {}
}

/// In-memory cache for tests. TTLs are recorded, not enforced.
#[derive(Default)]
pub struct InMemoryCache {
    entries: Mutex<HashMap<String, (String, u64)>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        InMemoryCache::default()
    }

    pub fn keys(&self) -> Vec<String> {
        let guard = self.entries.lock().unwrap();
        let mut keys: Vec<String> = guard.keys().cloned().collect();
        keys.sort();
        keys
    }

    pub fn ttl_of(&self, key: &str) -> Option<u64> {
        let guard = self.entries.lock().unwrap();
        guard.get(key).map(|(_, ttl)| *ttl)
    }

    /// Replace a stored value without going through the trait (to prove cache hits).
    pub fn put_raw(&self, key: &str, value: &str) {
        let mut guard = self.entries.lock().unwrap();
        guard.insert(key.to_string(), (value.to_string(), 60));
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        let guard = self.entries.lock().unwrap();
        guard.get(key).map(|(value, _)| value.clone())
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) {
        let mut guard = self.entries.lock().unwrap();
        guard.insert(key.to_string(), (value.to_string(), ttl_secs));
    }

    async fn delete(&self, key: &str) {
        let mut guard = self.entries.lock().unwrap();
        guard.remove(key);
    }

    async fn delete_prefix(&self, prefix: &str) {
        let mut guard = self.entries.lock().unwrap();
        guard.retain(|key, _| !key.starts_with(prefix));
    }
}
