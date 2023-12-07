use cfg_if::cfg_if;
use serde::*;
use std::collections::HashMap;
use std::default::Default;
use std::option::Option;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Unknown,
    InProgress,
    Success,
    Fail,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Target {
    pub name: String,
    pub status: Status,
    pub kind: String,
    pub start: std::time::SystemTime,
    pub end: Option<std::time::SystemTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Test {
    pub name: String,
    pub success: bool,
    pub duration: std::time::Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
