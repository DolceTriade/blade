use anyhow::{anyhow, Context};
use diesel::prelude::*;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};

mod models;
#[allow(non_snake_case)]
mod schema;

#[allow(dead_code)]
pub struct Sqlite {
    pub(crate) conn: diesel::sqlite::SqliteConnection,
}

#[allow(dead_code)]
impl Sqlite {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        diesel::SqliteConnection::establish(path)
            .map(|conn| Self { conn })
            .context("creating sqlite db")
    }

    pub fn run_migrations(&mut self) -> anyhow::Result<()> {
        let r = runfiles::Runfiles::create().expect("Must run using bazel with runfiles");
        let path = r.rlocation("blade/blade/db/sqlite/migrations");
        let finder: FileBasedMigrations = FileBasedMigrations::from_path(
            path.to_str()
                .ok_or(anyhow!("failed to convert path to str: {path:#?}"))?,
        )
        .map_err(|e| anyhow!("failed to run migrations: {e:#?}"))?;
        MigrationHarness::run_pending_migrations(&mut self.conn, finder)
            .map(|_| {})
            .map_err(|e| anyhow!("failed to run migrations: {e:#?}"))
    }
}

impl crate::DB for Sqlite {
    fn upsert_invocation(&mut self, invocation: &state::InvocationResults) -> anyhow::Result<()> {
        use schema::Invocations::dsl::*;
        let val = models::Invocation::from_state(invocation)?;
        diesel::insert_into(schema::Invocations::table)
            .values(&val)
            .on_conflict(id)
            .do_update()
            .set(&val)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to insert invocation")
    }

    fn upsert_target(&mut self, _id: &str, _target: &state::Target) -> anyhow::Result<()> {
        todo!()
    }

    fn upsert_test(&mut self, _id: &str, _test: &state::Test) -> anyhow::Result<()> {
        todo!()
    }

    fn upsert_test_run(&mut self, _id: &str, _run: &state::TestRun) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::DB;
    use diesel::prelude::*;

    use super::schema;

    #[test]
    fn test_migration() {
        let tmp = tempdir::TempDir::new("test_invocation").unwrap();
        let db_path = tmp.path().join("test.db");
        let mut db = super::Sqlite::new(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
    }

    #[test]
    fn test_invocation() {
        let tmp = tempdir::TempDir::new("test_invocation").unwrap();
        let db_path = tmp.path().join("test.db");
        let mut db = super::Sqlite::new(db_path.to_str().unwrap()).unwrap();
        db.run_migrations().unwrap();
        let mut inv = state::InvocationResults {
            id: "blah".to_string(),
            output: "whatever".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            ..Default::default()
        };
        db.upsert_invocation(&inv).unwrap();
        {
            let res = schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .filter(schema::Invocations::id.eq("blah"))
                .get_result(&mut db.conn)
                .unwrap();
            assert_eq!(res.id, inv.id);
            assert_eq!(res.output, inv.output);
        }
        inv.output.push_str("more output");
        db.upsert_invocation(&inv).unwrap();
        {
            let res = schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .filter(schema::Invocations::id.eq("blah"))
                .get_result(&mut db.conn)
                .unwrap();
            assert_eq!(res.id, inv.id);
            assert_eq!(res.output, inv.output);
        }
    }
}
