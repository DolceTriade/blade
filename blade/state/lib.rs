use cfg_if::cfg_if;
use serde::*;
use std::collections::HashMap;
use std::default::Default;
use std::option::Option;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Status {
    Unknown,
    InProgress,
    Success,
    Fail,
    Skip,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Status {
    pub fn parse(s: &str) -> Self {
        match s {
            "InProgress" => Status::InProgress,
            "Success" => Status::Success,
            "Fail" => Status::Fail,
            "Skip" => Status::Skip,
            _ => Status::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Target {
    pub name: String,
    pub status: Status,
    pub kind: String,
    pub start: std::time::SystemTime,
    pub end: Option<std::time::SystemTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Artifact {
    pub size: usize,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TestRun {
    pub run: i32,
    pub shard: i32,
    pub attempt: i32,
    pub status: Status,
    pub details: String,
    pub duration: std::time::Duration,
    pub files: HashMap<String, Artifact>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Test {
    pub name: String,
    pub status: Status,
    pub duration: std::time::Duration,
    pub runs: Vec<TestRun>,
    pub num_runs: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InvocationResults {
    pub id: String,
    pub targets: HashMap<String, Target>,
    pub tests: HashMap<String, Test>,
    pub status: Status,
    pub output: String,
    pub start: std::time::SystemTime,
    pub command: String,
    pub pattern: Vec<String>,
}

impl Default for InvocationResults {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            targets: HashMap::new(),
            tests: HashMap::new(),
            status: Status::Unknown,
            output: "".into(),
            command: "".into(),
            pattern: vec![],
            start: std::time::UNIX_EPOCH,
        }
    }
}

cfg_if! {
if #[cfg(feature = "ssr")] {

pub trait DB {
    fn upsert_shallow_invocation(&mut self, invocation: &InvocationResults) -> anyhow::Result<()>;
    #[allow(clippy::type_complexity)]
    fn update_shallow_invocation(&mut self, invocation_id: &str, upd: Box<dyn FnOnce(&mut InvocationResults) -> anyhow::Result<()>>) -> anyhow::Result<()>;
    fn get_progress(&mut self, invocation_id: &str) -> anyhow::Result<String>;
    fn upsert_target(&mut self, id: &str, target: &Target) -> anyhow::Result<()>;
    fn update_target_result(&mut self, invocation_id: &str, name: &str, status: Status, end: std::time::SystemTime) -> anyhow::Result<()>;
    fn upsert_test(&mut self, id: &str, test: &Test) -> anyhow::Result<String>;
    fn get_test(&mut self, id: &str, name: &str) -> anyhow::Result<Test>;
    fn update_test_result(&mut self, invocation_id: &str, name: &str, status: Status, duration: std::time::Duration, num_runs: usize) -> anyhow::Result<()>;
    fn insert_test_run(&mut self, id: &str, test_id: &str, run: &TestRun) -> anyhow::Result<()>;

    fn get_invocation(&mut self, id: &str) -> anyhow::Result<InvocationResults>;
}

pub trait DBManager: std::marker::Send + std::marker::Sync {
    fn get(&self) -> anyhow::Result<Box<dyn DB>>;
}

pub struct Global {
    pub db_manager: Box<dyn DBManager>,
    pub bytestream_client: bytestream::Client,
    pub allow_local: bool,
}

}
}
