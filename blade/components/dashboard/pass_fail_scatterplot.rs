use leptos::prelude::*;
use leptos_dom::helpers::window;
use state::{Status, TestHistory};

use crate::{charts::scatterplot::ScatterPlot, summaryheader::format_time};

#[allow(non_snake_case)]
#[component]
pub fn PassFailScatterPlot(history: TestHistory) -> impl IntoView {
    let on_point_click = |point: state::TestHistoryPoint| {
        let link = format!("/invocation/{}", point.invocation_id);
        window().location().set_href(&link).unwrap();
    };

    view! {
        <ScatterPlot
            data=history.history
            x_accessor=|point: &state::TestHistoryPoint| {
                point.start.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
            }
            y_accessor=|point| match point.test.status {
                Status::Success => 1.0,
                Status::Fail => 0.0,
                _ => 0.5,
            }
            _x_label_accessor=|point: &state::TestHistoryPoint| { format_time(&point.start) }
            point_color_accessor=|point| match point.test.status {
                Status::Success => "#48bb78".to_string(),
                Status::Fail => "#f56565".to_string(),
                _ => "#a0aec0".to_string(),
            }
            tooltip_content_accessor=|point| {
                format!(
                    "Invocation: {}\nStatus: {}\nDate: {}",
                    point.invocation_id.chars().take(8).collect::<String>(),
                    point.test.status,
                    format_time(&point.start),
                )
            }
            on_point_click=on_point_click
            x_axis_label="Time"
            y_axis_label="Status (0=Fail, 1=Success)"
        />
    }
}
