use leptos::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn LineChart<T, X, Y, PC, TC>(
    data: Vec<T>,
    x_accessor: X,
    y_accessor: Y,
    line_color: &'static str,
    point_color_accessor: PC,
    tooltip_content_accessor: TC,
    #[prop(optional)] on_point_click: Option<fn(T)>,
    #[prop(default = "")] x_axis_label: &'static str,
    #[prop(default = "")] y_axis_label: &'static str,
    #[prop(default = 500)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(default = (50, 50, 50, 50))] margin: (u32, u32, u32, u32), // top, right, bottom, left
    #[prop(default = 5)] x_axis_ticks_count: u32,
    #[prop(optional)] x_tick_formatter: Option<Box<dyn Fn(f64) -> String + 'static + Send>>,
    #[prop(optional)] x_axis_label_rotation: Option<f64>,
    #[prop(default = true)] show_y_axis_labels: bool,
    #[prop(default = true)] show_x_axis_labels: bool,
    #[prop(default = true)] show_line: bool,
) -> impl IntoView
where
    T: Clone + 'static + Send,
    X: Fn(&T) -> f64 + Copy + 'static + Send,
    Y: Fn(&T) -> f64 + Copy + 'static + Send,
    PC: Fn(&T) -> String + Copy + 'static + Send,
    TC: Fn(&T) -> String + Copy + 'static + Send,
{
    let (hovered_index, set_hovered_index) = signal(None::<usize>);
    let (tooltip_position, set_tooltip_position) = signal(None::<(f64, f64)>);

    // Adjust margins dynamically based on what labels are shown
    let adjusted_margin = (
        margin.0, // top
        margin.1, // right
        if show_x_axis_labels {
            margin.2 + 20
        } else {
            margin.2
        }, // bottom - extra space for tick labels + axis label
        if show_y_axis_labels {
            margin.3 + 10
        } else {
            margin.3
        }, // left - extra space for wider tick labels
    );

    let chart_width = width - adjusted_margin.3 - adjusted_margin.1;
    let chart_height = height - adjusted_margin.0 - adjusted_margin.2;

    let x_tick_formatter = x_tick_formatter.unwrap_or_else(|| Box::new(|v: f64| format!("{v:.1}")));

    let max_y = data
        .iter()
        .map(y_accessor)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1.0); // Avoid division by zero

    let min_x = data.iter().map(x_accessor).fold(f64::INFINITY, f64::min);
    let max_x = data
        .iter()
        .map(x_accessor)
        .fold(f64::NEG_INFINITY, f64::max);

    let x_scale = if (max_x - min_x) == 0.0 {
        0.0
    } else {
        chart_width as f64 / (max_x - min_x)
    };

    let points = data
        .iter()
        .map(|p| {
            let x = adjusted_margin.3 as f64 + (x_accessor(p) - min_x) * x_scale;
            let y = adjusted_margin.0 as f64 + chart_height as f64
                - (y_accessor(p) / max_y) * chart_height as f64;
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
        .enumerate()
        .map(|(i, (x, y))| {
            let cloned_point = data[i].clone();
            let on_click_handler = move |_| {
                if let Some(callback) = on_point_click {
                    callback(cloned_point.clone());
                }
            };
            let on_mouse_enter = move |ev| {
                set_hovered_index.set(Some(i));
                set_tooltip_position.set(Some(super::get_mouse_position_from_event(&ev)));
            };
            let on_mouse_leave = move |_| {
                set_hovered_index.set(None);
                set_tooltip_position.set(None);
            };
            view! {
                <circle
                    cx=x.to_string()
                    cy=y.to_string()
                    r="4"
                    fill=point_color_accessor(&data[i])
                    stroke="#1a202c"
                    stroke-width="2"
                    on:mouseenter=on_mouse_enter
                    on:mouseleave=on_mouse_leave
                    on:click=on_click_handler
                    class="hover:r-6 transition-all cursor-pointer"
                />
            }
        })
        .collect_view();

    let x_axis_ticks = if show_x_axis_labels {
        (0..=x_axis_ticks_count)
            .map(|i| {
                let value = min_x + (max_x - min_x) / x_axis_ticks_count as f64 * i as f64;
                let x = adjusted_margin.3 as f64 + (value - min_x) * x_scale;
                let y = height as f64 - adjusted_margin.2 as f64 + 15.0;
                view! {
                    <text
                        x=x.to_string()
                        y=y.to_string()
                        style:text-anchor="middle"
                        fill="#a0aec0"
                        style:font-size="10"
                        transform=x_axis_label_rotation.map(|r| format!("rotate({r}, {x}, {y})"))
                    >
                        {x_tick_formatter(value)}
                    </text>
                }
            })
            .collect_view()
    } else {
        vec![].into_iter().collect_view()
    };

    // Calculate dynamic positioning for axis labels to avoid overlap
    let x_axis_label_y = if show_x_axis_labels {
        // Position below tick labels with extra spacing
        height as f64 - adjusted_margin.2 as f64 + 35.0
    } else {
        // Position closer when no tick labels
        height as f64 - 15.0
    };

    let y_axis_label_x = if show_y_axis_labels {
        // Position further left to avoid tick labels
        10.0
    } else {
        // Position closer when no tick labels
        25.0
    };

    let x_axis_tick_marks = (0..=x_axis_ticks_count)
        .map(|i| {
            let value = min_x + (max_x - min_x) / x_axis_ticks_count as f64 * i as f64;
            let x = adjusted_margin.3 as f64 + (value - min_x) * x_scale;
            let y_start = (adjusted_margin.0 + chart_height) as f64;
            let y_end = y_start + 5.0; // 5px tick marks
            view! {
                <line
                    x1=x.to_string()
                    y1=y_start.to_string()
                    x2=x.to_string()
                    y2=y_end.to_string()
                    stroke="#a0aec0"
                    stroke-width="1"
                />
            }
        })
        .collect_view();

    let y_axis_ticks = if show_y_axis_labels {
        (0..=5)
            .map(|i| {
                let value = (max_y / 5.0) * i as f64;
                let y = adjusted_margin.0 as f64 + chart_height as f64
                    - (i as f64 / 5.0) * chart_height as f64;
                view! {
                    <text
                        x=(adjusted_margin.3 - 10).to_string()
                        y=y.to_string()
                        style:text-anchor="end"
                        fill="#a0aec0"
                        style:font-size="10"
                    >
                        {format!("{value:.1}")}
                    </text>
                }
            })
            .collect_view()
    } else {
        vec![].into_iter().collect_view()
    };

    let tooltip = move || {
        hovered_index
            .get()
            .zip(tooltip_position.get())
            .map(|(i, (x, y))| {
                let point = &data[i];
                view! {
                    <div
                        style="
                        position: absolute;
                        background-color: #2d3748;
                        border: 1px solid #4a5568;
                        border-radius: 5px;
                        padding: 5px 10px;
                        color: white;
                        font-size: 12px;
                        text-align: center;
                        display: inline-block;
                        "
                        style:top=format!("{y}px")
                        style:left=format!("{x}px")
                    >
                        {tooltip_content_accessor(point)}
                    </div>
                }
            })
    };

    view! {
        <svg width="100%" height="100%" viewBox=format!("0 0 {width} {height}")>
            // X-axis line
            <line
                x1=adjusted_margin.3.to_string()
                y1=(adjusted_margin.0 + chart_height).to_string()
                x2=(adjusted_margin.3 + chart_width).to_string()
                y2=(adjusted_margin.0 + chart_height).to_string()
                stroke="#a0aec0"
                stroke-width="1"
            />
            // Y-axis line
            <line
                x1=adjusted_margin.3.to_string()
                y1=adjusted_margin.0.to_string()
                x2=adjusted_margin.3.to_string()
                y2=(adjusted_margin.0 + chart_height).to_string()
                stroke="#a0aec0"
                stroke-width="1"
            />

            {show_line
                .then(|| {
                    view! { <path d=path_data fill="none" stroke=line_color stroke-width="2" /> }
                })}
            {circles}
            {x_axis_tick_marks}
            {x_axis_ticks}
            {y_axis_ticks}

            // X-axis label
            <text
                x=(adjusted_margin.3 as f64 + chart_width as f64 / 2.0).to_string()
                y=x_axis_label_y.to_string()
                style:text-anchor="middle"
                fill="#a0aec0"
                style:font-size="14"
            >
                {x_axis_label}
            </text>

            // Y-axis label
            <text
                x=y_axis_label_x.to_string()
                y=(adjusted_margin.0 as f64 + chart_height as f64 / 2.0).to_string()
                transform=format!(
                    "rotate(-90, {}, {})",
                    y_axis_label_x,
                    adjusted_margin.0 as f64 + chart_height as f64 / 2.0,
                )
                style:text-anchor="middle"
                fill="#a0aec0"
                style:font-size="14"
            >
                {y_axis_label}
            </text>

        </svg>
        {tooltip}
    }
}
