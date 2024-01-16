use anyhow::Context;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

pub struct SqliteManager {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteManager {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
        crate::sqlite::init_db(uri)?;
        let manager = ConnectionManager::<SqliteConnection>::new(uri);
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .context("failed to build db connection pool")?;
        Ok(Box::new(Self { pool }))
    }
}

impl state::DBManager for SqliteManager {
    fn get(&self) -> anyhow::Result<Box<dyn state::DB>> {
        let conn = self
            .pool
            .get()
            .context("failed to get connection from pool")?;
        Ok(Box::new(crate::sqlite::Sqlite::new(conn)?))
    }
}

pub struct PostgresManager {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresManager {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
        crate::postgres::init_db(uri)?;
        let manager = ConnectionManager::<PgConnection>::new(uri);
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .context("failed to build db connection pool")?;
        Ok(Box::new(Self { pool }))
    }
}

impl state::DBManager for PostgresManager {
    fn get(&self) -> anyhow::Result<Box<dyn state::DB>> {
        let conn = self
            .pool
            .get()
            .context("failed to get connection from pool")?;
        Ok(Box::new(crate::postgres::Postgres { conn }))
    }
}
