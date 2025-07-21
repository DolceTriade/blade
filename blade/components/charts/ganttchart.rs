use leptos::*;
use leptos::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn GanttChart<T>(
    tasks: Vec<T>,
    start_accessor: impl Fn(&T) -> f64 + Copy + 'static,
    duration_accessor: impl Fn(&T) -> f64 + Copy + 'static,
    color_accessor: impl Fn(&T) -> String + Copy + 'static,
    tooltip_content_accessor: impl Fn(&T) -> String + Copy + 'static,
    #[prop(into)] zoom: Signal<f64>,
    #[prop(default = 500)] width: u32,
    #[prop(default = 200)] height: u32,
) -> impl IntoView
where
    T: Clone + 'static + Send,
{
    let visible_tasks = tasks.clone().into_iter().filter(|_task| true).collect::<Vec<_>>(); // Placeholder for visibility logic

    view! {
        <div class="gantt-chart" style=move || format!("zoom: {}%; width: {}px; height: {}px;", zoom.get() * 100.0, width, height)>
            {visible_tasks.into_iter().map(|task| view! {
                <div class="gantt-task"
                     style=format!("left: {}%; width: {}%; background-color: {}%;", start_accessor(&task), duration_accessor(&task), color_accessor(&task))
                     on:mouseover=move |_| show_tooltip(&tooltip_content_accessor(&task))>
                    {tooltip_content_accessor(&task)}
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

fn show_tooltip(_details: &str) {
    // Logic to display a tooltip with the given details
}
