use leptos::prelude::*;
use state::{Status, TestHistory};

use crate::components::charts::barchart::BarChart;
use leptos_dom::helpers::window;
// use chrono::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn PassFailChart(history: TestHistory) -> impl IntoView {
    let on_bar_click = |point: state::TestHistoryPoint| {
        let link = format!("/invocation/{}", point.invocation_id);
        window().location().set_href(&link).unwrap();
    };

    view! {
        <BarChart
            data=history.history
            y_accessor=|point| match point.test.status {
                Status::Success => 1.0,
                Status::Fail => 1.0,
                _ => 0.0,
            }
            x_label_accessor=|_point| {
                // let datetime: chrono::DateTime<chrono::Utc> = point.start.into();
                // datetime.format("%m/%d %H:%M").to_string()
                "Date".to_string() // Placeholder
            }
            bar_color_accessor=|point| match point.test.status {
                Status::Success => "#48bb78".to_string(), // green-500
                Status::Fail => "#f56565".to_string(),    // red-500
                _ => "#a0aec0".to_string(),               // gray-500
            }
            tooltip_content_accessor=|point| {
                format!(
                    "Invocation: {}\nStatus: {}\nDate: {}",
                    point.invocation_id.chars().take(8).collect::<String>(),
                    point.test.status,
                    // {
                    //     let datetime: chrono::DateTime<chrono::Utc> = point.start.into();
                    //     datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    // }
                    "Date Placeholder"
                )
            }
            on_bar_click=on_bar_click
            x_axis_label="Time"
            y_axis_label="Status"
        />
    }
}
