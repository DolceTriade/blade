use leptos::prelude::*;
use leptos_dom::helpers::window;
use state::TestHistory;

use crate::{charts::linechart::LineChart, summaryheader::format_time};

pub fn format_unix(t: f64) -> String {
    let d = std::time::Duration::from_secs_f64(t);
    let ss = std::time::SystemTime::UNIX_EPOCH
        .checked_add(d)
        .unwrap_or_else(std::time::SystemTime::now);
    format_time(&ss)
}

#[allow(non_snake_case)]
#[component]
pub fn DurationChart(history: TestHistory) -> impl IntoView {
    let on_point_click = |point: state::TestHistoryPoint| {
        let link = format!("/invocation/{}", point.invocation_id);
        window().open_with_url_and_target(&link, "_blank").unwrap();
    };

    // Sort data so that successful tests are rendered first and failed tests last
    // This ensures failed points (red) appear on top of successful points (green) when they overlap
    let mut sorted_history = history.history;
    sorted_history.sort_by(|a, b| {
        // Put failures last (so they render on top)
        // Success = false, Failure = true, so failures come after successes
        matches!(a.test.status, state::Status::Fail).cmp(&matches!(b.test.status, state::Status::Fail))
    });

    view! {
        <LineChart
            data=sorted_history
            x_accessor=|point| {
                point.start.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
            }
            y_accessor=|point| point.test.duration.as_secs_f64()
            line_color="#4299e1"
            point_color_accessor=|p| {
                (match p.test.status {
                    state::Status::Success => "#48bb78",
                    _ => "#f56565",
                })
                    .to_string()
            }
            tooltip_content_accessor=|point| {
                format!(
                    "Invocation: {}\nDuration: {}\nDate: {}",
                    point.invocation_id.chars().take(8).collect::<String>(),
                    humantime::format_duration(point.test.duration),
                    format_time(&point.start),
                )
            }
            x_tick_formatter=Box::new(format_unix)
            on_point_click=on_point_click
            x_axis_label="Time"
            y_axis_label="Duration (s)"
            x_axis_label_rotation=10.0
            show_line=false
        />
    }
}
