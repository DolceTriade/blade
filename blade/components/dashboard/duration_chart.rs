use leptos::prelude::*;
use state::TestHistory;

#[allow(non_snake_case)]
#[component]
pub fn DurationChart(history: TestHistory) -> impl IntoView {
    let width = 500;
    let height = 200;

    let max_duration = history
        .history
        .iter()
        .map(|p| p.test.duration.as_millis() as u32)
        .max()
        .unwrap_or(0)
        .max(1); // Avoid division by zero

    let points = history
        .history
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let x = (i as f32 / (history.history.len() - 1).max(1) as f32) * width as f32;
            let y = height as f32
                - (p.test.duration.as_millis() as f32 / max_duration as f32) * height as f32;
            (x, y)
        })
        .collect::<Vec<_>>();

    let path_data = points
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, (x, y))| {
            if i == 0 {
                acc.push_str(&format!("M {x} {y}"));
            } else {
                acc.push_str(&format!(" L {x} {y}"));
            }
            acc
        });

    let circles = points
        .iter()
        .cloned()
        .map(|(x, y)| {
            view! { <circle cx=move || format!("{x}") cy=move || format!("{y}") r="3" fill="#4299e1" /> }
        })
        .collect_view();

    view! {
        <svg width="100%" height="100%" viewBox=format!("0 0 {} {}", width, height)>
            <path d=path_data fill="none" stroke="#4299e1" stroke-width="2" />
            {circles}
        </svg>
    }
}
