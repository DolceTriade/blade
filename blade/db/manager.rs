use anyhow::Context;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_tracing::{pg::InstrumentedPgConnection, sqlite::InstrumentedSqliteConnection};

pub struct SqliteManager {
    pool: Pool<ConnectionManager<InstrumentedSqliteConnection>>,
}

impl SqliteManager {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
        crate::sqlite::init_db(uri)?;
        let manager = ConnectionManager::<InstrumentedSqliteConnection>::new(uri);
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
    pool: Pool<ConnectionManager<InstrumentedPgConnection>>,
}

impl PostgresManager {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(uri: &str) -> anyhow::Result<Box<dyn state::DBManager>> {
        crate::postgres::init_db(uri)?;
        let manager = ConnectionManager::<InstrumentedPgConnection>::new(uri);
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
