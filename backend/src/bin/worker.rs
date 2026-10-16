use xerxes_backend::core::config::Settings;
use xerxes_backend::core::init_tracing;
use xerxes_backend::core::jobs::run_worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::from_env()?;
    run_worker(&settings.redis_url()).await
}
