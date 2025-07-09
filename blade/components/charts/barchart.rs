use leptos::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn BarChart<T, Y, XL, BC, TC>(
    data: Vec<T>,
    y_accessor: Y,
    x_label_accessor: XL,
    bar_color_accessor: BC,
    tooltip_content_accessor: TC,
    #[prop(optional)] on_bar_click: Option<fn(T)>,
    #[prop(default = "")] x_axis_label: &'static str,
    #[prop(default = "")] y_axis_label: &'static str,
    #[prop(default = 500)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(default = (50, 50, 50, 50))] margin: (u32, u32, u32, u32), // top, right, bottom, left
) -> impl IntoView
where
    T: Clone + 'static + Send,
    Y: Fn(&T) -> f64 + Copy + 'static + Send,
    XL: Fn(&T) -> String + Copy + 'static + Send,
    BC: Fn(&T) -> String + Copy + 'static + Send,
    TC: Fn(&T) -> String + Copy + 'static + Send,
{
    let (hovered_index, set_hovered_index) = signal(None::<usize>);

    let chart_width = width - margin.3 - margin.1;
    let chart_height = height - margin.0 - margin.2;

    let max_y = data
        .iter()
        .map(y_accessor)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1.0); // Avoid division by zero

    let num_bars = data.len();
    let bar_width = if num_bars > 0 {
        chart_width as f64 / num_bars as f64
    } else {
        0.0
    };

    let bars = data
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x = margin.3 as f64 + (i as f64 * bar_width);
            let bar_height = (y_accessor(point) / max_y) * chart_height as f64;
            let y = margin.0 as f64 + chart_height as f64 - bar_height;
            let fill = bar_color_accessor(point);

            let cloned_point = point.clone();
            let on_click_handler = move |_| {
                if let Some(callback) = on_bar_click {
                    callback(cloned_point.clone());
                }
            };

            view! {
                <rect
                    x=x.to_string()
                    y=y.to_string()
                    width=bar_width.to_string()
                    height=bar_height.to_string()
                    fill=fill
                    opacity="0.7"
                    on:mouseenter=move |_| set_hovered_index.set(Some(i))
                    on:mouseleave=move |_| set_hovered_index.set(None)
                    on:click=on_click_handler
                    style="transition: opacity 0.2s; cursor: pointer;"
                    class="hover:opacity-100"
                />
            }
        })
        .collect_view();

    let x_axis_ticks = data
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x = margin.3 as f64 + (i as f64 * bar_width) + (bar_width / 2.0);
            let y = height as f64 - margin.2 as f64 + 15.0;
            view! {
                <text
                    x=x.to_string()
                    y=y.to_string()
                    style:text-anchor="middle"
                    fill="#a0aec0"
                    style:font-size="10"
                >
                    {x_label_accessor(point)}
                </text>
            }
        })
        .collect_view();

    let y_axis_ticks = (0..=5)
        .map(|i| {
            let value = (max_y / 5.0) * i as f64;
            let y = margin.0 as f64 + chart_height as f64 - (i as f64 / 5.0) * chart_height as f64;
            view! {
                <text
                    x=(margin.3 - 10).to_string()
                    y=y.to_string()
                    style:text-anchor="end"
                    fill="#a0aec0"
                    style:font-size="10"
                >
                    {format!("{value:.1}")}
                </text>
            }
        })
        .collect_view();

    let tooltip = move || {
        hovered_index.get().map(|i| {
            let point = &data[i];
            let x = margin.3 as f64 + (i as f64 * bar_width) + (bar_width / 2.0);
            let y = margin.0 as f64 + chart_height as f64 - (y_accessor(point) / max_y) * chart_height as f64 - 10.0; // Above the bar

            view! {
                <g class="pointer-events-none" transform=format!("translate({}, {})", x, y)>
                    <rect
                        x="-75"
                        y="-30"
                        width="150"
                        height="60"
                        rx="5"
                        fill="#2d3748"
                        stroke="#4a5568"
                        stroke-width="1"
                    />
                    <text
                        x="0"
                        y="-10"
                        style:text-anchor="middle"
                        fill="white"
                        style:font-size="12"
                    >
                        {tooltip_content_accessor(point)}
                    </text>
                </g>
            }
        })
    };

    view! {
        <svg width="100%" height="100%" viewBox=format!("0 0 {} {}", width, height)>
            // X-axis line
            <line
                x1=margin.3.to_string()
                y1=(margin.0 + chart_height).to_string()
                x2=(margin.3 + chart_width).to_string()
                y2=(margin.0 + chart_height).to_string()
                stroke="#a0aec0"
                stroke-width="1"
            />
            // Y-axis line
            <line
                x1=margin.3.to_string()
                y1=margin.0.to_string()
                x2=margin.3.to_string()
                y2=(margin.0 + chart_height).to_string()
                stroke="#a0aec0"
                stroke-width="1"
            />

            {bars}
            {x_axis_ticks}
            {y_axis_ticks}

            // X-axis label
            <text
                x=(margin.3 as f64 + chart_width as f64 / 2.0).to_string()
                y=(height as f64 - 10.0).to_string()
                style:text-anchor="middle"
                fill="#a0aec0"
                style:font-size="14"
            >
                {x_axis_label}
            </text>

            // Y-axis label
            <text
                x="15"
                y=(margin.0 as f64 + chart_height as f64 / 2.0).to_string()
                transform=format!(
                    "rotate(-90, 15, {})",
                    margin.0 as f64 + chart_height as f64 / 2.0,
                )
                style:text-anchor="middle"
                fill="#a0aec0"
                style:font-size="14"
            >
                {y_axis_label}
            </text>

            {tooltip}
        </svg>
    }
}
