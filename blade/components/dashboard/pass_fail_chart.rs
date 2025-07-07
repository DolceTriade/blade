use leptos::prelude::*;
use state::{Status, TestHistory};

#[allow(non_snake_case)]
#[component]
pub fn PassFailChart(history: TestHistory) -> impl IntoView {
    let width = 500;
    let height = 200;
    let bar_width = width / (history.history.len() as u32 * 2);

    let bars = history
        .history
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x = (i as u32 * bar_width * 2) + (bar_width / 2);
            let fill = match point.test.status {
                Status::Success => "#48bb78", // green-500
                Status::Fail => "#f56565",    // red-500
                _ => "#a0aec0",               // gray-500
            };
            view! { <rect x=x y=0 width=bar_width height=height fill=fill /> }
        })
        .collect_view();

    view! {
        <svg width="100%" height="100%" viewBox=format!("0 0 {} {}", width, height)>
            {bars}
        </svg>
    }
}
