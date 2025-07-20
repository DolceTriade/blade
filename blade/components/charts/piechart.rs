use std::f64::consts::PI;

use leptos::{either::Either, prelude::*};
use leptos::ev::MouseEvent;
use super::tooltip::{Tooltip, TooltipPosition, get_mouse_position_from_event};

#[allow(non_snake_case)]
#[component]
pub fn PieChart<T, V, L, C, TC>(
    data: Vec<T>,
    value_accessor: V,
    label_accessor: L,
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
    let (_hovered_index, set_hovered_index) = signal(None::<usize>);
    let (tooltip_position, set_tooltip_position) = signal(None::<TooltipPosition>);
    let (tooltip_content, set_tooltip_content) = signal(String::new());

    let total_value = data
        .iter()
        .map(value_accessor)
        .sum::<f64>()
        .max(f64::EPSILON); // Avoid division by zero

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
            format!("M {x1_inner} {y1_inner} L {x1_outer} {y1_outer} A {radius} {radius} 0 {large_arc_flag} 1 {x2_outer} {y2_outer} L {x2_inner} {y2_inner} A {inner_radius} {inner_radius} 0 {large_arc_flag} 0 {x1_inner} {y1_inner} Z")
        } else {
            format!("M {center} {center} L {x1_outer} {y1_outer} A {radius} {radius} 0 {large_arc_flag} 1 {x2_outer} {y2_outer} Z")
        };

        let mid_angle = current_angle + angle_delta / 2.0;
        let color = color_accessor(item);
        current_angle = end_angle;

        (path, mid_angle, color)
    }).collect::<Vec<_>>();

    let mid_angles: Vec<f64> = slices.iter().map(|v| v.1).collect();
    let data_clone = data.clone(); // Clone data for slice views

    let slice_views = slices.iter().enumerate().map(|(i, (path, _mid_angle, color))| {
        let transform = move || {
            if _hovered_index.get() == Some(i) {
                "scale(1.05)".to_string()
            } else {
                "scale(1.0)".to_string()
            }
        };

        let cloned_data_item = data_clone[i].clone(); // Clone for tooltip access
        let on_mouse_enter = move |event: MouseEvent| {
            set_hovered_index.set(Some(i));
            set_tooltip_position.set(Some(get_mouse_position_from_event(&event)));
            set_tooltip_content.set(tooltip_content_accessor(&cloned_data_item));
        };
        let on_mouse_leave = move |_| {
            set_hovered_index.set(None);
            set_tooltip_position.set(None);
        };

        view! {
            <path
                d=path.clone()
                fill=color.clone()
                stroke="white"
                stroke-width="1"
                on:mouseenter=on_mouse_enter
                on:mouseleave=on_mouse_leave
                style=format!(
                    "transition: transform 0.2s ease-out; cursor: pointer; transform-origin: {}px {}px;",
                    center,
                    center,
                )
                transform=transform
            />
        }
    }).collect_view();

    let label_views = mid_angles
        .iter()
        .enumerate()
        .map(|(i, mid_angle)| {
            let item = &data[i];
            let label_radius = if inner_radius_ratio > 0.0 {
                radius * (inner_radius_ratio + (1.0 - inner_radius_ratio) / 2.0)
            } else {
                radius * 0.65
            };

            let x = center + label_radius * mid_angle.cos();
            let y = center + label_radius * mid_angle.sin();

            // Hide label for very small slices to avoid clutter
            let value = value_accessor(item);
            let percentage = value / total_value;
            if percentage < 0.03 {
                return Either::Left(view! { <g></g> });
            }

            Either::Right(view! {
                <text
                    x=x
                    y=y
                    style:text-anchor="middle"
                    style:dominant-baseline="middle"
                    fill="white"
                    style:font-size="0.5rem"
                    class="pointer-events-none"
                >
                    {label_accessor(item)}
                </text>
            })
        })
        .collect_view();

    const HOVER_SCALE: f64 = 1.05;
    let scaled_size = size as f64 * HOVER_SCALE;
    let offset = (scaled_size - size as f64) / 2.0;

    view! {
        <div style="position: relative;">
            <svg
                width="100%"
                height="100%"
                viewBox=format!("{} {} {} {}", -offset, -offset, scaled_size, scaled_size)
            >
                <g>{slice_views}</g>
                <g>{label_views}</g>
            </svg>

            <Tooltip position=tooltip_position>
                {move || tooltip_content.get()}
            </Tooltip>
        </div>
    }
}
