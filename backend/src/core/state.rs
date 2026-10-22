use std::sync::Arc;

use crate::core::cache::Cache;
use crate::core::config::Settings;
use crate::core::jobs::JobQueue;
use crate::modules::api::base::users::repository::UserRepository;

/// Shared application state. Every dependency is a trait object so tests can
/// inject in-memory fakes.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Settings>,
    pub users: Arc<dyn UserRepository>,
    pub cache: Arc<dyn Cache>,
    pub jobs: Arc<dyn JobQueue>,
}

impl AppState {
    pub fn new(
        config: Arc<Settings>,
        users: Arc<dyn UserRepository>,
        cache: Arc<dyn Cache>,
        jobs: Arc<dyn JobQueue>,
    ) -> Self {
        AppState {
            config,
            users,
            cache,
            jobs,
        }
    }
}
