use std::sync::Arc;

use xerxes_backend::build_router;
use xerxes_backend::core::cache::{Cache, RedisCache, RedisPool};
use xerxes_backend::core::config::Settings;
use xerxes_backend::core::db;
use xerxes_backend::core::jobs::{JobQueue, RedisJobQueue};
use xerxes_backend::core::state::AppState;
use xerxes_backend::core::{init_tracing, shutdown_signal};
use xerxes_backend::modules::api::base::users::repository::{PgUserRepository, UserRepository};
use xerxes_backend::modules::api::base::users::service::UserService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let settings = Settings::from_env()?;
    settings.validate()?;

    let pool = db::connect_pool(&settings).await?;
    match db::find_migrations_dir(settings.migrations_dir.as_deref()) {
        Some(dir) => db::run_migrations(&pool, &dir).await?,
        None => anyhow::bail!("migrations directory not found (set MIGRATIONS_DIR)"),
    }

    let redis = Arc::new(RedisPool::new(&settings.redis_url())?);
    let cache: Arc<dyn Cache> = Arc::new(RedisCache::new(redis.clone()));
    let jobs: Arc<dyn JobQueue> = Arc::new(RedisJobQueue::new(redis));
    let users: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool));

    let state = AppState::new(Arc::new(settings.clone()), users, cache, jobs);

    UserService::new(&state)
        .ensure_first_superuser(
            &settings.first_superuser,
            &settings.first_superuser_password,
        )
        .await?;

    let app = build_router(state);

    let addr = format!("{}:{}", settings.app_host, settings.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(
        "{} listening on {} (environment: {})",
        settings.project_name,
        addr,
        settings.environment
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("server stopped");
    Ok(())
}
