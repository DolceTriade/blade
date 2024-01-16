use anyhow::anyhow;

mod manager;
mod postgres;
mod sqlite;
mod time;

pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
    if uri.starts_with("postgres://") {
        return manager::PostgresManager::new(uri);
    }
    if uri.starts_with("sqlite://") {
        return manager::SqliteManager::new(uri);
    }
    Err(anyhow!("unknown database implementation: {}", uri))
}
