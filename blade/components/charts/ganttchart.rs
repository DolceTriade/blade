use leptos::{html, prelude::*};
use trace_event_parser::{BazelTrace, Counter, Event};

const TRACE_NAME_WIDTH: f64 = 200.0;
const ROW_HEIGHT: f64 = 30.0;
const EVENT_HEIGHT: f64 = 20.0;
const V_PADDING: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 30.0; // Increased for more space
const COUNTER_CHART_HEIGHT: f64 = 50.0;

#[derive(Clone, Debug)]
struct PositionedEvent {
    id: String,
    event: Event,
    row: usize,
}

fn calculate_layout(events: &[Event], trace_index: usize) -> (Vec<PositionedEvent>, usize) {
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
                    id: format!("{}-{}", trace_index, i),
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
                id: format!("{}-{}", trace_index, i),
                event: event.clone(),
                row: new_row,
            });
            row_ends.push(end_time);
        }
    }

    (positioned_events, row_ends.len().max(1))
}

fn contrasting_text_color(hex_color: &str) -> &'static str {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return "#000000"; // Default to black
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    // Using the luminance formula
    let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;

    if luminance > 0.5 {
        "#000000" // Black for light backgrounds
    } else {
        "#FFFFFF" // White for dark backgrounds
    }
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

fn format_duration(duration_us: i64) -> String {
    if duration_us < 0 {
        return "0µs".to_string();
    }
    if duration_us >= 1_000_000 {
        format!("{:.3}s", duration_us as f64 / 1_000_000.0)
    } else if duration_us >= 1_000 {
        format!("{:.3}ms", duration_us as f64 / 1_000.0)
    } else {
        format!("{}µs", duration_us)
    }
}

fn format_time(time_us: f64) -> String {
    if time_us < 0.0 {
        return "0µs".to_string();
    }
    if time_us >= 1_000_000.0 {
        format!("{:.3}s", time_us / 1_000_000.0)
    } else if time_us >= 1_000.0 {
        format!("{:.3}ms", time_us / 1_000.0)
    } else {
        format!("{:.0}µs", time_us)
    }
}

#[allow(non_snake_case)]
#[component]
pub fn BazelTraceChart(
    mut bazel_trace: BazelTrace,
    #[prop(default = 800)] height: u32,
) -> impl IntoView {
    // Sort traces by pid and tid to ensure deterministic order
    bazel_trace
        .traces
        .sort_by(|a, b| a.pid.cmp(&b.pid).then(a.tid.cmp(&b.tid)));
    // Sort counters by name to ensure deterministic order
    bazel_trace.counters.sort_by(|a, b| a.name.cmp(&b.name));

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

    let duration = (max_end_time - min_start_time).max(0) as f64;

    let layouts = StoredValue::new(
        bazel_trace
            .traces
            .iter()
            .enumerate()
            .map(|(trace_index, trace)| calculate_layout(&trace.events, trace_index))
            .collect::<Vec<_>>(),
    );

    let counters_height = bazel_trace.counters.len() as f64 * COUNTER_CHART_HEIGHT;

    let traces_height = layouts.with_value(|l| {
        l.iter()
            .map(|(_, num_rows)| *num_rows as f64 * ROW_HEIGHT)
            .sum::<f64>()
    });

    let total_height = traces_height + counters_height + X_AXIS_HEIGHT;

    let bazel_trace = StoredValue::new(bazel_trace);

    let (zoom, set_zoom) = signal(1.0);
    let initial_zoom = RwSignal::new(1.0);

    let hovered_event = RwSignal::new(None::<Event>);
    let tooltip_pos = RwSignal::new((0.0, 0.0));
    let tooltip_visible = RwSignal::new(false);

    let hovered_counter_info = RwSignal::new(None::<(String, f64)>);
    let counter_tooltip_pos = RwSignal::new((0.0, 0.0));
    let counter_tooltip_visible = RwSignal::new(false);

    let hover_time = RwSignal::new(None::<f64>);
    let hover_line_text_pos = RwSignal::new((0.0, 0.0));
    let container_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        if let Some(container) = container_ref.get() {
            let container_width = container.client_width() as f64;
            let new_initial_zoom = if duration > 0.0 {
                (container_width - TRACE_NAME_WIDTH) / duration
            } else {
                1.0
            };
            initial_zoom.set(new_initial_zoom);
            set_zoom.set(new_initial_zoom);
        }
    });

    let timeline_width = Signal::derive(move || duration * zoom.get());

    let x_axis_ticks = Memo::new(move |_| {
        let timeline_w = timeline_width.get();
        if timeline_w <= 0.0 || duration <= 0.0 {
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

        // 2. Calculate a "nice" tick interval.
        let target_tick_spacing_px = 150.0;
        let min_tick_count = (timeline_w / target_tick_spacing_px).floor() as u32;
        let tick_range_us = duration;

        let rough_tick_interval = tick_range_us / (min_tick_count.max(1) as f64);
        let exponent = 10.0_f64.powf(rough_tick_interval.log10().floor());
        let nice_fractions = [1.0, 2.0, 5.0, 10.0];
        let fraction = rough_tick_interval / exponent;
        let nice_fraction = nice_fractions
            .iter()
            .find(|&f| *f >= fraction)
            .unwrap_or(&10.0);
        let nice_tick_interval = nice_fraction * exponent;

        // 3. Generate the ticks based on the nice interval.
        let mut ticks = Vec::new();
        if nice_tick_interval == 0.0 {
            return ticks;
        }

        let first_tick = (min_start_time as f64 / nice_tick_interval).floor() * nice_tick_interval;

        let mut current_tick = first_tick;
        while current_tick <= max_end_time as f64 {
            let normalized_tick = current_tick - min_start_time as f64;
            if normalized_tick >= 0.0 {
                let x = normalized_tick * zoom.get();
                let label_val = current_tick / divisor;

                let display_label = if (label_val.fract().abs() * divisor) < 1.0 {
                    format!("{:.0}{}", label_val.round(), unit_label)
                } else {
                    format!("{:.2}{}", label_val, unit_label)
                };
                ticks.push((x, display_label, current_tick));
            }
            current_tick += nice_tick_interval;
        }
        ticks
    });

    let on_container_mousemove = move |ev: web_sys::MouseEvent| {
        if let Some(container) = container_ref.get() {
            let rect = container.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left() + container.scroll_left() as f64;

            if x >= TRACE_NAME_WIDTH {
                let timeline_x = x - TRACE_NAME_WIDTH;
                let time_us = (timeline_x / zoom.get()) + min_start_time as f64;
                hover_time.set(Some(time_us));
                hover_line_text_pos.set((ev.client_x() as f64, rect.top()));
            } else {
                hover_time.set(None);
            }
        }
    };

    let on_container_mouseleave = move |_| {
        hover_time.set(None);
        counter_tooltip_visible.set(false);
    };

    view! {
        <div class="relative">
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
                        on:click=move |_| set_zoom.set(initial_zoom.get())
                    >
                        "Reset"
                    </button>
                </div>
                <div
                    node_ref=container_ref
                    style=format!(
                        "overflow: auto; width: 100%; height: {}px; border: 1px solid #ccc;",
                        height,
                    )
                    class="rounded"
                    on:mousemove=on_container_mousemove
                    on:mouseleave=on_container_mouseleave
                >
                    <svg
                        class="bazel-trace-chart"
                        xmlns="http://www.w3.org/2000/svg"
                        width=move || TRACE_NAME_WIDTH + timeline_width.get()
                        height=total_height
                        viewBox=move || {
                            format!(
                                "0 0 {} {}",
                                TRACE_NAME_WIDTH + timeline_width.get(),
                                total_height,
                            )
                        }
                    >
                        // Definitions for clipping paths
                        <defs>
                            {layouts
                                .with_value(|l| l.clone())
                                .into_iter()
                                .flat_map(|(events, _)| events)
                                .map(|p_event| {
                                    let event = p_event.event;
                                    let event_width = Signal::derive(move || {
                                        (event.duration.unwrap_or(1) as f64 * zoom.get()).max(1.0)
                                    });
                                    view! {
                                        <clipPath id=format!("clip-{}", p_event.id)>
                                            <rect x="0" y="0" width=event_width height=EVENT_HEIGHT />
                                        </clipPath>
                                    }
                                })
                                .collect_view()}
                        </defs>

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
                                each=move || x_axis_ticks.get()
                                key=move |(_, _, tick_val)| *tick_val as u64
                                children=move |(x, label, _)| {
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

                        // Counter Names Sidebar
                        <g
                            class="counter-names"
                            transform=format!("translate(0, {})", X_AXIS_HEIGHT)
                        >
                            <rect
                                x="0"
                                y="0"
                                width=TRACE_NAME_WIDTH
                                height=counters_height
                                class="fill-slate-50 dark:fill-slate-800"
                            />
                            <For
                                each=move || {
                                    bazel_trace.with_value(|bt| bt.counters.clone()).into_iter().enumerate()
                                }
                                key=|(_, counter)| counter.name.clone()
                                children=move |(i, counter)| {
                                    let y = i as f64 * COUNTER_CHART_HEIGHT;
                                    let (_, max_val) = counter
                                        .time_series
                                        .iter()
                                        .fold((f64::MAX, f64::MIN), |(min, max), point| {
                                            (min.min(point.value), max.max(point.value))
                                        });

                                    view! {
                                        <g>
                                            <text
                                                x="10"
                                                y=y + COUNTER_CHART_HEIGHT / 2.0
                                                dominant-baseline="middle"
                                                font-size="12"
                                                class="fill-slate-900 dark:fill-slate-200"
                                            >
                                                {counter.name}
                                            </text>
                                            <text
                                                x=TRACE_NAME_WIDTH - 10.0
                                                y=y + 15.0
                                                text-anchor="end"
                                                font-size="10"
                                                class="fill-slate-500 dark:fill-slate-400"
                                            >
                                                {format!("{:.2}", max_val)}
                                            </text>
                                            <line
                                                x1="0"
                                                y1=y + COUNTER_CHART_HEIGHT
                                                x2=TRACE_NAME_WIDTH
                                                y2=y + COUNTER_CHART_HEIGHT
                                                class="stroke-slate-200 dark:stroke-slate-700"
                                            />
                                        </g>
                                    }
                                }
                            />
                        </g>

                        // Counter Charts
                        <g
                            class="counters"
                            transform=format!("translate({}, {})", TRACE_NAME_WIDTH, X_AXIS_HEIGHT)
                        >
                            <For
                                each=move || {
                                    bazel_trace.with_value(|bt| bt.counters.clone()).into_iter().enumerate()
                                }
                                key=|(_, counter)| counter.name.clone()
                                children=move |(i, counter): (usize, Counter)| {
                                    let y_offset = i as f64 * COUNTER_CHART_HEIGHT;
                                    let (min_val, max_val) = counter
                                        .time_series
                                        .iter()
                                        .fold((f64::MAX, f64::MIN), |(min, max), point| {
                                            (min.min(point.value), max.max(point.value))
                                        });

                                    let time_series_for_path = counter.time_series.clone();
                                    let path_data = Signal::derive(move || {
                                        if time_series_for_path.is_empty() {
                                            return "M 0 0".to_string();
                                        }

                                        let first_point = &time_series_for_path[0];
                                        let first_x =
                                            (first_point.timestamp - min_start_time) as f64 * zoom.get();
                                        let first_y = if max_val > min_val {
                                            COUNTER_CHART_HEIGHT
                                                - ((first_point.value - min_val) / (max_val - min_val))
                                                    * COUNTER_CHART_HEIGHT
                                        } else {
                                            COUNTER_CHART_HEIGHT / 2.0
                                        };

                                        let mut d = format!(
                                            "M {} {} L {} {}",
                                            first_x, COUNTER_CHART_HEIGHT, first_x, first_y
                                        );

                                        for i in 1..time_series_for_path.len() {
                                            let prev_point = &time_series_for_path[i - 1];
                                            let curr_point = &time_series_for_path[i];

                                            let prev_y = if max_val > min_val {
                                                COUNTER_CHART_HEIGHT
                                                    - ((prev_point.value - min_val)
                                                        / (max_val - min_val))
                                                        * COUNTER_CHART_HEIGHT
                                            } else {
                                                COUNTER_CHART_HEIGHT / 2.0
                                            };

                                            let curr_x = (curr_point.timestamp - min_start_time) as f64
                                                * zoom.get();
                                            let curr_y = if max_val > min_val {
                                                COUNTER_CHART_HEIGHT
                                                    - ((curr_point.value - min_val)
                                                        / (max_val - min_val))
                                                        * COUNTER_CHART_HEIGHT
                                            } else {
                                                COUNTER_CHART_HEIGHT / 2.0
                                            };

                                            d.push_str(&format!(" L {} {}", curr_x, prev_y));
                                            d.push_str(&format!(" L {} {}", curr_x, curr_y));
                                        }

                                        let last_x =
                                            (time_series_for_path.last().unwrap().timestamp - min_start_time)
                                                as f64 * zoom.get();
                                        d.push_str(&format!(" L {} {}", last_x, COUNTER_CHART_HEIGHT));
                                        d.push('Z');
                                        d
                                    });

                                    let on_counter_mousemove = {
                                        let counter_name = counter.name.clone();
                                        let time_series = counter.time_series;
                                        move |ev: web_sys::MouseEvent| {
                                            ev.stop_propagation();
                                            if let Some(container) = container_ref.get() {
                                                let rect = container.get_bounding_client_rect();
                                                let x = ev.client_x() as f64 - rect.left()
                                                    + container.scroll_left() as f64;
                                                let timeline_x = x - TRACE_NAME_WIDTH;

                                                if timeline_x < 0.0 {
                                                    return;
                                                }

                                                let time_us = (timeline_x / zoom.get())
                                                    + min_start_time as f64;

                                                let value = match time_series.binary_search_by(|p| {
                                                    (p.timestamp as f64).total_cmp(&time_us)
                                                }) {
                                                    Ok(i) => time_series[i].value,
                                                    Err(i) => {
                                                        if i == 0 {
                                                            // Hovering before the first data point
                                                            0.0
                                                        } else if i >= time_series.len() {
                                                            // Hovering after the last data point
                                                            time_series.last().unwrap().value
                                                        } else {
                                                            // Hovering between two data points
                                                            time_series[i - 1].value
                                                        }
                                                    },
                                                };

                                                hovered_counter_info
                                                    .set(Some((counter_name.clone(), value)));
                                                counter_tooltip_pos.set((
                                                    ev.client_x() as f64,
                                                    ev.client_y() as f64,
                                                ));
                                                counter_tooltip_visible.set(true);
                                            }
                                        }
                                    };

                                    let on_counter_mouseout = move |ev: web_sys::MouseEvent| {
                                        ev.stop_propagation();
                                        counter_tooltip_visible.set(false);
                                    };

                                    view! {
                                        <g transform=format!("translate(0, {})", y_offset)>
                                            <path
                                                d=path_data
                                                fill=color_for_category(&counter.name)
                                                fill-opacity="0.5"
                                                class="stroke-slate-200 dark:stroke-slate-700"
                                                stroke-width="1"
                                                on:mousemove=on_counter_mousemove
                                                on:mouseout=on_counter_mouseout
                                            />
                                        </g>
                                    }
                                }
                            />
                        </g>

                        // Trace Names Sidebar
                        <g
                            class="trace-names"
                            transform=format!("translate(0, {})", X_AXIS_HEIGHT + counters_height)
                        >
                            <rect
                                x="0"
                                y="0"
                                width=TRACE_NAME_WIDTH
                                height=traces_height
                                class="fill-slate-50 dark:fill-slate-800"
                            />
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
                                bazel_trace.with_value(|bt| {
                                    bt.traces
                                        .iter()
                                        .zip(layouts.with_value(|l| l.clone()).into_iter())
                                        .zip(trace_y_offsets.into_iter())
                                        .map(|((trace, (_, num_rows)), current_y)| {
                                            let trace_height = num_rows as f64 * ROW_HEIGHT;
                                            view! {
                                                <g>
                                                    <text
                                                        x="10"
                                                        y=current_y + trace_height / 2.0
                                                        dominant-baseline="middle"
                                                        font-size="12"
                                                        class="fill-slate-900 dark:fill-slate-200"
                                                    >
                                                        {format!("{} (tid: {})", trace.name, trace.tid)}
                                                    </text>
                                                    <line
                                                        x1="0"
                                                        y1=current_y + trace_height
                                                        x2=TRACE_NAME_WIDTH
                                                        y2=current_y + trace_height
                                                        class="stroke-slate-200 dark:stroke-slate-700"
                                                    />
                                                </g>
                                            }
                                        })
                                        .collect_view()
                                })
                            }
                        </g>

                        // Traces
                        <g
                            class="traces"
                            transform=format!("translate(0, {})", X_AXIS_HEIGHT + counters_height)
                        >
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
                                bazel_trace.with_value(|bt| {
                                    bt.traces
                                        .clone()
                                        .into_iter()
                                        .zip(layouts.with_value(|l| l.clone()).into_iter())
                                        .zip(trace_y_offsets.into_iter())
                                        .map(|((_, (positioned_events, num_rows)), current_y)| {
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
                                                        width=move || {
                                                            TRACE_NAME_WIDTH + timeline_width.get()
                                                        }
                                                        height=trace_height
                                                        fill="none"
                                                        class="stroke-slate-200 dark:stroke-slate-700"
                                                    />

                                                    // Timeline
                                                    <g
                                                        class="timeline"
                                                        transform=format!(
                                                            "translate({}, 0)",
                                                            TRACE_NAME_WIDTH,
                                                        )
                                                    >
                                                        <For
                                                            each=move || positioned_events.clone()
                                                            key=|p_event| p_event.id.clone()
                                                            children=move |p_event| {
                                                                let event = p_event.event;
                                                                let y = p_event.row as f64
                                                                    * ROW_HEIGHT + V_PADDING;
                                                                let color =
                                                                    color_for_category(&event.category);
                                                                let normalized_start = (event
                                                                    .start
                                                                    - min_start_time)
                                                                    as f64;
                                                                let event_width = Signal::derive(
                                                                    move || {
                                                                        (event
                                                                            .duration
                                                                            .unwrap_or(1)
                                                                            as f64
                                                                            * zoom.get())
                                                                        .max(1.0)
                                                                    },
                                                                );
                                                                let transform = Signal::derive(
                                                                    move || {
                                                                        format!(
                                                                            "translate({}, {})",
                                                                            normalized_start
                                                                                * zoom.get(),
                                                                            y,
                                                                        )
                                                                    },
                                                                );

                                                                view! {
                                                                    <g transform=transform>
                                                                        <rect
                                                                            x="0"
                                                                            y="0"
                                                                            width=event_width
                                                                            height=EVENT_HEIGHT
                                                                            fill=color.clone()
                                                                            on:mousemove=move |ev| {
                                                                                ev.stop_propagation()
                                                                            }
                                                                            on:mouseover={
                                                                                let event_clone = event
                                                                                    .clone();
                                                                                move |ev: web_sys::MouseEvent| {
                                                                                    ev
                                                                                        .stop_propagation();
                                                                                    hovered_event
                                                                                        .set(
                                                                                            Some(
                                                                                                event_clone
                                                                                                    .clone(),
                                                                                            ),
                                                                                        );
                                                                                    tooltip_pos
                                                                                        .set((
                                                                                            ev.client_x()
                                                                                                as f64,
                                                                                            ev.client_y()
                                                                                                as f64,
                                                                                        ));
                                                                                    tooltip_visible
                                                                                        .set(true);
                                                                                }
                                                                            }
                                                                            on:mouseout=move |ev| {
                                                                                ev
                                                                                    .stop_propagation();
                                                                                hovered_event
                                                                                    .set(None);
                                                                                tooltip_visible
                                                                                    .set(false);
                                                                            }
                                                                        />
                                                                        <Show when=move || {
                                                                            event_width.get() > 30.0
                                                                        }>
                                                                            <text
                                                                                x="5"
                                                                                y=EVENT_HEIGHT / 2.0
                                                                                dominant-baseline="middle"
                                                                                font-size="12"
                                                                                fill=contrasting_text_color(
                                                                                    &color,
                                                                                )
                                                                                clip-path=format!(
                                                                                    "url(#clip-{})",
                                                                                    p_event.id,
                                                                                )
                                                                                class="pointer-events-none"
                                                                            >
                                                                                {event.name.clone()}
                                                                            </text>
                                                                        </Show>
                                                                    </g>
                                                                }
                                                            }
                                                        />
                                                    </g>
                                                </g>
                                            }
                                        })
                                        .collect_view()
                                })
                            }
                        </g>
                        <Show when=move || {
                            hover_time.get().is_some() && !tooltip_visible.get()
                        }>
                            {move || {
                                let time = hover_time.get().unwrap();
                                let x = (time - min_start_time as f64) * zoom.get();
                                view! {
                                    <g
                                        class="pointer-events-none"
                                        transform=format!("translate({}, 0)", TRACE_NAME_WIDTH)
                                    >
                                        <line
                                            x1=x
                                            y1=X_AXIS_HEIGHT
                                            x2=x
                                            y2=total_height
                                            class="stroke-red-500"
                                            stroke-dasharray="4"
                                        />
                                    </g>
                                }
                            }}
                        </Show>
                    </svg>
                </div>
            </div>
            <div
                class="absolute z-10 p-1 bg-red-500 text-white text-xs rounded pointer-events-none"
                style=move || {
                    let (x, y) = hover_line_text_pos.get();
                    let display = if hover_time.get().is_some() && !tooltip_visible.get() {
                        "block"
                    } else {
                        "none"
                    };
                    format!(
                        "position: fixed; left: {}px; top: {}px; transform: translate(10px, 10px); display: {};",
                        x,
                        y,
                        display,
                    )
                }
            >
                {move || hover_time.get().map(format_time)}
            </div>
            <div
                class="absolute z-10 p-2 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded shadow-lg pointer-events-none"
                style=move || {
                    let (x, y) = counter_tooltip_pos.get();
                    let display = if counter_tooltip_visible.get() {
                        "block"
                    } else {
                        "none"
                    };
                    format!(
                        "position: fixed; left: {}px; top: {}px; transform: translate(10px, 10px); display: {};",
                        x,
                        y,
                        display,
                    )
                }
            >
                {move || {
                    hovered_counter_info
                        .get()
                        .map(|(name, value)| {
                            view! {
                                <div class="text-sm text-slate-900 dark:text-slate-200">
                                    <div class="font-bold">{name}</div>
                                    <div>
                                        <strong>"Value: "</strong>
                                        {format!("{:.2}", value)}
                                    </div>
                                </div>
                            }
                        })
                }}
            </div>
            <div
                class="absolute z-10 p-2 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded shadow-lg pointer-events-none"
                style=move || {
                    let (x, y) = tooltip_pos.get();
                    let display = if tooltip_visible.get() { "block" } else { "none" };
                    format!(
                        "position: fixed; left: {}px; top: {}px; transform: translate(10px, 10px); display: {};",
                        x,
                        y,
                        display,
                    )
                }
            >
                {move || {
                    hovered_event
                        .get()
                        .map(|event| {
                            view! {
                                <div class="text-sm text-slate-900 dark:text-slate-200">
                                    <div class="font-bold">{event.name}</div>
                                    <div>
                                        <strong>"Category: "</strong>
                                        {event.category}
                                    </div>
                                    <div>
                                        <strong>"Duration: "</strong>
                                        {format_duration(event.duration.unwrap_or(0))}
                                    </div>
                                    {event
                                        .args
                                        .map(|args| {
                                            view! {
                                                <div>
                                                    <strong>"Args: "</strong>
                                                    {format!(
                                                        "{}",
                                                        serde_json::to_string(&args).unwrap_or_default(),
                                                    )}
                                                </div>
                                            }
                                        })}
                                </div>
                            }
                        })
                }}
            </div>
        </div>
    }
}
