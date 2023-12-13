use cfg_if::cfg_if;
use serde::*;
use std::collections::HashMap;
use std::default::Default;
use std::option::Option;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
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
pub struct TestRun {
    pub duration: std::time::Duration,
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Run {
    pub run: i32,
    pub shard: i32,
    pub attempt: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Test {
    pub name: String,
    pub success: bool,
    pub duration: std::time::Duration,
    pub runs: HashMap<Run, TestRun>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InvocationResults {
    pub targets: HashMap<String, Target>,
    pub tests: HashMap<String, Test>,
    pub status: Status,
    pub output: String,
}

impl Default for InvocationResults {
    fn default() -> Self {
        Self {
            targets: HashMap::new(),
            tests: HashMap::new(),
            status: Status::Unknown,
            output: "".into(),
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
