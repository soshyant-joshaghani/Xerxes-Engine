//! Postgres pool and plain-SQL migrations.

use std::path::PathBuf;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

use crate::core::config::Settings;
use crate::core::error::ApiError;

/// Map a database error to a 500 (details are logged, not returned).
pub fn db_err(err: sqlx::Error) -> ApiError {
    ApiError::internal(err)
}

/// Connect to Postgres, retrying for up to about a minute while it starts.
pub async fn connect_pool(settings: &Settings) -> anyhow::Result<PgPool> {
    let options = PgConnectOptions::new()
        .host(&settings.postgres_server)
        .port(settings.postgres_port)
        .username(&settings.postgres_user)
        .password(&settings.postgres_password)
        .database(&settings.postgres_db);

    let mut last_error: Option<sqlx::Error> = None;
    for attempt in 1..=30u32 {
        let result = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options.clone())
            .await;
        match result {
            Ok(pool) => return Ok(pool),
            Err(err) => {
                tracing::warn!("postgres not ready (attempt {}/30): {}", attempt, err);
                last_error = Some(err);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    match last_error {
        Some(err) => Err(anyhow::anyhow!("could not connect to postgres: {}", err)),
        None => Err(anyhow::anyhow!("could not connect to postgres")),
    }
}

/// `MIGRATIONS_DIR`, then `./migrations`, then `../migrations` (and `./backend/migrations`).
pub fn find_migrations_dir(explicit: Option<&str>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = explicit {
        candidates.push(PathBuf::from(dir));
    }
    candidates.push(PathBuf::from("./migrations"));
    candidates.push(PathBuf::from("../migrations"));
    candidates.push(PathBuf::from("./backend/migrations"));
    candidates.into_iter().find(|p| p.is_dir())
}

/// Apply every `*.sql` file in name order that is not yet in `schema_migrations`.
pub async fn run_migrations(pool: &PgPool, dir: &PathBuf) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\
         name text PRIMARY KEY, \
         applied_at timestamptz NOT NULL DEFAULT now())",
    )
    .execute(pool)
    .await?;

    let applied: Vec<(String,)> =
        sqlx::query_as::<_, (String,)>("SELECT name FROM schema_migrations")
            .fetch_all(pool)
            .await?;
    let applied: Vec<String> = applied.into_iter().map(|(name,)| name).collect();

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("sql") {
            files.push(path);
        }
    }
    files.sort();

    for path in files {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if applied.contains(&name) {
            continue;
        }
        let sql = std::fs::read_to_string(&path)?;
        tracing::info!("applying migration {}", name);
        sqlx::raw_sql(sql.as_str()).execute(pool).await?;
        sqlx::query(
            "INSERT INTO schema_migrations (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
        )
        .bind(name.as_str())
        .execute(pool)
        .await?;
    }
    Ok(())
}
