use leptos::prelude::*;
use std::f64::consts::PI;

#[allow(non_snake_case)]
#[component]
pub fn PieChart<T, V, L, C, TC>(
    data: Vec<T>,
    value_accessor: V,
    _label_accessor: L, // Not used directly in this version, but good for API consistency
    color_accessor: C,
    tooltip_content_accessor: TC,
    #[prop(default = 200)] size: u32,
    #[prop(default = 0.0)] inner_radius_ratio: f64, // 0.0 for pie, > 0.0 for doughnut
) -> impl IntoView
where
    T: Clone + 'static + Send,
    V: Fn(&T) -> f64 + Copy + 'static + Send,
    L: Fn(&T) -> String + Copy + 'static + Send,
    C: Fn(&T) -> String + Copy + 'static + Send,
    TC: Fn(&T) -> String + Copy + 'static + Send,
{
    let (hovered_index, set_hovered_index) = signal(None::<usize>);

    let total_value = data.iter().map(value_accessor).sum::<f64>().max(f64::EPSILON); // Avoid division by zero
    let center = size as f64 / 2.0;
    let radius = center;

    let mut current_angle = -PI / 2.0; // Start from top

    let slices = data.iter().map(|item| {
        let value = value_accessor(item);
        let percentage = value / total_value;
        let mut angle_delta = percentage * 2.0 * PI;
        if percentage >= 1.0 {
            angle_delta = 2.0 * PI - 0.0001; // Use a slightly smaller angle for a full circle to avoid SVG path issues
        }
        let end_angle = current_angle + angle_delta;

        let large_arc_flag = if angle_delta > PI { 1 } else { 0 };

        let x1_outer = center + radius * current_angle.cos();
        let y1_outer = center + radius * current_angle.sin();
        let x2_outer = center + radius * end_angle.cos();
        let y2_outer = center + radius * end_angle.sin();

        let path = if inner_radius_ratio > 0.0 {
            let inner_radius = radius * inner_radius_ratio;
            let x1_inner = center + inner_radius * current_angle.cos();
            let y1_inner = center + inner_radius * current_angle.sin();
            let x2_inner = center + inner_radius * end_angle.cos();
            let y2_inner = center + inner_radius * end_angle.sin();
            format!(
                "M {x1_inner} {y1_inner} L {x1_outer} {y1_outer} A {radius} {radius} 0 {large_arc_flag} 1 {x2_outer} {y2_outer} L {x2_inner} {y2_inner} A {inner_radius} {inner_radius} 0 {large_arc_flag} 0 {x1_inner} {y1_inner} Z",
            )
        } else {
            format!(
                "M {center} {center} L {x1_outer} {y1_outer} A {radius} {radius} 0 {large_arc_flag} 1 {x2_outer} {y2_outer} Z",
            )
        };

        let mid_angle = current_angle + angle_delta / 2.0;
        let color = color_accessor(item);
        current_angle = end_angle;

        (path, mid_angle, color)
    }).collect::<Vec<_>>();

    let slice_views = slices.into_iter().enumerate().map(|(i, (path, _mid_angle, color))| {
        let transform = move || {
            if hovered_index.get() == Some(i) {
                "scale(1.05)".to_string()
            } else {
                "scale(1.0)".to_string()
            }
        };

        view! {
            <path
                d=path
                fill=color
                stroke="white"
                stroke-width="1"
                on:mouseenter=move |_| set_hovered_index.set(Some(i))
                on:mouseleave=move |_| set_hovered_index.set(None)
                style=format!(
                    "transition: transform 0.2s ease-out; cursor: pointer; transform-origin: {}px {}px;",
                    center, center
                )
                transform=transform
            />
        }
    }).collect_view();

    let tooltip = move || {
        hovered_index.get().map(|i| {
            let item = &data[i];
            let value = value_accessor(item);
            let percentage = (value / total_value) * 100.0;

            view! {
                <g class="pointer-events-none">
                    <text
                        x=center
                        y=center
                        style:text-anchor="middle"
                        style:dominant-baseline="middle"
                        fill="currentColor"
                        style:font-size=format!("{}px", size / 12)
                    >
                        {tooltip_content_accessor(item)}
                    </text>
                    <text
                        x=center
                        y=center + (size as f64 / 10.0)
                        style:text-anchor="middle"
                        style:dominant-baseline="middle"
                        fill="#a0aec0"
                        style:font-size=format!("{}px", size / 16)
                    >
                        {format!("{percentage:.1}%")}
                    </text>
                </g>
            }
        })
    };

    view! {
        <svg width="100%" height="100%" viewBox=format!("0 0 {size} {size}")>
            <g>{slice_views}</g>
            {tooltip}
        </svg>
    }
}
