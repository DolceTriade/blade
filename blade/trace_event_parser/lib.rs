use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct TraceEvent {
    pub name: String,
    pub cat: Option<String>,
    pub ph: String,
    #[serde(default)]
    pub ts: f64,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    pub args: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TraceEventFile {
    #[serde(rename = "traceEvents")]
    pub trace_events: Vec<TraceEvent>,
}

impl TraceEventFile {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trace_event_file() {
        let json = r#"{
            "traceEvents": [
                {"name": "A", "ph": "B", "ts": 1.0, "pid": 1, "tid": 1},
                {"name": "A", "ph": "E", "ts": 2.0, "pid": 1, "tid": 1}
            ]
        }"#;

        let parsed = TraceEventFile::from_json(json).unwrap();

        assert_eq!(parsed.trace_events.len(), 2);
        assert_eq!(parsed.trace_events[0].name, "A");
        assert_eq!(parsed.trace_events[0].ph, "B");
        assert_eq!(parsed.trace_events[1].ph, "E");
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
    }
}
