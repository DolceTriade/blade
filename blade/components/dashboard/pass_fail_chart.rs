use leptos::prelude::*;
use state::{Status, TestHistory};

use crate::components::charts::piechart::PieChart;
// use chrono::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn PassFailChart(history: TestHistory) -> impl IntoView {
    let (pass, fail): (Vec<_>, Vec<_>) = history
        .history
        .into_iter()
        .partition(|p| matches!(p.test.status, Status::Success));
    let passed = pass.len();
    let failed = fail.len();

    view! {
        <PieChart
            data=vec![(true, passed), (false, failed)]
            value_accessor=|v| v.1 as f64
            _label_accessor=|v| (if v.0 { "Pass" } else { "Fail" }).to_string()
            color_accessor=|v| (if v.0 { "green" } else { "red" }).to_string()
            tooltip_content_accessor=|v| (if v.0 { "Pass" } else { "Fail" }).to_string()
        />
    }
}
