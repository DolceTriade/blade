use cfg_if::cfg_if;
use futures::lock::Mutex;
use serde::*;
use std::collections::HashMap;
use std::default::Default;
use std::option::Option;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Target {
    pub name: String,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Test {
    pub name: String,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvocationResults {
    pub targets: Vec<Target>,
    pub tests: Vec<Test>,
    pub success: Option<bool>,
    pub output: String,
}

impl Default for InvocationResults {
    fn default() -> Self {
        Self {
            targets: vec![],
            tests: vec![],
            success: None,
            output: "".into(),
        }
    }
}

cfg_if! {
if #[cfg(feature = "ssr")] {
    use tokio::sync::mpsc;
#[derive(Debug)]
pub struct Invocation {
    pub results: InvocationResults,
    pub rx: mpsc::Receiver<()>,
    pub tx: mpsc::Sender<()>,
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

impl Default for Invocation {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(128);
        Invocation {
            results: Default::default(),
            rx,
            tx,
        }
    }
}
}
}
