use std::collections::HashMap;

use anyhow::{anyhow, Context};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use r2d2::PooledConnection;

mod models;
#[allow(non_snake_case)]
mod schema;

#[allow(dead_code)]
pub struct Sqlite {
    pub(crate) conn: PooledConnection<ConnectionManager<SqliteConnection>>,
}

impl Sqlite {
    pub fn new(
        mut conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> anyhow::Result<Self> {
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .context("failed to enable foreign keys")?;
        Ok(Self { conn })
    }
}

#[allow(dead_code)]
pub fn init_db(db_path: &str) -> anyhow::Result<()> {
    let mut me = diesel::SqliteConnection::establish(db_path).context("creating sqlite db")?;
    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(&mut me)
        .context("failed to enable foreign keys")?;
    let r = runfiles::Runfiles::create().expect("Must run using bazel with runfiles");
    let path = r.rlocation("_main/blade/db/sqlite/migrations");
    let finder: FileBasedMigrations = FileBasedMigrations::from_path(
        path.to_str()
            .ok_or(anyhow!("failed to convert path to str: {path:#?}"))?,
    )
    .map_err(|e| anyhow!("failed to run migrations: {e:#?}"))?;
    MigrationHarness::run_pending_migrations(&mut me, finder)
        .map(|_| {})
        .map_err(|e| anyhow!("failed to run migrations: {e:#?}"))
}

impl state::DB for Sqlite {
    fn upsert_shallow_invocation(
        &mut self,
        invocation: &state::InvocationResults,
    ) -> anyhow::Result<()> {
        use schema::Invocations::dsl::*;
        let val = models::Invocation::from_state(invocation)?;
        diesel::insert_into(schema::Invocations::table)
            .values(&val)
            .on_conflict(id)
            .do_update()
            .set(&val)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to upsert invocation")
    }

    fn upsert_target(&mut self, inv_id: &str, target: &state::Target) -> anyhow::Result<()> {
        use schema::Targets::dsl::*;
        let val = models::Target::from_state(inv_id, target)?;
        diesel::insert_into(schema::Targets::table)
            .values(&val)
            .on_conflict(id)
            .do_update()
            .set(&val)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to upsert target")
    }

    fn upsert_test(&mut self, inv_id: &str, test: &state::Test) -> anyhow::Result<String> {
        use schema::Tests::dsl::*;
        let val = models::Test::from_state(inv_id, test)?;
        diesel::insert_into(schema::Tests::table)
            .values(&val)
            .on_conflict(id)
            .do_update()
            .set(&val)
            .get_result(&mut self.conn)
            .map(|r: models::Test| r.id)
            .context("failed to upsert test")
    }

    fn insert_test_run(
        &mut self,
        inv_id: &str,
        test_id_: &str,
        test_run: &state::TestRun,
    ) -> anyhow::Result<()> {
        let val = models::TestRun::from_state(inv_id, test_id_, test_run)?;
        diesel::insert_into(schema::TestRuns::table)
            .values(&val)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to upsert testrun")?;
        let artifacts = test_run
            .files
            .iter()
            .map(|e| models::TestArtifact::from_state(inv_id, &val.id, e.0, e.1))
            .collect::<Vec<_>>();
        diesel::insert_into(schema::TestArtifacts::table)
            .values(artifacts.as_slice())
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to insert test artifacts")?;
        Ok(())
    }

    fn get_invocation(&mut self, id: &str) -> anyhow::Result<state::InvocationResults> {
        let mut ret = schema::Invocations::table
            .select(models::Invocation::as_select())
            .find(id)
            .get_result(&mut self.conn)
            .map(|res| -> anyhow::Result<state::InvocationResults> {
                Ok(state::InvocationResults {
                    id: res.id.to_string(),
                    status: state::Status::parse(&res.status),
                    output: res.output,
                    start: res.start.into(),
                    command: res.command,
                    pattern: res
                        .pattern
                        .unwrap_or_default()
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                    ..Default::default()
                })
            })?
            .context("failed to get invocation")?;
        let targets = schema::Targets::table
            .select(models::Target::as_select())
            .filter(schema::Targets::dsl::invocation_id.eq(id))
            .load(&mut self.conn)?;
        targets.iter().for_each(|res| {
            ret.targets.insert(
                res.name.clone(),
                state::Target {
                    name: res.name.clone(),
                    status: state::Status::parse(&res.status),
                    kind: res.kind.clone(),
                    start: crate::time::to_systemtime(&res.start)
                        .unwrap_or_else(|_| std::time::SystemTime::now()),
                    end: res.end.as_ref().map(|t| {
                        crate::time::to_systemtime(t)
                            .unwrap_or_else(|_| std::time::SystemTime::now())
                    }),
                },
            );
        });
        let tests = schema::Tests::table
            .select(models::Test::as_select())
            .filter(schema::Tests::invocation_id.eq(id))
            .load(&mut self.conn)?;

        let test_runs = models::TestRun::belonging_to(&tests)
            .select(models::TestRun::as_select())
            .load(&mut self.conn)?;

        let mut test_artifacts: std::collections::VecDeque<_> = schema::TestArtifacts::table
            .select(models::TestArtifact::as_select())
            .filter(schema::TestArtifacts::dsl::invocation_id.eq(id))
            .load(&mut self.conn)?
            .grouped_by(&test_runs)
            .into();
        let test_runs = test_runs.grouped_by(&tests);
        tests.into_iter().zip(test_runs).for_each(|(test, trs)| {
            ret.tests.insert(
                test.name.clone(),
                state::Test {
                    name: test.name,
                    status: state::Status::parse(&test.status),
                    duration: std::time::Duration::from_secs_f64(test.duration_s.unwrap_or(0.0)),
                    end: crate::time::to_systemtime(&test.end)
                        .unwrap_or_else(|_| std::time::SystemTime::now()),
                    num_runs: test.num_runs.map(|nr| nr as usize).unwrap_or(0),
                    runs: trs
                        .into_iter()
                        .map(|tr| state::TestRun {
                            run: tr.run,
                            shard: tr.shard,
                            attempt: tr.attempt,
                            status: state::Status::parse(&tr.status),
                            details: tr.details,
                            duration: std::time::Duration::from_secs_f64(tr.duration_s),
                            files: test_artifacts
                                .pop_front()
                                .map(|v| {
                                    v.into_iter()
                                        .map(|ta| {
                                            (
                                                ta.name,
                                                state::Artifact {
                                                    size: 0,
                                                    uri: ta.uri,
                                                },
                                            )
                                        })
                                        .collect::<HashMap<_, _>>()
                                })
                                .unwrap_or_default(),
                        })
                        .collect::<Vec<_>>(),
                },
            );
        });
        Ok(ret)
    }

    fn update_shallow_invocation(
        &mut self,
        invocation_id: &str,
        upd: Box<dyn FnOnce(&mut state::InvocationResults) -> anyhow::Result<()>>,
    ) -> anyhow::Result<()> {
        let mut ret = schema::Invocations::table
            .select(models::Invocation::as_select())
            .filter(schema::Invocations::id.eq(invocation_id))
            .get_result(&mut self.conn)
            .map(|res| res.into_state())?;
        upd(&mut ret)?;
        self.upsert_shallow_invocation(&ret)
    }

    fn get_progress(&mut self, invocation_id: &str) -> anyhow::Result<String> {
        schema::Invocations::table
            .select(models::Invocation::as_select())
            .filter(schema::Invocations::id.eq(invocation_id))
            .get_result(&mut self.conn)
            .map(|res: models::Invocation| res.output)
            .context("failed to get progress")
    }

    fn update_target_result(
        &mut self,
        invocation_id: &str,
        name: &str,
        status: state::Status,
        end: std::time::SystemTime,
    ) -> anyhow::Result<()> {
        let id = models::Target::gen_id(invocation_id, name);
        let mut res: models::Target = schema::Targets::table
            .select(models::Target::as_select())
            .find(id.clone())
            .get_result(&mut self.conn)?;
        res.status = status.to_string();
        res.end = Some(end.into());
        diesel::update(schema::Targets::table.find(id))
            .set(&res)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to update target result")
    }

    fn get_test(&mut self, id: &str, name: &str) -> anyhow::Result<state::Test> {
        let t = schema::Tests::table
            .select(models::Test::as_select())
            .find(models::Test::gen_id(id, name))
            .get_result(&mut self.conn)?;
        Ok(t.into_state())
    }

    fn update_test_result(
        &mut self,
        invocation_id: &str,
        name: &str,
        status: state::Status,
        duration: std::time::Duration,
        num_runs: usize,
    ) -> anyhow::Result<()> {
        let id = models::Test::gen_id(invocation_id, name);
        let mut t: models::Test = schema::Tests::table
            .select(models::Test::as_select())
            .find(id.clone())
            .get_result(&mut self.conn)?;
        t.status = status.to_string();
        t.duration_s = Some(duration.as_secs_f64());
        t.num_runs = Some(num_runs as i32);
        diesel::update(schema::Tests::dsl::Tests.find(id))
            .set(&t)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to update test result")
    }
    fn delete_invocation(&mut self, id: &str) -> anyhow::Result<()> {
        diesel::delete(schema::Invocations::table.find(id))
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to delete invocation")
    }

    fn delete_invocations_since(&mut self, ts: &std::time::SystemTime) -> anyhow::Result<()> {
        let ot: time::OffsetDateTime = (*ts).into();
        diesel::delete(
            schema::Invocations::table
                .filter(unixepoch(schema::Invocations::start).le(unixepoch(ot))),
        )
        .execute(&mut self.conn)
        .map(|_| {})
        .context(format!("failed to delete invocation since {:#?}", ot))
    }
}

sql_function! {
    fn unixepoch(ts: diesel::sql_types::TimestamptzSqlite) -> diesel::sql_types::Integer;
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        time::{Duration, UNIX_EPOCH},
    };

    use diesel::prelude::*;

    use super::schema;

    #[test]
    fn test_migration() {
        let tmp = tempdir::TempDir::new("test_invocation").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_invocation() {
        let tmp = tempdir::TempDir::new("test_invocation").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();
        let mut conn = SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
        let mut inv = state::InvocationResults {
            id: "blah".to_string(),
            output: "whatever".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            ..Default::default()
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        {
            let res = schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .filter(schema::Invocations::id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(res.id, inv.id);
            assert_eq!(res.output, inv.output);
        }
        inv.output.push_str("more output");
        db.upsert_shallow_invocation(&inv).unwrap();
        {
            let res = schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .filter(schema::Invocations::id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(res.id, inv.id);
            assert_eq!(res.output, inv.output);
        }
    }

    #[test]
    fn test_target() {
        let tmp = tempdir::TempDir::new("test_target").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mut conn = SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let inv = state::InvocationResults {
            id: "blah".to_string(),
            output: "whatever".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            ..Default::default()
        };
        let mut target = state::Target {
            name: "//target/path:thing".to_string(),
            status: state::Status::InProgress,
            kind: "real_rule".to_string(),
            start: std::time::SystemTime::now(),
            end: None,
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        db.upsert_target("blah", &target).unwrap();
        {
            let res = schema::Targets::table
                .select(super::models::Target::as_select())
                .filter(schema::Targets::invocation_id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(target.name, res.name);
            assert_eq!(target.status.to_string(), res.status);
            assert!(res.end.is_none());
        }
        target.status = state::Status::Success;
        target.end = Some(std::time::SystemTime::now());
        db.upsert_target("blah", &target).unwrap();
        {
            let res = schema::Targets::table
                .select(super::models::Target::as_select())
                .filter(schema::Targets::invocation_id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(target.name, res.name);
            assert_eq!(target.status.to_string(), res.status);
            assert!(res.end.is_some());
        }
        let targets = super::schema::Targets::table
            .select(super::models::Target::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(targets.len(), 1);
        let invs = super::schema::Invocations::table
            .select(super::models::Invocation::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(invs.len(), 1);
    }

    #[test]
    fn test_test() {
        let tmp = tempdir::TempDir::new("test_test").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut conn = SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let inv = state::InvocationResults {
            id: "blah".to_string(),
            output: "whatever".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            ..Default::default()
        };
        let mut test = state::Test {
            name: "//target/path:thing".to_string(),
            status: state::Status::InProgress,
            duration: std::time::Duration::from_secs_f64(4.343),
            end: std::time::SystemTime::now(),
            num_runs: 0,
            runs: vec![],
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        db.upsert_test("blah", &test).unwrap();
        {
            let res = schema::Tests::table
                .select(super::models::Test::as_select())
                .filter(schema::Tests::invocation_id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(test.name, res.name);
            assert_eq!(test.status.to_string(), res.status);
        }
        test.status = state::Status::Success;
        db.upsert_test("blah", &test).unwrap();
        {
            let res = schema::Tests::table
                .select(super::models::Test::as_select())
                .filter(schema::Tests::invocation_id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(test.name, res.name);
            assert_eq!(test.status.to_string(), res.status);
        }
        let tests = super::schema::Tests::table
            .select(super::models::Test::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(tests.len(), 1);
        let invs = super::schema::Invocations::table
            .select(super::models::Invocation::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(invs.len(), 1);
    }

    #[test]
    fn test_all() {
        let tmp = tempdir::TempDir::new("test_invocation").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let mut inv = state::InvocationResults {
            id: "blah".to_string(),
            output: "whatever".to_string(),
            command: "test".to_string(),
            status: state::Status::InProgress,
            start: std::time::SystemTime::now(),
            end: None,
            pattern: vec!["//...".to_string()],
            targets: HashMap::from([
                (
                    "//target1".to_string(),
                    state::Target {
                        name: "//target1".to_string(),
                        status: state::Status::Success,
                        kind: "real_rule".to_string(),
                        start: std::time::SystemTime::now(),
                        end: Some(std::time::SystemTime::now()),
                    },
                ),
                (
                    "//target1:some_test".to_string(),
                    state::Target {
                        name: "//target1:some_test".to_string(),
                        status: state::Status::Success,
                        kind: "real_test".to_string(),
                        start: std::time::SystemTime::now(),
                        end: Some(std::time::SystemTime::now()),
                    },
                ),
            ]),
            tests: HashMap::from([(
                "//target1:some_test".to_string(),
                state::Test {
                    name: "//target1:some_test".to_string(),
                    status: state::Status::Fail,
                    duration: std::time::Duration::from_secs(5),
                    end: std::time::SystemTime::now(),
                    num_runs: 2,
                    runs: vec![
                        state::TestRun {
                            run: 1,
                            shard: 1,
                            attempt: 1,
                            status: state::Status::Success,
                            details: "".to_string(),
                            duration: std::time::Duration::from_secs(5),
                            files: HashMap::from([
                                (
                                    "test.log".to_string(),
                                    state::Artifact {
                                        size: 0,
                                        uri: "file://path/to/test.log".to_string(),
                                    },
                                ),
                                (
                                    "test.xml".to_string(),
                                    state::Artifact {
                                        size: 0,
                                        uri: "file://path/to/test.xml".to_string(),
                                    },
                                ),
                            ]),
                        },
                        state::TestRun {
                            run: 2,
                            shard: 1,
                            attempt: 1,
                            status: state::Status::Fail,
                            details: "".to_string(),
                            duration: std::time::Duration::from_secs(2),
                            files: HashMap::from([
                                (
                                    "test.log".to_string(),
                                    state::Artifact {
                                        size: 0,
                                        uri: "file://path2/to/test.log".to_string(),
                                    },
                                ),
                                (
                                    "test.xml".to_string(),
                                    state::Artifact {
                                        size: 0,
                                        uri: "file://path2/to/test.xml".to_string(),
                                    },
                                ),
                            ]),
                        },
                    ],
                },
            )]),
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        let old = inv.id;
        inv.id = "another".to_string();
        db.upsert_shallow_invocation(&inv).unwrap();
        inv.id = old;
        inv.targets.iter().for_each(|t| {
            db.upsert_target(&inv.id, t.1).unwrap();
            db.upsert_target("another", t.1).unwrap();
        });
        inv.tests.iter().for_each(|t| {
            let t_id = db.upsert_test(&inv.id, t.1).unwrap();
            t.1.runs.iter().for_each(|r| {
                db.insert_test_run(&inv.id, &t_id, r).unwrap();
            })
        });
        let new_inv = db.get_invocation("blah").unwrap();
        assert_eq!(new_inv.id, inv.id);
        assert_eq!(new_inv.tests.len(), inv.tests.len());
        assert_eq!(new_inv.targets.len(), inv.targets.len());
        let _ = db.get_test("blah", "//target1:some_test").unwrap();
        db.delete_invocation("blah").unwrap();
        let _ = db.get_invocation("blah").unwrap_err();
        let _ = db.get_test("blah", "//target1:some_test").unwrap_err();
    }

    #[test]
    fn test_delete_since() {
        let tmp = tempdir::TempDir::new("test_target").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mut conn = SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let start = UNIX_EPOCH;
        let mut curr = start;
        let day = Duration::from_secs(60 * 60 * 24);
        for i in 0..5 {
            db.upsert_shallow_invocation(&state::InvocationResults {
                id: format!("id{}", i),
                start: curr.checked_add(day).unwrap(),
                ..Default::default()
            })
            .unwrap();
            curr += day;
        }
        {
            let res = super::schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .get_results(&mut conn)
                .unwrap();
            assert_eq!(res.len(), 5);
        }

        curr = start;
        for i in 0..5 {
            db.delete_invocations_since(&curr).unwrap();
            let res = super::schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .get_results(&mut conn)
                .unwrap();
            assert_eq!(res.len(), 5 - i);
            curr += day;
        }
    }
}
