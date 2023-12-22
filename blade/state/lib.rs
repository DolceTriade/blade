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
use futures::lock::Mutex;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Invocation {
    pub results: InvocationResults,
}

pub struct Global {
    pub sessions: Mutex<HashMap<String, Arc<Mutex<Invocation>>>>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for Global {
         fn default() -> Self {
             Self::new()
}}

}
}
