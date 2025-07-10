use leptos::prelude::*;

#[component]
pub fn ScatterPlot<T, X, Y, XL, PC, TC>(
    data: Vec<T>,
    x_accessor: X,
    y_accessor: Y,
    _x_label_accessor: XL,
    point_color_accessor: PC,
    tooltip_content_accessor: TC,
    #[prop(optional)] on_point_click: Option<fn(T)>,
    x_axis_label: &'static str,
    y_axis_label: &'static str,
) -> impl IntoView
where
    T: Clone + 'static,
    X: Fn(&T) -> f64 + Copy,
    Y: Fn(&T) -> f64 + Copy,
    XL: Fn(&T) -> String + 'static,
    PC: Fn(&T) -> String + 'static,
    TC: Fn(&T) -> String + 'static,
{
    let x_min = data.iter().map(x_accessor).fold(f64::INFINITY, f64::min);
    let x_max = data
        .iter()
        .map(x_accessor)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = data.iter().map(y_accessor).fold(f64::INFINITY, f64::min);
    let y_max = data
        .iter()
        .map(y_accessor)
        .fold(f64::NEG_INFINITY, f64::max);

    let width = 800;
    let height = 400;
    let margin = 50;

    let x_scale = |val: f64| {
        (val - x_min) / (x_max - x_min) * (width as f64 - 2.0 * margin as f64) + margin as f64
    };
    let y_scale = |val: f64| {
        height as f64
            - ((val - y_min) / (y_max - y_min) * (height as f64 - 2.0 * margin as f64)
                + margin as f64)
    };

    view! {
        <div class="chart-container">
            <svg width=width height=height viewBox=format!("0 0 {} {}", width, height)>
                // X-axis
                <line
                    x1=margin
                    y1=height - margin
                    x2=width - margin
                    y2=height - margin
                    stroke="currentColor"
                />
                <text x=width / 2 y=height - margin / 2 text-anchor="middle" fill="currentColor">
                    {x_axis_label}
                </text>

                // Y-axis
                <line x1=margin y1=margin x2=margin y2=height - margin stroke="currentColor" />
                <text
                    x=margin / 2
                    y=height / 2
                    text-anchor="middle"
                    dominant-baseline="middle"
                    transform=format!("rotate(-90, {}, {})", margin / 2, height / 2)
                    fill="currentColor"
                >
                    {y_axis_label}
                </text>

                {data
                    .into_iter()
                    .map(|d| {
                        let cx = x_scale(x_accessor(&d));
                        let cy = y_scale(y_accessor(&d));
                        let color = point_color_accessor(&d);
                        let tooltip_content = tooltip_content_accessor(&d);
                        let cloned_d = d.clone();
                        view! {
                            <circle
                                cx=cx
                                cy=cy
                                r="5"
                                fill=color
                                stroke="white"
                                stroke-width="1"
                                on:click=move |_| {
                                    if let Some(f) = on_point_click.as_ref() {
                                        f(cloned_d.clone())
                                    }
                                }
                            >
                                <title>{tooltip_content}</title>
                            </circle>
                        }
                    })
                    .collect_view()}
            </svg>
        </div>
    }
}
