use leptos::prelude::*;
use trace_event_parser::{BazelTrace, Event};

const TRACE_NAME_WIDTH: f64 = 200.0;
const ROW_HEIGHT: f64 = 30.0;
const EVENT_HEIGHT: f64 = 20.0;
const V_PADDING: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 30.0; // Increased for more space

#[derive(Clone, Debug)]
struct PositionedEvent {
    id: usize,
    event: Event,
    row: usize,
}

fn calculate_layout(events: &[Event]) -> (Vec<PositionedEvent>, usize) {
    let mut positioned_events = Vec::new();
    let mut row_ends: Vec<f64> = Vec::new();

    let mut sorted_events = events.to_vec();
    sorted_events.sort_by(|a, b| {
        if a.start != b.start {
            a.start.cmp(&b.start)
        } else {
            a.name.cmp(&b.name)
        }
    });

    for (i, event) in sorted_events.into_iter().enumerate() {
        let start_time = event.start as f64;
        let duration = event.duration.unwrap_or(1) as f64;
        let end_time = start_time + duration;

        let mut placed = false;
        for (j, row_end) in row_ends.iter_mut().enumerate() {
            if start_time >= *row_end {
                positioned_events.push(PositionedEvent {
                    id: i,
                    event: event.clone(),
                    row: j,
                });
                *row_end = end_time;
                placed = true;
                break;
            }
        }

        if !placed {
            let new_row = row_ends.len();
            positioned_events.push(PositionedEvent {
                id: i,
                event: event.clone(),
                row: new_row,
            });
            row_ends.push(end_time);
        }
    }

    (positioned_events, row_ends.len().max(1))
}

fn color_for_category(category: &str) -> String {
    let mut hash: u32 = 0;
    for char in category.chars() {
        hash = (char as u32).wrapping_add(hash.wrapping_shl(5).wrapping_sub(hash));
    }
    let r = (hash & 0xFF0000) >> 16;
    let g = (hash & 0x00FF00) >> 8;
    let b = hash & 0x0000FF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[allow(non_snake_case)]
#[component]
pub fn BazelTraceChart(
    mut bazel_trace: BazelTrace,
    #[prop(default = 1000)] width: u32,
    #[prop(default = 800)] height: u32,
) -> impl IntoView {
    // Sort traces by pid and tid to ensure deterministic order
    bazel_trace
        .traces
        .sort_by(|a, b| a.pid.cmp(&b.pid).then(a.tid.cmp(&b.tid)));

    let (min_start_time, max_end_time) = bazel_trace
        .traces
        .iter()
        .flat_map(|trace| &trace.events)
        .fold((i64::MAX, 0), |(min_s, max_e), event| {
            (
                min_s.min(event.start),
                max_e.max(event.start + event.duration.unwrap_or(1)),
            )
        });

    let min_start_time = if min_start_time == i64::MAX {
        0
    } else {
        min_start_time
    };

    let duration = (max_end_time - min_start_time) as f64;

    let layouts = StoredValue::new(
        bazel_trace
            .traces
            .iter()
            .map(|trace| calculate_layout(&trace.events))
            .collect::<Vec<_>>(),
    );

    let total_height = layouts.with_value(|l| {
        l.iter()
            .map(|(_, num_rows)| *num_rows as f64 * ROW_HEIGHT)
            .sum::<f64>()
            + X_AXIS_HEIGHT
    });

    let initial_zoom = if duration > 0.0 {
        (width as f64 - TRACE_NAME_WIDTH) / duration
    } else {
        1.0
    };
    let (zoom, set_zoom) = signal(initial_zoom);

    let scale = move || zoom.get();
    let timeline_width = move || duration * scale();

    let x_axis_ticks = move || {
        let timeline_w = timeline_width();
        if timeline_w <= 0.0 {
            return Vec::new();
        }

        // 1. Determine the time unit for the whole axis based on duration
        let (unit_label, divisor) = if duration >= 1_000_000.0 {
            ("s", 1_000_000.0)
        } else if duration >= 1_000.0 {
            ("ms", 1_000.0)
        } else {
            ("µs", 1.0)
        };

        // 2. Calculate a "nice" time interval for ticks
        let target_tick_spacing_px = 150.0;
        let rough_tick_interval_us = (target_tick_spacing_px / timeline_w) * duration;

        if rough_tick_interval_us <= 0.0 {
            return Vec::new();
        }

        let exponent = 10.0_f64.powf(rough_tick_interval_us.log10().floor());
        let fraction = rough_tick_interval_us / exponent;

        let nice_fraction = if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };

        let nice_tick_interval_us = nice_fraction * exponent;

        // 3. Generate the ticks
        let mut ticks = Vec::new();
        if nice_tick_interval_us == 0.0 {
            return ticks;
        }
        let num_ticks = (duration / nice_tick_interval_us).ceil() as i32;

        for i in 0..=num_ticks {
            let tick_duration = i as f64 * nice_tick_interval_us;
            if tick_duration > duration * 1.001 {
                break;
            }
            let x = tick_duration * scale();
            let original_time = tick_duration + min_start_time as f64;
            let label_val = original_time / divisor;

            // Avoid floating point precision issues in label (e.g., "5.00s" -> "5s")
            let label = if label_val.fract().abs() < 1e-9 {
                format!("{:.0}{}", label_val, unit_label)
            } else {
                format!("{:.2}{}", label_val, unit_label)
            };

            ticks.push((x, label));
        }
        ticks
    };

    view! {
        <div>
            <div class="flex space-x-2 mb-2">
                <button
                    class="px-2 py-1 border rounded bg-slate-100 dark:bg-slate-700 text-slate-900 dark:text-slate-200 border-slate-300 dark:border-slate-600"
                    on:click=move |_| set_zoom.update(|z| *z *= 1.5)
                >
                    "Zoom In"
                </button>
                <button
                    class="px-2 py-1 border rounded bg-slate-100 dark:bg-slate-700 text-slate-900 dark:text-slate-200 border-slate-300 dark:border-slate-600"
                    on:click=move |_| set_zoom.update(|z| *z /= 1.5)
                >
                    "Zoom Out"
                </button>
                <button
                    class="px-2 py-1 border rounded bg-slate-100 dark:bg-slate-700 text-slate-900 dark:text-slate-200 border-slate-300 dark:border-slate-600"
                    on:click=move |_| set_zoom.set(initial_zoom)
                >
                    "Reset"
                </button>
            </div>
            <div
                style=format!(
                    "overflow: auto; width: {}px; height: {}px; border: 1px solid #ccc;",
                    width,
                    height,
                )
                class="rounded"
            >
                <svg
                    class="bazel-trace-chart"
                    xmlns="http://www.w3.org/2000/svg"
                    width=move || TRACE_NAME_WIDTH + timeline_width()
                    height=total_height
                    viewBox=move || {
                        format!("0 0 {} {}", TRACE_NAME_WIDTH + timeline_width(), total_height)
                    }
                >
                    // X-Axis
                    <g
                        class="x-axis"
                        transform=format!("translate({}, {})", TRACE_NAME_WIDTH, X_AXIS_HEIGHT)
                    >
                        <line
                            x1="0"
                            y1="0"
                            x2=timeline_width
                            y2="0"
                            class="stroke-slate-900 dark:stroke-slate-200"
                        />
                        <For
                            each=x_axis_ticks
                            key=|(_, label)| label.clone()
                            children=move |(x, label)| {
                                view! {
                                    <g>
                                        <line
                                            x1=x
                                            y1="-5"
                                            x2=x
                                            y2="0"
                                            class="stroke-slate-900 dark:stroke-slate-200"
                                        />
                                        <text
                                            x=x
                                            y="-8"
                                            text-anchor="middle"
                                            font-size="10"
                                            class="fill-slate-900 dark:fill-slate-200"
                                        >
                                            {label}
                                        </text>
                                    </g>
                                }
                            }
                        />
                    </g>

                    // Traces
                    <g class="traces" transform=format!("translate(0, {})", X_AXIS_HEIGHT)>
                        {
                            let trace_y_offsets: Vec<f64> = layouts
                                .with_value(|l| {
                                    l.iter()
                                        .scan(
                                            0.0,
                                            |state, (_, num_rows)| {
                                                let current_y = *state;
                                                *state += *num_rows as f64 * ROW_HEIGHT;
                                                Some(current_y)
                                            },
                                        )
                                        .collect()
                                });
                            bazel_trace
                                .traces
                                .into_iter()
                                .zip(layouts.with_value(|l| l.clone()).into_iter())
                                .zip(trace_y_offsets.into_iter())
                                .map(|((trace, (positioned_events, num_rows)), current_y)| {
                                    let trace_height = num_rows as f64 * ROW_HEIGHT;

                                    view! {
                                        <g
                                            class="trace-group"
                                            transform=format!("translate(0, {})", current_y)
                                        >
                                            // Trace Border
                                            <rect
                                                x="0"
                                                y="0"
                                                width=move || TRACE_NAME_WIDTH + timeline_width()
                                                height=trace_height
                                                fill="none"
                                                class="stroke-slate-200 dark:stroke-slate-700"
                                            />

                                            // Trace Name
                                            <text
                                                x="10"
                                                y=trace_height / 2.0
                                                dominant-baseline="middle"
                                                font-size="12"
                                                class="fill-slate-900 dark:fill-slate-200"
                                            >
                                                {trace.name}
                                            </text>

                                            // Timeline
                                            <g
                                                class="timeline"
                                                transform=format!("translate({}, 0)", TRACE_NAME_WIDTH)
                                            >
                                                <For
                                                    each=move || positioned_events.clone()
                                                    key=|p_event| p_event.id
                                                    children=move |p_event| {
                                                        let event = p_event.event;
                                                        let y = p_event.row as f64 * ROW_HEIGHT + V_PADDING;
                                                        let color = color_for_category(&event.category);
                                                        let normalized_start =
                                                            (event.start - min_start_time) as f64;

                                                        view! {
                                                            <rect
                                                                x=move || normalized_start * scale()
                                                                y=y
                                                                width=move || {
                                                                    (event.duration.unwrap_or(1) as f64 * scale()).max(1.0)
                                                                }
                                                                height=EVENT_HEIGHT
                                                                fill=color
                                                                on:mouseover={
                                                                    let event_clone = event.clone();
                                                                    move |_| show_tooltip(&format!("{:?}", event_clone))
                                                                }
                                                                on:mouseout=move |_| hide_tooltip()
                                                            />
                                                        }
                                                    }
                                                />
                                            </g>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }
                    </g>
                </svg>
            </div>
        </div>
    }
}

fn show_tooltip(details: &str) {
    tracing::info!("Tooltip: {}", details);
}

fn hide_tooltip() {
    tracing::info!("Tooltip hidden");
}
