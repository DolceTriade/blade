use std::time::UNIX_EPOCH;

use anyhow::Context;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use time::macros::format_description;

pub(crate) fn format_time(t: &std::time::SystemTime) -> anyhow::Result<String> {
    let ts: time::OffsetDateTime = (*t).into();
    ts.format(&format_description!(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]"
    ))
    .context("error formatting time")
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = super::schema::Invocations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Invocation {
    pub id: String,
    pub status: String,
    pub start: String,
    pub output: String,
    pub command: String,
    pub pattern: Option<String>,
}

impl Invocation {
    pub fn from_state(ir: &state::InvocationResults) -> anyhow::Result<Self> {
        Ok(Self {
            id: ir.id.clone(),
            status: ir.status.to_string(),
            start: format_time(&ir.start)?,
            output: ir.output.clone(),
            command: ir.command.clone(),
            pattern: Some(ir.pattern.join(",")),
        })
    }

    pub fn into_state(self) -> state::InvocationResults {
        state::InvocationResults {
            id: self.id,
            status: state::Status::parse(&self.status),
            output: self.output,
            start: super::parse_time(&self.start).unwrap_or(UNIX_EPOCH),
            command: self.command,
            pattern: self
                .pattern.as_ref()
                .map(|p| p.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = super::schema::Targets)]
#[diesel(belongs_to(Invocation, foreign_key = invocation_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Target {
    pub id: String,
    pub invocation_id: String,
    pub name: String,
    pub status: String,
    pub kind: String,
    pub start: String,
    pub end: Option<String>,
}

impl Target {
    pub fn gen_id(invocation_id: &str, name: &str) -> String {
        [invocation_id, name].join("|")
    }

    pub fn from_state(invocation_id: &str, t: &state::Target) -> anyhow::Result<Self> {
        let end = {
            if let Some(end) = &t.end {
                Some(format_time(end)?)
            } else {
                None
            }
        };
        Ok(Self {
            id: Self::gen_id(invocation_id, &t.name),
            invocation_id: invocation_id.to_string(),
            name: t.name.clone(),
            status: t.status.to_string(),
            kind: t.kind.clone(),
            start: format_time(&t.start)?,
            end,
        })
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = super::schema::Tests)]
#[diesel(belongs_to(Invocation, foreign_key = invocation_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Test {
    pub id: String,
    pub invocation_id: String,
    pub name: String,
    pub status: String,
    pub duration_s: Option<f64>,
    pub end: String,
    pub num_runs: Option<i32>,
}

impl Test {
    pub fn gen_id(invocation_id: &str, name: &str) -> String {
        [invocation_id, name].join("|")
    }
    pub fn from_state(invocation_id: &str, t: &state::Test) -> anyhow::Result<Self> {
        Ok(Self {
            id: Self::gen_id(invocation_id, &t.name),
            invocation_id: invocation_id.to_string(),
            name: t.name.clone(),
            status: t.status.to_string(),
            end: format_time(&t.end)?,
            duration_s: Some(t.duration.as_secs_f64()),
            num_runs: Some(t.num_runs as i32),
        })
    }

    pub fn into_state(self) -> state::Test {
        state::Test {
            name: self.name,
            duration: self.duration_s.map(std::time::Duration::from_secs_f64).unwrap_or_default(),
            num_runs: self.num_runs.unwrap_or(0) as usize,
            runs: vec![],
            status: state::Status::parse(&self.status),
            end: super::parse_time(&self.end).unwrap_or(UNIX_EPOCH),
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = super::schema::TestRuns)]
#[diesel(belongs_to(Invocation, foreign_key = invocation_id))]
#[diesel(belongs_to(Test, foreign_key = test_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TestRun {
    pub id: String,
    pub invocation_id: String,
    pub test_id: String,
    pub run: i32,
    pub shard: i32,
    pub attempt: i32,
    pub status: String,
    pub details: String,
    pub duration_s: f64,
}

impl TestRun {
    pub fn gen_id(
        invocation_id: &str,
        test_id: &str,
        run: &str,
        shard: &str,
        attempt: &str,
    ) -> String {
        [invocation_id, test_id, run, shard, attempt].join("|")
    }
    pub fn from_state(
        invocation_id: &str,
        test_id: &str,
        t: &state::TestRun,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: Self::gen_id(
                invocation_id,
                test_id,
                &t.run.to_string(),
                &t.shard.to_string(),
                &t.attempt.to_string(),
            ),
            invocation_id: invocation_id.to_string(),
            test_id: test_id.to_string(),
            run: t.run,
            shard: t.shard,
            attempt: t.attempt,
            status: t.status.to_string(),
            details: t.details.to_string(),
            duration_s: t.duration.as_secs_f64(),
        })
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = super::schema::TestArtifacts)]
#[diesel(belongs_to(Invocation, foreign_key = invocation_id))]
#[diesel(belongs_to(TestRun, foreign_key = test_run_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TestArtifact {
    pub id: String,
    pub invocation_id: String,
    pub test_run_id: String,
    pub name: String,
    pub uri: String,
}

impl TestArtifact {
    pub fn from_state(
        invocation_id: &str,
        test_run_id: &str,
        artifact_name: &str,
        t: &state::Artifact,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v5(
                &uuid::Uuid::nil(),
                format!("{}/{}/{}", invocation_id, test_run_id, artifact_name).as_bytes(),
            )
            .to_string(),
            invocation_id: invocation_id.to_string(),
            test_run_id: test_run_id.to_string(),
            name: artifact_name.to_string(),
            uri: t.uri.clone(),
        }
    }
}
