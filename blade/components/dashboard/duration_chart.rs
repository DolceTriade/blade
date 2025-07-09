use leptos::prelude::*;
use leptos_dom::helpers::window;
use state::TestHistory;

use crate::components::charts::linechart::LineChart;
// use chrono::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn DurationChart(history: TestHistory) -> impl IntoView {
    let on_point_click = |point: state::TestHistoryPoint| {
        let link = format!("/invocation/{}", point.invocation_id);
        window().location().set_href(&link).unwrap();
    };

    view! {
        <LineChart
            data=history.history
            x_accessor=|point| {
                point.start.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
            }
            y_accessor=|point| point.test.duration.as_millis() as f64
            x_label_accessor=|_point| { "Date".to_string() }
            line_color="#4299e1"
            point_color="#4299e1"
            tooltip_content_accessor=|point| {
                format!(
                    "Invocation: {}\nDuration: {}ms\nDate: {}",
                    point.invocation_id.chars().take(8).collect::<String>(),
                    point.test.duration.as_millis(),
                    "Date Placeholder",
                )
            }
            on_point_click=on_point_click
            x_axis_label="Time"
            y_axis_label="Duration (ms)"
        />
    }
}
