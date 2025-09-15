use anyhow::anyhow;

mod envscrub;
mod exec;
mod manager;
mod postgres;
mod sqlite;
mod time;

pub use exec::{run, run_group, transaction};

pub fn new(uri: &str) -> anyhow::Result<std::sync::Arc<dyn state::DBManager>> {
    if uri.starts_with("postgres://") {
        return Ok(std::sync::Arc::from(manager::PostgresManager::new(uri)?));
    }
    if uri.starts_with("sqlite://") {
        return Ok(std::sync::Arc::from(manager::SqliteManager::new(uri)?));
    }
    Err(anyhow!("unknown database implementation: {}", uri))
}
