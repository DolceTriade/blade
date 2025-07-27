use leptos::{html, prelude::*};
use trace_event_parser::{BazelTrace, Event};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};

const TRACE_NAME_WIDTH: f64 = 200.0;
const ROW_HEIGHT: f64 = 30.0;
const EVENT_HEIGHT: f64 = 20.0;
const V_PADDING: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 30.0;
const COUNTER_CHART_HEIGHT: f64 = 50.0;
const COUNTER_CHART_TOP_MARGIN: f64 = 10.0;

#[derive(Clone, Debug)]
struct PositionedEvent {
    id: String,
    event: Event,
    // Pre-computed positioning data
    normalized_start: f64,
    y_position: f64,
    color: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ComputedEventLayout {
    id: String,
    event: Event,
    x: f64,
    y: f64,
    width: f64,
    color: String,
    show_text: bool,
}

fn calculate_layout(
    events: &[Event],
    trace_index: usize,
    min_start_time: i64,
) -> (Vec<PositionedEvent>, usize) {
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
                    id: format!("{trace_index}-{i}"),
                    normalized_start: (event.start - min_start_time) as f64,
                    y_position: j as f64 * ROW_HEIGHT + V_PADDING,
                    color: color_for_category(&event.category),
                    event: event.clone(),
                });
                *row_end = end_time;
                placed = true;
                break;
            }
        }

        if !placed {
            let new_row = row_ends.len();
            positioned_events.push(PositionedEvent {
                id: format!("{trace_index}-{i}"),
                normalized_start: (event.start - min_start_time) as f64,
                y_position: new_row as f64 * ROW_HEIGHT + V_PADDING,
                color: color_for_category(&event.category),
                event: event.clone(),
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
    format!("#{r:02x}{g:02x}{b:02x}")
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
        format!("{duration_us}µs")
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
        format!("{time_us:.0}µs")
    }
}

fn find_event_at_position(
    events: &[ComputedEventLayout],
    x: f64,
    y: f64,
    viewport_left: f64,
) -> Option<&ComputedEventLayout> {
    let canvas_x = x + viewport_left;

    for event in events {
        if canvas_x >= event.x
            && canvas_x <= event.x + event.width
            && y >= event.y
            && y <= event.y + EVENT_HEIGHT
        {
            return Some(event);
        }
    }
    None
}

fn render_gantt_chart_viewport(
    ctx: &CanvasRenderingContext2d,
    trace_data: &BazelTrace,
    layout_data: &[(Vec<PositionedEvent>, usize)],
    zoom: f64,
    min_start_time: i64,
    total_height: f64,
    viewport_width: f64,
    traces_height: f64,
    counters_height: f64,
    scroll_offset: f64,
) {
    // Draw sidebar background (always visible)
    ctx.set_fill_style_str("#f8fafc"); // slate-50
    ctx.fill_rect(0.0, 0.0, TRACE_NAME_WIDTH, total_height);

    // Draw timeline background
    ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(TRACE_NAME_WIDTH, 0.0, viewport_width, total_height);

    // Calculate viewport bounds in timeline coordinates
    let viewport_start = scroll_offset;
    let viewport_end = scroll_offset + viewport_width;

    // Draw x-axis
    let axis_y = X_AXIS_HEIGHT;
    ctx.set_stroke_style_str("#1f2937"); // gray-800
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(TRACE_NAME_WIDTH, axis_y);
    ctx.line_to(TRACE_NAME_WIDTH + viewport_width, axis_y);
    ctx.stroke();

    // Draw counter section
    let counter_start_y = X_AXIS_HEIGHT + COUNTER_CHART_TOP_MARGIN;
    for (i, counter) in trace_data.counters.iter().enumerate() {
        let counter_y = counter_start_y + i as f64 * COUNTER_CHART_HEIGHT;

        // Draw counter name (always visible)
        ctx.set_fill_style_str("#1f2937");
        ctx.set_font("12px sans-serif");
        let _ = ctx.fill_text(&counter.name, 10.0, counter_y + COUNTER_CHART_HEIGHT / 2.0);

        // Draw counter chart if has data and intersects viewport
        if !counter.time_series.is_empty() {
            let (min_val, max_val) = counter.time_series.iter().fold(
                (f64::MAX, f64::MIN),
                |(min, max), point| (min.min(point.value), max.max(point.value))
            );

            ctx.set_fill_style_str(&color_for_category(&counter.name));
            ctx.set_global_alpha(0.5);
            ctx.begin_path();

            // Only draw points that are visible in viewport
            let mut path_started = false;
            for point in &counter.time_series {
                let point_x = (point.timestamp - min_start_time) as f64 * zoom;

                // Skip points that are way outside viewport (with some buffer)
                if point_x < viewport_start - 100.0 || point_x > viewport_end + 100.0 {
                    continue;
                }

                let canvas_x = TRACE_NAME_WIDTH + point_x - scroll_offset;
                let point_y = if max_val > min_val {
                    counter_y + COUNTER_CHART_HEIGHT - ((point.value - min_val) / (max_val - min_val)) * COUNTER_CHART_HEIGHT
                } else {
                    counter_y + COUNTER_CHART_HEIGHT / 2.0
                };

                if !path_started {
                    ctx.move_to(canvas_x, counter_y + COUNTER_CHART_HEIGHT);
                    ctx.line_to(canvas_x, point_y);
                    path_started = true;
                } else {
                    ctx.line_to(canvas_x, point_y);
                }
            }

            if path_started {
                // Close to bottom - use the last drawn x position
                ctx.line_to(TRACE_NAME_WIDTH + viewport_width - scroll_offset, counter_y + COUNTER_CHART_HEIGHT);
                ctx.close_path();
                ctx.fill();
            }
            ctx.set_global_alpha(1.0);
        }

        // Draw counter separator
        ctx.set_stroke_style_str("#e5e7eb");
        ctx.begin_path();
        ctx.move_to(0.0, counter_y + COUNTER_CHART_HEIGHT);
        ctx.line_to(TRACE_NAME_WIDTH + viewport_width, counter_y + COUNTER_CHART_HEIGHT);
        ctx.stroke();
    }

    // Draw traces section
    let traces_start_y = X_AXIS_HEIGHT + counters_height + COUNTER_CHART_TOP_MARGIN;
    let mut current_trace_y = traces_start_y;

    for (trace_idx, trace) in trace_data.traces.iter().enumerate() {
        if let Some((positioned_events, num_rows)) = layout_data.get(trace_idx) {
            let trace_height = *num_rows as f64 * ROW_HEIGHT;

            // Draw trace name (always visible)
            ctx.set_fill_style_str("#1f2937");
            ctx.set_font("12px sans-serif");
            let trace_name = format!("{} (tid: {})", trace.name, trace.tid);
            let _ = ctx.fill_text(&trace_name, 10.0, current_trace_y + trace_height / 2.0);

            // Draw trace background
            ctx.set_fill_style_str("rgba(248, 250, 252, 0.5)");
            ctx.fill_rect(TRACE_NAME_WIDTH, current_trace_y, viewport_width, trace_height);

            // Draw events that intersect with viewport
            for p_event in positioned_events {
                let event_x = p_event.normalized_start * zoom;
                let event_width = (p_event.event.duration.unwrap_or(1) as f64 * zoom).max(1.0);

                // Skip events outside viewport
                if event_x + event_width < viewport_start || event_x > viewport_end {
                    continue;
                }

                let canvas_x = TRACE_NAME_WIDTH + event_x - scroll_offset;
                let event_y = current_trace_y + p_event.y_position;

                // Draw event rectangle
                ctx.set_fill_style_str(&p_event.color);
                ctx.fill_rect(canvas_x, event_y, event_width, EVENT_HEIGHT);

                // Draw event text if wide enough
                if event_width > 50.0 {
                    ctx.set_fill_style_str(contrasting_text_color(&p_event.color));
                    ctx.set_font("10px sans-serif");
                    let _ = ctx.fill_text(&p_event.event.name, canvas_x + 2.0, event_y + EVENT_HEIGHT / 2.0 + 3.0);
                }
            }

            // Draw trace separator
            ctx.set_stroke_style_str("#e5e7eb");
            ctx.begin_path();
            ctx.move_to(0.0, current_trace_y + trace_height);
            ctx.line_to(TRACE_NAME_WIDTH + viewport_width, current_trace_y + trace_height);
            ctx.stroke();

            current_trace_y += trace_height;
        }
    }
}#[allow(non_snake_case)]
#[component]
pub fn BazelTraceChart(
    bazel_trace: BazelTrace,
    #[prop(default = 800)] height: u32,
) -> impl IntoView {
    // Pre-process data once
    let mut sorted_trace = bazel_trace.clone();
    sorted_trace.traces.sort_by(|a, b| a.pid.cmp(&b.pid).then(a.tid.cmp(&b.tid)));
    sorted_trace.counters.sort_by(|a, b| a.name.cmp(&b.name));

    let (mut min_start_time, mut max_end_time) = sorted_trace
        .traces
        .iter()
        .flat_map(|trace| &trace.events)
        .fold((i64::MAX, 0), |(min_s, max_e), event| {
            (
                min_s.min(event.start),
                max_e.max(event.start + event.duration.unwrap_or(1)),
            )
        });

    let (min_counter_time, max_counter_time) = sorted_trace
        .counters
        .iter()
        .flat_map(|c| &c.time_series)
        .fold((i64::MAX, 0), |(min_t, max_t), point| {
            (min_t.min(point.timestamp), max_t.max(point.timestamp))
        });

    if min_counter_time != i64::MAX {
        min_start_time = min_start_time.min(min_counter_time);
    }
    max_end_time = max_end_time.max(max_counter_time);

    let min_start_time = if min_start_time == i64::MAX { 0 } else { min_start_time };
    let duration = (max_end_time - min_start_time).max(1) as f64;

    // Pre-compute layouts once
    let computed_layouts = sorted_trace
        .traces
        .iter()
        .enumerate()
        .map(|(trace_index, trace)| {
            calculate_layout(&trace.events, trace_index, min_start_time)
        })
        .collect::<Vec<_>>();

    let counters_height = sorted_trace.counters.len() as f64 * COUNTER_CHART_HEIGHT;
    let traces_height = computed_layouts
        .iter()
        .map(|(_, num_rows)| *num_rows as f64 * ROW_HEIGHT)
        .sum::<f64>();
    let total_height = traces_height + counters_height + X_AXIS_HEIGHT + COUNTER_CHART_TOP_MARGIN;

    // Store immutable data
    let trace_data = StoredValue::new(sorted_trace);
    let layout_data = StoredValue::new(computed_layouts);

    // Reactive state
    let (zoom, set_zoom) = signal(1.0);
    let initial_zoom = RwSignal::new(1.0);
    let hovered_event = RwSignal::new(None::<Event>);
    let tooltip_pos = RwSignal::new((0.0, 0.0));
    let tooltip_visible = RwSignal::new(false);
    let hover_time = RwSignal::new(None::<f64>);
    let hover_line_text_pos = RwSignal::new((0.0, 0.0));

    // Canvas refs
    let container_ref = NodeRef::<html::Div>::new();
    let main_canvas_ref = NodeRef::<html::Canvas>::new();

    // Track scroll and viewport
    let scroll_left = RwSignal::new(0.0);
    let viewport_width = RwSignal::new(800.0);

    // Initialize zoom based on container size
    Effect::new(move |_| {
        if let Some(container) = container_ref.get() {
            let container_width = container.client_width() as f64;
            if container_width > 0.0 {
                let new_initial_zoom = if duration > 0.0 {
                    let duration_ms = duration / 1000.0; // Convert microseconds to milliseconds
                    let available_width = container_width - TRACE_NAME_WIDTH;
                    let target_width = duration_ms.max(available_width); // Ensure reasonable scale
                    available_width / target_width * 1000.0 // Convert back to microseconds scale
                } else {
                    1.0
                };
                initial_zoom.set(new_initial_zoom);
                set_zoom.set(new_initial_zoom);
                viewport_width.set(container_width);
            }
        }
    });

    // Main canvas rendering effect
    Effect::new(move |_| {
        let current_zoom = zoom.get();
        let viewport_w = viewport_width.get();
        let scroll_offset = scroll_left.get();

        if let Some(canvas) = main_canvas_ref.get() {
            let canvas: HtmlCanvasElement = canvas.unchecked_into();

            // Calculate timeline width at current zoom
            let timeline_width_val = duration * current_zoom;

            // Limit canvas size to viewport width + sidebar for performance
            let canvas_width = TRACE_NAME_WIDTH + viewport_w;
            canvas.set_width(canvas_width as u32);
            canvas.set_height(total_height as u32);

            // Set CSS size to full timeline width for proper scrolling
            let canvas_element: &web_sys::Element = canvas.as_ref();
            if let Some(html_element) = canvas_element.dyn_ref::<web_sys::HtmlElement>() {
                let style = html_element.style();
                let _ = style.set_property("width", &format!("{}px", TRACE_NAME_WIDTH + timeline_width_val));
                let _ = style.set_property("height", &format!("{}px", total_height));
            }

            if let Ok(ctx) = canvas.get_context("2d") {
                if let Some(ctx) = ctx {
                    let ctx: CanvasRenderingContext2d = ctx.unchecked_into();

                    // Clear entire canvas
                    ctx.clear_rect(0.0, 0.0, canvas_width, total_height);

                    // Render with current zoom level and scroll offset
                    trace_data.with_value(|bt| {
                        layout_data.with_value(|layouts| {
                            render_gantt_chart_viewport(&ctx, bt, layouts, current_zoom, min_start_time, total_height, viewport_w, traces_height, counters_height, scroll_offset);
                        });
                    });
                }
            }
        }
    });

    // Simplified event handlers
    let on_container_scroll = move |_| {
        if let Some(container) = container_ref.get() {
            scroll_left.set(container.scroll_left() as f64);
            viewport_width.set(container.client_width() as f64);
        }
    };

    let on_canvas_mousemove = move |ev: web_sys::MouseEvent| {
        if let Some(canvas) = main_canvas_ref.get() {
            let rect = canvas.get_bounding_client_rect();
            let canvas_x = ev.client_x() as f64 - rect.left();
            let canvas_y = ev.client_y() as f64 - rect.top();

            // Check if in timeline area
            if canvas_x >= TRACE_NAME_WIDTH {
                let timeline_x = canvas_x - TRACE_NAME_WIDTH + scroll_left.get();
                let time_us = (timeline_x / zoom.get()) + min_start_time as f64;
                hover_time.set(Some(time_us));
                hover_line_text_pos.set((ev.client_x() as f64, rect.top()));

                // TODO: Implement proper event detection for tooltips
                // This would require checking which event is under the mouse cursor
                // taking into account the current zoom level and scroll position
                tooltip_visible.set(false);
            } else {
                hover_time.set(None);
                tooltip_visible.set(false);
            }
        }
    };

    let on_canvas_mouseleave = move |_| {
        hovered_event.set(None);
        tooltip_visible.set(false);
        hover_time.set(None);
    };

    view! {
        <div class="relative">
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
                style=format!("height: {height}px;")
                class="rounded overflow-auto max-w-full w-full border"
                on:scroll=on_container_scroll
            >
                <canvas
                    node_ref=main_canvas_ref
                    style="display: block; cursor: crosshair;"
                    on:mousemove=on_canvas_mousemove
                    on:mouseleave=on_canvas_mouseleave
                />
            </div>

            // Time tooltip
            <div
                class="absolute z-10 p-1 bg-red-500 text-white text-xs rounded pointer-events-none"
                style=move || {
                    let (x, y) = hover_line_text_pos.get();
                    let display = if hover_time.get().is_some() { "block" } else { "none" };
                    format!(
                        "position: fixed; left: {x}px; top: {y}px; transform: translate(10px, 10px); display: {display};",
                    )
                }
            >
                {move || hover_time.get().map(format_time)}
            </div>

            // Event tooltip
            <div
                class="absolute z-10 p-2 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded shadow-lg pointer-events-none"
                style=move || {
                    let (x, y) = tooltip_pos.get();
                    let display = if tooltip_visible.get() { "block" } else { "none" };
                    format!(
                        "position: fixed; left: {x}px; top: {y}px; transform: translate(10px, 10px); display: {display};",
                    )
                }
            >
                {move || {
                    hovered_event.get().map(|event| {
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
                            </div>
                        }
                    })
                }}
            </div>
        </div>
    }
}
