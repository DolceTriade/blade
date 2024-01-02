mod sqlite;
mod manager;

pub fn new(uri: &str) -> anyhow::Result<impl state::DBManager + Send + Sync> {
    manager::SqliteManager::new(uri)
}