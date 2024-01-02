use anyhow::anyhow;

mod sqlite;
mod postgres;
mod manager;

pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
    if uri.starts_with("postgres://") {
        return manager::PostgresManager::new(uri)
    }
    if uri.starts_with("sqlite://") {
        return manager::SqliteManager::new(uri)
    }
    Err(anyhow!("unknown database implementation: {}", uri))
}