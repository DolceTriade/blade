use anyhow::Context;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use time::macros::format_description;

fn format_time(t: &std::time::SystemTime) -> anyhow::Result<String> {
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
}
