use std::collections::HashMap;

use anyhow::{Context, anyhow};
use diesel::{prelude::*, r2d2::ConnectionManager};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::sqlite::InstrumentedSqliteConnection;
use r2d2::PooledConnection;

mod models;
#[allow(non_snake_case)]
mod schema;

#[allow(dead_code)]
pub struct Sqlite {
    pub(crate) conn: PooledConnection<ConnectionManager<InstrumentedSqliteConnection>>,
}

impl Sqlite {
    pub fn new(
        mut conn: PooledConnection<ConnectionManager<InstrumentedSqliteConnection>>,
    ) -> anyhow::Result<Self> {
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .context("failed to enable foreign keys")?;
        Ok(Self { conn })
    }
}

pub fn init_db(db_path: &str) -> anyhow::Result<()> {
    let mut me = InstrumentedSqliteConnection::establish(db_path).context("creating sqlite db")?;
    diesel::sql_query("PRAGMA journal_mode = WAL;")
        .execute(&mut me)
        .context("failed to set WAL mode")?;
    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(&mut me)
        .context("failed to enable foreign keys")?;
    let r = runfiles::Runfiles::create().expect("Must run using bazel with runfiles");
    let path = r.rlocation("_main/blade/db/sqlite/migrations").unwrap();
    let finder: FileBasedMigrations = FileBasedMigrations::from_path(path)
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

    fn upsert_test_run(
        &mut self,
        inv_id: &str,
        test_id_: &str,
        test_run: &state::TestRun,
    ) -> anyhow::Result<()> {
        let val = models::TestRun::from_state(inv_id, test_id_, test_run)?;
        diesel::insert_into(schema::TestRuns::table)
            .values(&val)
            .on_conflict(schema::TestRuns::dsl::id)
            .do_update()
            .set(&val)
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
            .context(anyhow!(
                "failed to insert test artifacts: {:#?}",
                &artifacts
            ))?;
        Ok(())
    }

    fn get_invocation(&mut self, id: &str) -> anyhow::Result<state::InvocationResults> {
        let mut ret = schema::Invocations::table
            .select(models::Invocation::as_select())
            .find(id)
            .get_result(&mut self.conn)
            .map(|res| -> anyhow::Result<state::InvocationResults> {
                Ok(res.into_state())
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

        let mut test_artifacts: HashMap<String, Vec<models::TestArtifact>> = HashMap::new();
        schema::TestArtifacts::table
            .select(models::TestArtifact::as_select())
            .filter(schema::TestArtifacts::dsl::invocation_id.eq(id))
            .load(&mut self.conn)?
            .into_iter()
            .for_each(|a: models::TestArtifact| {
                let v = test_artifacts.entry(a.test_run_id.clone()).or_default();
                v.push(a);
            });
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
                                .get_mut(&tr.id)
                                .map(|v| {
                                    v.drain(..)
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
        match schema::InvocationOutput::table
            .select(schema::InvocationOutput::line)
            .filter(schema::InvocationOutput::invocation_id.eq(invocation_id))
            .order(schema::InvocationOutput::id.asc())
            .load::<String>(&mut self.conn)
        {
            Ok(res) => Ok(res.join("\n")),
            Err(e) => match e {
                diesel::result::Error::NotFound => Ok("".to_string()),
                _ => Err(e).context("failed to get progress"),
            },
        }
    }

    fn delete_last_output_lines(&mut self, id: &str, num_lines: u32) -> anyhow::Result<()> {
        let to_delete = schema::InvocationOutput::table
            .filter(schema::InvocationOutput::invocation_id.eq(id))
            .order(schema::InvocationOutput::id.desc())
            .limit(num_lines.into())
            .select(schema::InvocationOutput::id)
            .load::<i32>(&mut self.conn);
        if let Err(e) = to_delete {
            match e {
                diesel::result::Error::NotFound => {
                    return Ok(());
                },
                _ => {
                    return Err(e.into());
                },
            }
        }
        let to_delete = to_delete.unwrap();
        if to_delete.is_empty() {
            return Ok(());
        }

        diesel::delete(
            schema::InvocationOutput::table.filter(schema::InvocationOutput::id.eq_any(to_delete)),
        )
        .execute(&mut self.conn)?;

        Ok(())
    }

    fn insert_output_lines(&mut self, id: &str, lines: Vec<String>) -> anyhow::Result<()> {
        let input = lines
            .into_iter()
            .map(|l| models::InvocationOutput {
                invocation_id: id.to_string(),
                line: l,
            })
            .collect::<Vec<models::InvocationOutput>>();
        diesel::insert_into(schema::InvocationOutput::table)
            .values(&input)
            .execute(&mut self.conn)
            .map(|_| ())
            .context("failed to insert lines")
    }

    fn get_shallow_invocation(
        &mut self,
        invocation_id: &str,
    ) -> anyhow::Result<state::InvocationResults> {
        schema::Invocations::table
            .select(models::Invocation::as_select())
            .filter(schema::Invocations::id.eq(invocation_id))
            .get_result(&mut self.conn)
            .map(|res| res.into_state())
            .context("failed to get shallow invocation")
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

    fn delete_invocations_since(&mut self, ts: &std::time::SystemTime) -> anyhow::Result<usize> {
        let ot: time::OffsetDateTime = (*ts).into();
        diesel::delete(
            schema::Invocations::table
                .filter(unixepoch(schema::Invocations::start).le(unixepoch(ot))),
        )
        .execute(&mut self.conn)
        .context(format!("failed to delete invocation since {ot:#?}"))
    }

    fn update_invocation_heartbeat(&mut self, invocation_id: &str) -> anyhow::Result<()> {
        use schema::Invocations::dsl::*;
        let now: time::OffsetDateTime = std::time::SystemTime::now().into();

        diesel::update(Invocations.find(invocation_id))
            .set(last_heartbeat.eq(Some(now)))
            .execute(&mut self.conn)
            .map(|_| ())
            .context("failed to update invocation heartbeat")
    }

    fn insert_options(
        &mut self,
        inv_id: &str,
        options: &state::BuildOptions,
    ) -> anyhow::Result<()> {
        use schema::Options::dsl::*;
        let mut vals = vec![];
        let mut vec_helper = |vec: &Vec<String>, kind_: &str| {
            if !vec.is_empty() {
                let uid = uuid::Uuid::new_v4().to_string();
                vec.iter().enumerate().for_each(|(i, v)| {
                    vals.push((
                        id.eq(format!("{uid}-{i:04}")),
                        invocation_id.eq(inv_id.to_string()),
                        kind.eq(kind_.to_string()),
                        keyval.eq(crate::envscrub::scrub(v)),
                    ));
                });
            }
        };
        vec_helper(&options.unstructured, "Unstructured");
        vec_helper(&options.startup, "Startup");
        vec_helper(&options.explicit_startup, "Explicit Startup");
        vec_helper(&options.cmd_line, "Command Line");
        vec_helper(&options.explicit_cmd_line, "Explicit Command Line");
        if !options.structured.is_empty() {
            options.structured.iter().for_each(|(k, vec)| {
                vec_helper(vec, k);
            });
        }
        if !options.build_metadata.is_empty() {
            options.build_metadata.iter().for_each(|(k, v)| {
                let uid = uuid::Uuid::new_v4().to_string();
                vals.push((
                    id.eq(uid),
                    invocation_id.eq(inv_id.to_string()),
                    kind.eq("Build Metadata".to_string()),
                    keyval.eq(format!("{}={}", k.clone(), v.clone()).to_string()),
                ));
            });
        }
        diesel::insert_into(schema::Options::table)
            .values(vals)
            .execute(&mut self.conn)
            .map(|_| {})
            .context("failed to insert options")
    }

    fn get_options(&mut self, id: &str) -> anyhow::Result<state::BuildOptions> {
        let mut opts = state::BuildOptions::default();
        let ret: Vec<_> = schema::Options::table
            .select((schema::Options::kind, schema::Options::keyval))
            .filter(schema::Options::invocation_id.eq(id))
            .order_by(schema::Options::id.asc())
            .load::<(String, String)>(&mut self.conn)?;

        ret.into_iter().for_each(|(kind, keyval)| match &kind[..] {
            "Unstructured" => opts.unstructured.push(keyval),
            "Startup" => opts.startup.push(keyval),
            "Explicit Startup" => opts.explicit_startup.push(keyval),
            "Command Line" => opts.cmd_line.push(keyval),
            "Explicit Command Line" => opts.explicit_cmd_line.push(keyval),
            "Build Metadata" => {
                let Some((k, v)) = keyval.split_once('=') else {
                    return;
                };
                opts.build_metadata.insert(k.to_string(), v.to_string());
            },
            _ => {
                opts.structured
                    .entry(kind)
                    .or_insert_with(Vec::new)
                    .push(keyval);
            },
        });
        Ok(opts)
    }

    fn get_test_history(
        &mut self,
        test_name: &str,
        filters: &[state::TestFilter],
        max_results: usize,
        default_days: Option<u32>,
    ) -> anyhow::Result<state::TestHistory> {
        use schema::*;
        // Helper struct to structure the flat results from our query, combining all
        // necessary models.
        #[derive(Queryable, Debug)]
        struct FullHistoryRun {
            test: models::Test,
            invocation: models::Invocation,
        }

        let max_results_i64: i64 = max_results
            .try_into()
            .context("failed to convert into i64")?;

        // Apply default date range if no DateRange filter is provided
        let mut filters = filters.to_vec();
        let has_date_range = filters
            .iter()
            .any(|f| matches!(f.filter, state::TestFilterItem::DateRange { .. }));

        if let Some(days) = default_days
            && !has_date_range
        {
            let now = std::time::SystemTime::now();
            let from = now
                .checked_sub(std::time::Duration::from_secs(days as u64 * 24 * 60 * 60))
                .unwrap_or(std::time::UNIX_EPOCH);
            filters.push(state::TestFilter {
                op: state::TestFilterOp::GreaterThan,
                invert: false,
                filter: state::TestFilterItem::DateRange { from, to: now },
            });
        }

        // 1. Start with a base query joining the necessary tables. We join testruns ->
        //    tests -> invocations. We box it to allow for dynamic modification based on
        //    filters.
        let mut query = Tests::table
            .inner_join(Invocations::table.on(Tests::invocation_id.eq(Invocations::id)))
            .filter(Tests::name.eq(test_name))
            .into_boxed()
            .select((models::Test::as_select(), models::Invocation::as_select()));

        // 2. Dynamically add filters to the query
        for f in &filters {
            query = match &f.filter {
                state::TestFilterItem::Status(status) => {
                    let db_status = status.to_string();
                    match f.op {
                        state::TestFilterOp::Equals => {
                            if f.invert {
                                query.filter(Tests::status.ne(db_status))
                            } else {
                                query.filter(Tests::status.eq(db_status))
                            }
                        },
                        _ => query, // Other ops are not applicable to Status
                    }
                },
                state::TestFilterItem::Duration(duration) => {
                    let duration_s = duration.as_secs_f64();
                    match f.op {
                        state::TestFilterOp::GreaterThan => {
                            if f.invert {
                                query.filter(Tests::duration_s.le(duration_s))
                            } else {
                                query.filter(Tests::duration_s.gt(duration_s))
                            }
                        },
                        state::TestFilterOp::LessThan => {
                            if f.invert {
                                query.filter(Tests::duration_s.ge(duration_s))
                            } else {
                                query.filter(Tests::duration_s.lt(duration_s))
                            }
                        },
                        _ => query, // Equals/Contains not applicable
                    }
                },
                state::TestFilterItem::Start(time) => {
                    let odt: time::OffsetDateTime = (*time).into();
                    match f.op {
                        state::TestFilterOp::GreaterThan => {
                            if f.invert {
                                query.filter(Invocations::start.le(odt))
                            } else {
                                query.filter(Invocations::start.gt(odt))
                            }
                        },
                        state::TestFilterOp::LessThan => {
                            if f.invert {
                                query.filter(Invocations::start.ge(odt))
                            } else {
                                query.filter(Invocations::start.lt(odt))
                            }
                        },
                        _ => query, // Equals/Contains not applicable
                    }
                },
                state::TestFilterItem::Metadata { key, value } => {
                    // For Build Metadata, the `kind` is "Build Metadata" and the `keyval` is
                    // "key=value"
                    let metadata_kind = "Build Metadata".to_string();
                    let keyval_filter = format!("{key}={value}");

                    let mut subquery = Options::table
                        .into_boxed()
                        .filter(Options::kind.eq(metadata_kind))
                        .select(Options::invocation_id)
                        .distinct();

                    subquery = match f.op {
                        state::TestFilterOp::Equals => {
                            subquery.filter(Options::keyval.eq(keyval_filter))
                        },
                        state::TestFilterOp::Contains => {
                            subquery.filter(Options::keyval.like(format!("%{value}%")))
                        },
                        _ => subquery, // Other ops not applicable
                    };

                    if f.invert {
                        query.filter(diesel::dsl::not(Invocations::id.eq_any(subquery)))
                    } else {
                        query.filter(Invocations::id.eq_any(subquery))
                    }
                },
                state::TestFilterItem::BazelFlags { flag, value } => {
                    // For Bazel flags, we need to search in various option kinds
                    let mut subquery = Options::table
                        .into_boxed()
                        .select(Options::invocation_id)
                        .distinct();

                    subquery = match f.op {
                        state::TestFilterOp::Equals => {
                            if value.is_empty() {
                                // Just check for flag existence (flag=*)
                                subquery.filter(Options::keyval.like(format!("{flag}=%")))
                            } else {
                                // Check for exact flag=value match
                                subquery.filter(Options::keyval.eq(format!("{flag}={value}")))
                            }
                        },
                        state::TestFilterOp::Contains => {
                            if value.is_empty() {
                                // Just check for flag existence
                                subquery.filter(Options::keyval.like(format!("{flag}=%")))
                            } else {
                                // Check for flag containing value
                                subquery.filter(Options::keyval.like(format!("%{flag}%{value}%")))
                            }
                        },
                        _ => subquery, // Other ops not applicable
                    };

                    if f.invert {
                        query.filter(diesel::dsl::not(Invocations::id.eq_any(subquery)))
                    } else {
                        query.filter(Invocations::id.eq_any(subquery))
                    }
                },
                state::TestFilterItem::LogOutput(search_term) => {
                    // Search in invocation output lines
                    let mut subquery = schema::InvocationOutput::table
                        .into_boxed()
                        .select(schema::InvocationOutput::invocation_id)
                        .distinct();

                    subquery = match f.op {
                        state::TestFilterOp::Equals => {
                            subquery.filter(schema::InvocationOutput::line.eq(search_term))
                        },
                        state::TestFilterOp::Contains => subquery.filter(
                            schema::InvocationOutput::line.like(format!("%{search_term}%")),
                        ),
                        _ => subquery, // Other ops not applicable for log output
                    };

                    if f.invert {
                        query.filter(diesel::dsl::not(Invocations::id.eq_any(subquery)))
                    } else {
                        query.filter(Invocations::id.eq_any(subquery))
                    }
                },
                state::TestFilterItem::DateRange { from, to } => {
                    let from_odt: time::OffsetDateTime = (*from).into();
                    let to_odt: time::OffsetDateTime = (*to).into();
                    if f.invert {
                        // Exclude results in this date range
                        query.filter(
                            Invocations::start
                                .lt(from_odt)
                                .or(Invocations::start.gt(to_odt)),
                        )
                    } else {
                        // Include only results in this date range
                        query.filter(
                            Invocations::start
                                .ge(from_odt)
                                .and(Invocations::start.le(to_odt)),
                        )
                    }
                },
            };
        }

        // 3. Apply ordering and limit to the final query
        let results = query
        .order_by(Invocations::start.desc())
        // We query more than the limit because we need to group runs by invocation.
        // This is a pragmatic approach; a more advanced solution might use a window function.
        .limit(max_results_i64 * 5) // Assume max 5 runs per invocation on average
        .load::<FullHistoryRun>(&mut self.conn)
        .context("Failed to load test history")?;

        // 4. Convert the map to the final Vec, sort by time, and apply the limit.
        let mut history: Vec<state::TestHistoryPoint> = results
            .into_iter()
            .map(|item| state::TestHistoryPoint {
                invocation_id: item.invocation.id.clone(),
                start: crate::time::to_systemtime(&item.invocation.start)
                    .unwrap_or_else(|_| std::time::SystemTime::now()),
                test: item.test.into_state(),
            })
            .collect();

        // Sort by the original invocation start time, descending (newest first)
        history.sort_by_key(|item| std::cmp::Reverse(item.start));

        let total_found = history.len();
        let was_truncated = total_found > max_results;
        history.truncate(max_results);

        // Extract query date range from filters
        let query_date_range = filters.iter().find_map(|f| match &f.filter {
            state::TestFilterItem::DateRange { from, to } => Some((*from, *to)),
            _ => None,
        });

        Ok(state::TestHistory {
            name: test_name.to_string(),
            history,
            total_found,
            limit_applied: max_results,
            was_truncated,
            query_date_range,
        })
    }

    fn search_test_names(&mut self, pattern: &str, limit: usize) -> anyhow::Result<Vec<String>> {
        use schema::Tests::dsl::*;

        let limit_i64: i64 = limit.try_into().context("failed to convert limit to i64")?;

        let results = Tests
            .select(name)
            .filter(name.like(format!("%{pattern}%")))
            .distinct()
            .order_by(name.asc())
            .limit(limit_i64)
            .load::<String>(&mut self.conn)
            .context("Failed to search test names")?;

        Ok(results)
    }
}

define_sql_function! {
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
        let inv = state::InvocationResults {
            id: "blah".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            is_live: false,
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
        }
        db.upsert_shallow_invocation(&inv).unwrap();
        {
            let res = schema::Invocations::table
                .select(super::models::Invocation::as_select())
                .filter(schema::Invocations::id.eq("blah"))
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(res.id, inv.id);
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
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            is_live: false,
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
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            is_live: false,
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
            command: "test".to_string(),
            status: state::Status::InProgress,
            start: std::time::SystemTime::now(),
            end: None,
            pattern: vec!["//...".to_string()],
            last_heartbeat: None,
            is_live: false,
            profile_uri: None,
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
                db.upsert_test_run(&inv.id, &t_id, r).unwrap();
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
                id: format!("id{i}"),
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

    #[test]
    fn test_options() {
        let tmp = tempdir::TempDir::new("test_target").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let opts = state::BuildOptions {
            unstructured: vec!["unstructured".to_string()],
            structured: HashMap::from([("key".to_string(), vec!["val".to_string()])]),
            startup: vec!["startup".to_string()],
            explicit_startup: vec!["explicit_startup".to_string()],
            cmd_line: vec!["cmd_line".to_string()],
            explicit_cmd_line: vec!["explicit_cmd_line".to_string()],
            build_metadata: HashMap::from([("key".to_string(), "val".to_string())]),
        };
        let inv = state::InvocationResults {
            id: "blah".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            is_live: false,
            ..Default::default()
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        db.insert_options("blah", &opts).unwrap();
        let res = db.get_options("blah").unwrap();
        assert_eq!(res.unstructured, opts.unstructured);
        assert_eq!(res.structured, opts.structured);
        assert_eq!(res.startup, opts.startup);
        assert_eq!(res.explicit_startup, opts.explicit_startup);
        assert_eq!(res.cmd_line, opts.cmd_line);
        assert_eq!(res.explicit_cmd_line, opts.explicit_cmd_line);
        assert_eq!(res.build_metadata, opts.build_metadata);
    }

    fn make<S>(lines: &[S]) -> Vec<String>
    where
        S: ToString,
    {
        lines.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_progress() {
        let tmp = tempdir::TempDir::new("test_target").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();
        let inv = state::InvocationResults {
            id: "blah".to_string(),
            command: "test".to_string(),
            status: state::Status::Fail,
            start: std::time::SystemTime::now(),
            is_live: false,
            ..Default::default()
        };
        db.upsert_shallow_invocation(&inv).unwrap();
        let initial = vec!["a", "b", "c", "d"];
        db.insert_output_lines(&inv.id, make(&initial)).unwrap();
        let prog = db.get_progress(&inv.id).unwrap();
        assert_eq!(prog, "a\nb\nc\nd");
        db.delete_last_output_lines(&inv.id, 2_u32).unwrap();
        let prog = db.get_progress(&inv.id).unwrap();
        assert_eq!(prog, "a\nb");
    }

    #[test]
    fn test_get_test_history() {
        use state::{Status, TestFilter, TestFilterItem, TestFilterOp};

        let tmp = tempdir::TempDir::new("test_target").unwrap();
        let db_path = tmp.path().join("test.db");
        super::init_db(db_path.to_str().unwrap()).unwrap();
        let mgr = crate::manager::SqliteManager::new(db_path.to_str().unwrap()).unwrap();
        let mut db = mgr.get().unwrap();

        let test_name = "//:my_test";
        let now = std::time::SystemTime::now();

        // Invocation 1 (now): a successful run, on main branch
        let inv1 = state::InvocationResults {
            id: "inv1".to_string(),
            start: now,
            is_live: false,
            ..Default::default()
        };
        let test1 = state::Test {
            name: test_name.to_string(),
            status: Status::Success,
            duration: Duration::from_secs(5),
            end: now,
            num_runs: 0,
            runs: vec![],
        };
        db.upsert_shallow_invocation(&inv1).unwrap();
        db.upsert_test(&inv1.id, &test1).unwrap();
        db.insert_options(
            &inv1.id,
            &state::BuildOptions {
                build_metadata: HashMap::from([("branch".to_string(), "main".to_string())]),
                ..Default::default()
            },
        )
        .unwrap();

        // Invocation 2 (now - 1 day): a failed run, on main branch
        let day = Duration::from_secs(24 * 60 * 60);
        let inv2_time = now.checked_sub(day).unwrap();
        let inv2 = state::InvocationResults {
            id: "inv2".to_string(),
            start: inv2_time,
            is_live: false,
            ..Default::default()
        };
        let test2 = state::Test {
            name: test_name.to_string(),
            status: Status::Fail,
            duration: Duration::from_secs(12),
            end: inv2_time,
            num_runs: 0,
            runs: vec![],
        };
        db.upsert_shallow_invocation(&inv2).unwrap();
        db.upsert_test(&inv2.id, &test2).unwrap();
        db.insert_options(
            &inv2.id,
            &state::BuildOptions {
                build_metadata: HashMap::from([("branch".to_string(), "main".to_string())]),
                ..Default::default()
            },
        )
        .unwrap();

        // Invocation 3 (now - 2 days): a successful run, on a feature branch
        let inv3_time = now.checked_sub(day * 2).unwrap();
        let inv3 = state::InvocationResults {
            id: "inv3".to_string(),
            start: inv3_time,
            is_live: false,
            ..Default::default()
        };
        let test3 = state::Test {
            name: test_name.to_string(),
            status: Status::Success,
            duration: Duration::from_secs(6),
            end: inv3_time,
            num_runs: 0,
            runs: vec![],
        };
        db.upsert_shallow_invocation(&inv3).unwrap();
        db.upsert_test(&inv3.id, &test3).unwrap();
        db.insert_options(
            &inv3.id,
            &state::BuildOptions {
                build_metadata: HashMap::from([("branch".to_string(), "feature/foo".to_string())]),
                ..Default::default()
            },
        )
        .unwrap();

        // Case 1: Get all history (limit 10)
        let history = db.get_test_history(test_name, &[], 10, Some(30)).unwrap();
        assert_eq!(history.name, test_name);
        assert_eq!(history.history.len(), 3);
        assert_eq!(history.history[0].invocation_id, "inv1");
        assert_eq!(history.history[1].invocation_id, "inv2");
        assert_eq!(history.history[2].invocation_id, "inv3");

        // Case 2: Filter by status: Success
        let filters = [TestFilter {
            op: TestFilterOp::Equals,
            invert: false,
            filter: TestFilterItem::Status(Status::Success),
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 2);
        assert!(
            history
                .history
                .iter()
                .all(|h| h.test.status == Status::Success)
        );

        // Case 3: Filter by duration > 10s
        let filters = [TestFilter {
            op: TestFilterOp::GreaterThan,
            invert: false,
            filter: TestFilterItem::Duration(Duration::from_secs(10)),
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0].invocation_id, "inv2");

        // Case 4: Filter by metadata: branch=main
        let filters = [TestFilter {
            op: TestFilterOp::Equals,
            invert: false,
            filter: TestFilterItem::Metadata {
                key: "branch".to_string(),
                value: "main".to_string(),
            },
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 2);
        assert_eq!(history.history[0].invocation_id, "inv1");
        assert_eq!(history.history[1].invocation_id, "inv2");

        // Case 5: Inverted metadata filter: branch!=main
        let filters = [TestFilter {
            op: TestFilterOp::Equals,
            invert: true,
            filter: TestFilterItem::Metadata {
                key: "branch".to_string(),
                value: "main".to_string(),
            },
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0].invocation_id, "inv3");

        // Add some log output for testing LogOutput filter
        db.insert_output_lines("inv1", vec!["INFO: Test passed successfully".to_string()])
            .unwrap();
        db.insert_output_lines("inv2", vec!["ERROR: Test failed with timeout".to_string()])
            .unwrap();
        db.insert_output_lines(
            "inv3",
            vec!["DEBUG: Running feature branch test".to_string()],
        )
        .unwrap();

        // Case 6: Filter by log output containing "ERROR"
        let filters = [TestFilter {
            op: TestFilterOp::Contains,
            invert: false,
            filter: TestFilterItem::LogOutput("ERROR".to_string()),
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0].invocation_id, "inv2");

        // Case 7: Filter by exact log line match
        let filters = [TestFilter {
            op: TestFilterOp::Equals,
            invert: false,
            filter: TestFilterItem::LogOutput("INFO: Test passed successfully".to_string()),
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0].invocation_id, "inv1");

        // Case 8: Inverted log output filter (no ERROR messages)
        let filters = [TestFilter {
            op: TestFilterOp::Contains,
            invert: true,
            filter: TestFilterItem::LogOutput("ERROR".to_string()),
        }];
        let history = db.get_test_history(test_name, &filters, 10, None).unwrap();
        assert_eq!(history.history.len(), 2);
        // Should return inv1 and inv3 (no ERROR logs)
        assert!(history.history.iter().any(|h| h.invocation_id == "inv1"));
        assert!(history.history.iter().any(|h| h.invocation_id == "inv3"));

        // Case 9: Limit to 1 result
        let history = db.get_test_history(test_name, &[], 1, None).unwrap();
        assert_eq!(history.history.len(), 1);
        assert_eq!(history.history[0].invocation_id, "inv1");
    }
}
