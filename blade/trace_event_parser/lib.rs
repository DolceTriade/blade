use std::collections::HashMap;

use serde::Deserialize;

/// Represents a single trace event in the Trace Event Format.
///
/// Fields:
/// - `name`: The name of the event, as displayed in Trace Viewer.
/// - `cat`: Optional. The event categories, a comma-separated list of
///   categories for the event.
/// - `ph`: The event type (phase). A single character indicating the type of
///   event (e.g., 'B' for begin, 'E' for end, 'X' for complete, etc.).
/// - `ts`: The tracing clock timestamp of the event, provided at microsecond
///   granularity.
/// - `pid`: Optional. The process ID for the process that output this event.
/// - `tid`: Optional. The thread ID for the thread that output this event.
/// - `args`: Optional. Additional arguments provided for the event. These can
///   include any custom data relevant to the event.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    #[serde(rename = "X")]
    Complete,
    #[serde(rename = "C")]
    Counter,
    #[serde(rename = "i")]
    Instant,
    #[serde(rename = "M")]
    Metadata,
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct TraceEvent {
    pub name: String,
    pub cat: Option<String>,
    pub ph: Phase,
    #[serde(default)]
    pub ts: i64,
    pub dur: Option<i64>,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    pub args: Option<serde_json::Value>,
}

impl TraceEvent {
    pub fn is_supported_phase(&self) -> bool { !matches!(self.ph, Phase::Unsupported) }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct TraceEventFile {
    #[serde(rename = "traceEvents")]
    pub trace_events: Vec<TraceEvent>,
}

impl TraceEventFile {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> { serde_json::from_str(json) }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BazelTrace {
    pub traces: Vec<Trace>,
    pub counters: Vec<Counter>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Trace {
    pub name: String,
    pub sort_index: Option<i32>,
    pub pid: u32,
    pub tid: u32,
    pub events: Vec<Event>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    pub category: String,
    pub name: String,
    pub start: i64,
    pub duration: Option<i64>,
    pub args: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Counter {
    pub name: String,
    pub tid: u32,
    pub color: Option<String>,
    pub time_series: Vec<TimeSeriesDataPoint>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TimeSeriesDataPoint {
    pub timestamp: i64, // Changed to i64
    pub value: f64,     // Changed to f64
}

impl BazelTrace {
    pub fn from_trace_events(events: Vec<TraceEvent>) -> Self {
        let mut traces_map: HashMap<(u32, u32), Trace> = HashMap::new();
        let mut counters_map: HashMap<(String, u32, u32), Counter> = HashMap::new();

        for event in events {
            match event.ph {
                Phase::Complete | Phase::Instant => {
                    let trace = traces_map
                        .entry((event.pid.unwrap_or_default(), event.tid.unwrap_or_default()))
                        .or_insert_with(|| Trace {
                            name: String::new(),
                            sort_index: None,
                            pid: event.pid.unwrap_or_default(),
                            tid: event.tid.unwrap_or_default(),
                            events: Vec::new(),
                        });

                    trace.events.push(Event {
                        category: event.cat.unwrap_or_default(),
                        name: event.name,
                        start: event.ts,
                        duration: event.dur,
                        args: event.args,
                    });
                }
                Phase::Metadata => {
                    let pid = event.pid.unwrap_or_default();
                    let tid = event.tid.unwrap_or_default();
                    let trace = traces_map.entry((pid, tid)).or_insert_with(|| Trace {
                        name: String::new(),
                        sort_index: None,
                        pid,
                        tid,
                        events: Vec::new(),
                    });

                    if event.name == "thread_name" {
                        if let Some(name) = event
                            .args
                            .as_ref()
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str())
                        {
                            trace.name = name.to_string();
                        }
                    } else if event.name == "thread_sort_index" {
                        if let Some(sort_index) = event
                            .args
                            .as_ref()
                            .and_then(|a| a.get("sort_index"))
                            .and_then(|i| i.as_i64())
                        {
                            trace.sort_index = Some(sort_index as i32);
                        }
                    }
                }
                Phase::Unsupported => {}
                Phase::Counter => {
                    let key = (
                        event.name.clone(),
                        event.pid.unwrap_or_default(),
                        event.tid.unwrap_or_default(),
                    );
                    let counter = counters_map.entry(key).or_insert_with(|| Counter {
                        name: event.name.clone(),
                        tid: event.tid.unwrap_or_default(),
                        color: None, // Placeholder for color extraction
                        time_series: Vec::new(),
                    });

                    counter.time_series.push(TimeSeriesDataPoint {
                        timestamp: event.ts,
                        value: event
                            .args
                            .as_ref()
                            .and_then(|args| args.get("value"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or_default(), // Extract f64 value from "value" key
                    });
                },
            }
        }

        let mut traces: Vec<Trace> = traces_map.into_values().collect();
        traces.sort_by_key(|trace| trace.sort_index);
        for trace in &mut traces {
            trace.events.sort_by(|a, b| a.start.cmp(&b.start));
        }

        let counters: Vec<Counter> = counters_map.into_values().collect();

        BazelTrace { traces, counters }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trace_event_file() {
        let json = r#"{
            "traceEvents": [
                {"name": "A", "ph": "B", "ts": 1, "pid": 1, "tid": 1},
                {"name": "A", "ph": "E", "ts": 2, "pid": 1, "tid": 1}
            ]
        }"#;

        let parsed = TraceEventFile::from_json(json).unwrap();

        assert_eq!(parsed.trace_events.len(), 2);
        assert_eq!(parsed.trace_events[0].name, "A");
        assert_eq!(parsed.trace_events[0].ph, Phase::Unsupported);
        assert_eq!(parsed.trace_events[1].ph, Phase::Unsupported);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "{ invalid json }";
        let parsed = TraceEventFile::from_json(json);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_trace_event_file_with_golden_file() {
        let json = include_str!("./testdata/command.profile");
        assert!(!json.is_empty(), "Test data should not be empty");
        let parsed = TraceEventFile::from_json(json).unwrap();

        // Validate that the parsed data contains the expected number of trace events
        assert!(!parsed.trace_events.is_empty());

        // Example validation: Check the first trace event's name
        assert_eq!(parsed.trace_events[0].name, "thread_name");
        parsed.trace_events.iter().for_each(|event| {
            assert!(
                event.is_supported_phase(),
                "Event phase should be supported"
            );
        });

        let bazel_trace = BazelTrace::from_trace_events(parsed.trace_events);

        // Ensure no counters have only one item
        for counter in &bazel_trace.counters {
            assert!(
                counter.time_series.len() > 1,
                "Counter {} should have more than one data point",
                counter.name
            );
        }
        println!("{bazel_trace:#?}");
    }

    #[test]
    fn test_bazel_trace_from_trace_events() {
        let json = include_str!("./testdata/command.profile");
        let trace_event_file =
            TraceEventFile::from_json(json).expect("Failed to parse command.profile");

        let bazel_trace = BazelTrace::from_trace_events(trace_event_file.trace_events);

        // Validate traces
        assert!(!bazel_trace.traces.is_empty(), "Traces should not be empty");
        for trace in &bazel_trace.traces {
            assert!(!trace.events.is_empty(), "Each trace should have events");
            assert!(
                trace.events.windows(2).all(|w| w[0].start <= w[1].start),
                "Events should be sorted by start time"
            );
        }

        // Validate counters
        assert!(
            !bazel_trace.counters.is_empty(),
            "Counters should not be empty"
        );
        for counter in &bazel_trace.counters {
            assert!(
                !counter.time_series.is_empty(),
                "Each counter should have time series data"
            );
        }
    }

    #[test]
    fn test_bazel_trace_counters_merge() {
        let json = r#"{
            "traceEvents": [
                {"name": "counter1", "ph": "C", "ts": 1, "pid": 1, "tid": 1, "args": {"value": 10}},
                {"name": "counter1", "ph": "C", "ts": 2, "pid": 1, "tid": 1, "args": {"value": 20}},
                {"name": "counter2", "ph": "C", "ts": 3, "pid": 2, "tid": 2, "args": {"value": 30}}
            ]
        }"#;

        let trace_event_file = TraceEventFile::from_json(json).unwrap();
        let bazel_trace = BazelTrace::from_trace_events(trace_event_file.trace_events);

        // Validate counters
        assert_eq!(
            bazel_trace.counters.len(),
            2,
            "There should be two unique counters"
        );

        let counter1 = bazel_trace
            .counters
            .iter()
            .find(|c| c.name == "counter1" && c.tid == 1)
            .expect("Counter1 should exist");
        assert_eq!(
            counter1.time_series.len(),
            2,
            "Counter1 should have two data points"
        );
        assert_eq!(counter1.time_series[0].timestamp, 1);
        assert_eq!(counter1.time_series[0].value, 10.0);
        assert_eq!(counter1.time_series[1].timestamp, 2);
        assert_eq!(counter1.time_series[1].value, 20.0);

        let counter2 = bazel_trace
            .counters
            .iter()
            .find(|c| c.name == "counter2" && c.tid == 2)
            .expect("Counter2 should exist");
        assert_eq!(
            counter2.time_series.len(),
            1,
            "Counter2 should have one data point"
        );
        assert_eq!(counter2.time_series[0].timestamp, 3);
        assert_eq!(counter2.time_series[0].value, 30.0);
    }

    #[test]
    fn test_bazel_trace_metadata_processing() {
        let json = r#"{
            "traceEvents": [
                {"name": "thread_name", "ph": "M", "pid": 1, "tid": 1, "args": {"name": "MainThread"}},
                {"name": "thread_sort_index", "ph": "M", "pid": 1, "tid": 1, "args": {"sort_index": -1}},
                {"name": "some_event", "ph": "X", "ts": 10, "dur": 5, "pid": 1, "tid": 1},
                {"name": "another_event", "ph": "X", "ts": 12, "dur": 3, "pid": 1, "tid": 2, "args": {}},
                {"name": "thread_name", "ph": "M", "pid": 1, "tid": 2, "args": {"name": "WorkerThread"}}
            ]
        }"#;

        let trace_event_file = TraceEventFile::from_json(json).unwrap();
        let bazel_trace = BazelTrace::from_trace_events(trace_event_file.trace_events);

        assert_eq!(bazel_trace.traces.len(), 2, "There should be two traces");

        let main_thread_trace = bazel_trace
            .traces
            .iter()
            .find(|t| t.pid == 1 && t.tid == 1)
            .expect("MainThread trace should exist");
        assert_eq!(main_thread_trace.name, "MainThread");
        assert_eq!(main_thread_trace.sort_index, Some(-1));
        assert_eq!(main_thread_trace.events.len(), 1);

        let worker_thread_trace = bazel_trace
            .traces
            .iter()
            .find(|t| t.pid == 1 && t.tid == 2)
            .expect("WorkerThread trace should exist");
        assert_eq!(worker_thread_trace.name, "WorkerThread");
        assert_eq!(worker_thread_trace.sort_index, None);
        assert_eq!(worker_thread_trace.events.len(), 1);
    }
}
