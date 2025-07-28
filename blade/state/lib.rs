use std::{collections::HashMap, default::Default};

use cfg_if::cfg_if;
use serde::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Status {
    Unknown,
    InProgress,
    Success,
    Fail,
    Skip,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{self:?}") }
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
    pub end: std::time::SystemTime,
    pub runs: Vec<TestRun>,
    pub num_runs: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InvocationResults {
    pub id: String,
    pub targets: HashMap<String, Target>,
    pub tests: HashMap<String, Test>,
    pub status: Status,
    pub start: std::time::SystemTime,
    pub end: Option<std::time::SystemTime>,
    pub command: String,
    pub pattern: Vec<String>,
    pub last_heartbeat: Option<std::time::SystemTime>,
    pub is_live: bool,
    pub profile_uri: Option<String>,
}

impl Default for InvocationResults {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            targets: HashMap::new(),
            tests: HashMap::new(),
            status: Status::Unknown,
            command: "".into(),
            pattern: vec![],
            start: std::time::UNIX_EPOCH,
            end: None,
            last_heartbeat: None,
            is_live: false,
            profile_uri: None,
        }
    }
}

impl InvocationResults {
    /// Determines if this invocation has an active stream based on heartbeat
    /// An invocation is considered "live" if:
    /// 1. It's in InProgress or Unknown status AND
    /// 2. It has a recent heartbeat (within last 2 minutes) OR no end time set
    #[cfg(feature = "ssr")]
    pub fn is_live(&self) -> bool { self.is_live_at(std::time::SystemTime::now()) }

    /// Determines if this invocation has an active stream based on heartbeat at
    /// a specific time This method is WASM-safe as it doesn't call
    /// SystemTime::now() directly
    pub fn is_live_at(&self, current_time: std::time::SystemTime) -> bool {
        match self.status {
            Status::InProgress | Status::Unknown => {
                if let Some(heartbeat) = self.last_heartbeat {
                    // Stream is live if heartbeat is within the last 2 minutes
                    let heartbeat_threshold = std::time::Duration::from_secs(2 * 60);
                    current_time
                        .duration_since(heartbeat)
                        .map(|duration| duration < heartbeat_threshold)
                        .unwrap_or(false)
                } else {
                    // No heartbeat yet, but if no end time and recently started, consider live
                    self.end.is_none()
                }
            },
            _ => false, // Completed builds are never live
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct BuildOptions {
    pub unstructured: Vec<String>,
    pub structured: HashMap<String, Vec<String>>,
    pub startup: Vec<String>,
    pub explicit_startup: Vec<String>,
    pub cmd_line: Vec<String>,
    pub explicit_cmd_line: Vec<String>,
    pub build_metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TestHistoryPoint {
    pub invocation_id: String,
    pub start: std::time::SystemTime,
    pub test: Test,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TestHistory {
    pub name: String,
    // Invocation ID + Test
    pub history: Vec<TestHistoryPoint>,
    // Metadata about the query results
    pub total_found: usize,
    pub limit_applied: usize,
    pub was_truncated: bool,
    pub query_date_range: Option<(std::time::SystemTime, std::time::SystemTime)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum TestFilterOp {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum TestFilterItem {
    Start(std::time::SystemTime),
    Duration(std::time::Duration),
    Metadata {
        key: String,
        value: String,
    },
    BazelFlags {
        flag: String,
        value: String,
    },
    Status(Status),
    LogOutput(String),
    DateRange {
        from: std::time::SystemTime,
        to: std::time::SystemTime,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TestFilter {
    pub op: TestFilterOp,
    pub invert: bool,
    pub filter: TestFilterItem,
}

cfg_if! {
if #[cfg(feature = "ssr")] {
use derivative::Derivative;

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
    fn upsert_test_run(&mut self, id: &str, test_id: &str, run: &TestRun) -> anyhow::Result<()>;
    fn get_invocation(&mut self, id: &str) -> anyhow::Result<InvocationResults>;
    fn get_shallow_invocation(&mut self, id: &str) -> anyhow::Result<InvocationResults>;
    fn delete_invocation(&mut self, id: &str) -> anyhow::Result<()>;
    fn delete_invocations_since(&mut self, ts: &std::time::SystemTime) -> anyhow::Result<usize>;
    fn update_invocation_heartbeat(&mut self, invocation_id: &str) -> anyhow::Result<()>;
    fn insert_options(&mut self, id: &str, options: &BuildOptions) -> anyhow::Result<()>;
    fn get_options(&mut self, id: &str) -> anyhow::Result<BuildOptions>;
    fn delete_last_output_lines(&mut self, id: &str, num_lines: u32) -> anyhow::Result<()>;
    fn insert_output_lines(&mut self, id: &str, lines: Vec<String>) -> anyhow::Result<()>;
    fn get_test_history(&mut self, test_name: &str, filters: &[TestFilter], max_results: usize, default_days: Option<u32>) -> anyhow::Result<TestHistory>;
    fn search_test_names(&mut self, pattern: &str, limit: usize) -> anyhow::Result<Vec<String>>;
}

pub trait DBManager: std::marker::Send + std::marker::Sync {
    fn get(&self) -> anyhow::Result<Box<dyn DB>>;
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Global {
    #[derivative(Debug="ignore")]
    pub db_manager: Box<dyn DBManager>,
    #[derivative(Debug="ignore")]
    pub bytestream_client: bytestream::Client,
    pub allow_local: bool,
    pub retention: Option<std::time::Duration>,
    pub session_lock_time: std::time::Duration,
}

}
}
