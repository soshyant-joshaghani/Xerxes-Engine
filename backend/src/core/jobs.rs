//! Redis list job protocol shared by the API (enqueue) and the worker (BRPOP).
//!
//! Queue: list `foxg:jobs`. Payload: `{"id","task","args","enqueued_at"}` and, on
//! retries, `"attempt": n`.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{SecondsFormat, Utc};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::cache::RedisPool;

pub const QUEUE_KEY: &str = "foxg:jobs";
/// A failed task is re-pushed at most this many times.
pub const MAX_RETRIES: u32 = 3;

fn is_zero(n: &u32) -> bool {
    *n == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub task: String,
    #[serde(default)]
    pub args: serde_json::Value,
    pub enqueued_at: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub attempt: u32,
}

impl Job {
    pub fn new(task: &str, args: serde_json::Value) -> Job {
        Job {
            id: Uuid::new_v4().to_string(),
            task: task.to_string(),
            args,
            enqueued_at: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            attempt: 0,
        }
    }
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueue a job and return its id. The error is a human readable message.
    async fn enqueue(&self, task: &str, args: serde_json::Value) -> Result<String, String>;
}

pub struct RedisJobQueue {
    pool: Arc<RedisPool>,
}

impl RedisJobQueue {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        RedisJobQueue { pool }
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue(&self, task: &str, args: serde_json::Value) -> Result<String, String> {
        let job = Job::new(task, args);
        let payload = serde_json::to_string(&job).map_err(|e| e.to_string())?;
        let mut conn = self.pool.connection().await?;
        let result: redis::RedisResult<()> = redis::cmd("LPUSH")
            .arg(QUEUE_KEY)
            .arg(payload)
            .query_async(&mut conn)
            .await;
        match result {
            Ok(()) => Ok(job.id),
            Err(err) => Err(err.to_string()),
        }
    }
}

/// In-memory queue for tests.
#[derive(Default)]
pub struct InMemoryJobQueue {
    jobs: Mutex<Vec<Job>>,
    fail_with: Mutex<Option<String>>,
}

impl InMemoryJobQueue {
    pub fn new() -> Self {
        InMemoryJobQueue::default()
    }

    /// Make every enqueue fail with this message (simulates Redis down).
    pub fn fail_with(&self, message: &str) {
        *self.fail_with.lock().unwrap() = Some(message.to_string());
    }

    pub fn jobs(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }
}

#[async_trait]
impl JobQueue for InMemoryJobQueue {
    async fn enqueue(&self, task: &str, args: serde_json::Value) -> Result<String, String> {
        let failure = self.fail_with.lock().unwrap().clone();
        if let Some(message) = failure {
            return Err(message);
        }
        let job = Job::new(task, args);
        let id = job.id.clone();
        self.jobs.lock().unwrap().push(job);
        Ok(id)
    }
}

// ---------------------------------------------------------------------------
// Worker
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum TaskError {
    Unknown(String),
    Failed(String),
}

/// Run one task by name.
pub async fn run_task(job: &Job) -> Result<(), TaskError> {
    match job.task.as_str() {
        "ping" => {
            let message = job
                .args
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("pong");
            tracing::info!("ping job received: {}", message);
            Ok(())
        }
        other => Err(TaskError::Unknown(other.to_string())),
    }
}

async fn brpop(conn: &mut MultiplexedConnection) -> redis::RedisResult<Option<String>> {
    let popped: Option<(String, String)> = redis::cmd("BRPOP")
        .arg(QUEUE_KEY)
        .arg(5)
        .query_async(conn)
        .await?;
    Ok(popped.map(|(_, payload)| payload))
}

async fn repush(conn: &mut MultiplexedConnection, job: &Job) -> redis::RedisResult<()> {
    let payload = serde_json::to_string(job).unwrap_or_default();
    let result: redis::RedisResult<()> = redis::cmd("LPUSH")
        .arg(QUEUE_KEY)
        .arg(payload)
        .query_async(conn)
        .await;
    result
}

async fn handle_payload(conn: &mut MultiplexedConnection, raw: &str) {
    let mut job: Job = match serde_json::from_str(raw) {
        Ok(job) => job,
        Err(err) => {
            tracing::warn!("dropping malformed job payload: {}", err);
            return;
        }
    };
    match run_task(&job).await {
        Ok(()) => tracing::info!("job {} ({}) done", job.id, job.task),
        Err(TaskError::Unknown(name)) => {
            tracing::warn!("job {}: unknown task '{}', dropped", job.id, name);
        }
        Err(TaskError::Failed(reason)) => {
            if job.attempt < MAX_RETRIES {
                job.attempt += 1;
                tracing::warn!(
                    "job {} ({}) failed: {}; retry {}/{}",
                    job.id,
                    job.task,
                    reason,
                    job.attempt,
                    MAX_RETRIES
                );
                if let Err(err) = repush(conn, &job).await {
                    tracing::error!("could not re-push job {}: {}", job.id, err);
                }
            } else {
                tracing::error!(
                    "job {} ({}) failed permanently: {}",
                    job.id,
                    job.task,
                    reason
                );
            }
        }
    }
}

/// Blocking worker loop: `BRPOP foxg:jobs 5`. Returns on Ctrl+C / SIGTERM.
pub async fn run_worker(redis_url: &str) -> anyhow::Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut shutdown = Box::pin(crate::core::shutdown_signal());
    tracing::info!("worker started, waiting for jobs on {}", QUEUE_KEY);

    loop {
        let connected = tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("worker shutting down");
                return Ok(());
            }
            result = client.get_multiplexed_async_connection() => result,
        };
        let mut conn = match connected {
            Ok(conn) => conn,
            Err(err) => {
                tracing::warn!("redis unavailable ({}); retrying in 2s", err);
                tokio::select! {
                    _ = &mut shutdown => {
                        tracing::info!("worker shutting down");
                        return Ok(());
                    }
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                }
                continue;
            }
        };

        loop {
            let popped = tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("worker shutting down");
                    return Ok(());
                }
                result = brpop(&mut conn) => result,
            };
            match popped {
                Ok(Some(raw)) => handle_payload(&mut conn, &raw).await,
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!("redis error ({}); reconnecting", err);
                    break;
                }
            }
        }

        tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("worker shutting down");
                return Ok(());
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}
