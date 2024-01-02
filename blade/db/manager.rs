use anyhow::Context;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

#[allow(dead_code)]
pub struct SqliteManager {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

#[allow(dead_code)]
impl SqliteManager {
    pub fn new(uri: &str) -> anyhow::Result<Self> {
        crate::sqlite::init_db(uri)?;
        let manager = ConnectionManager::<SqliteConnection>::new(uri);
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .context("failed to build db connection pool")?;
        Ok(Self { pool })
    }
}

impl state::DBManager for SqliteManager {
    fn get(&self) -> anyhow::Result<Box<dyn state::DB>> {
        let conn = self
            .pool
            .get()
            .context("failed to get connection from pool")?;
        Ok(Box::new(crate::sqlite::Sqlite { conn }))
    }
}
