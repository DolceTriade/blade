use components::charts::ganttchart::BazelTraceChart;
use leptos::prelude::*;
use trace_event_parser::{BazelTrace, TraceEventFile};

#[component]
pub fn GanttRoute() -> impl IntoView {
    let json = include_str!("command.profile");
    let trace_event_file =
        TraceEventFile::from_json(json).expect("Failed to parse command.profile");
    let bazel_trace = BazelTrace::from_trace_events(trace_event_file.trace_events);

    view! {
        <div class="gantt-container">
            <h1>Gantt Chart</h1>
            <BazelTraceChart bazel_trace=bazel_trace />
        </div>
    }
}
