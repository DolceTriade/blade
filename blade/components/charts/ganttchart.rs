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
        <svg class="gantt-chart" xmlns="http://www.w3.org/2000/svg" style=move || format!("zoom: {}%; width: {}px; height: {}px;", zoom.get() * 100.0, width, height)>
            {visible_tasks.into_iter().map(|task| view! {
                <g class="gantt-task">
                    <rect
                        x=format!("{}", start_accessor(&task))
                        y="10"
                        width=format!("{}", duration_accessor(&task))
                        height="20"
                        fill=color_accessor(&task)
                        on:mouseover={
                            let task_clone = task.clone();
                            move |_| show_tooltip(&tooltip_content_accessor(&task_clone))
                        }
                    />
                    <text
                        x=format!("{}", start_accessor(&task) + duration_accessor(&task) / 2.0)
                        y="25"
                        text-anchor="middle"
                        fill="black"
                    >
                        {tooltip_content_accessor(&task)}
                    </text>
                </g>
            }).collect::<Vec<_>>()}
        </svg>
    }
}

fn show_tooltip(_details: &str) {
    // Logic to display a tooltip with the given details
}
