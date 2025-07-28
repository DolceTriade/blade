use leptos::{html, prelude::*};
use trace_event_parser::{BazelTrace, Counter, Event, TimeSeriesDataPoint};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const TRACE_NAME_WIDTH: f64 = 200.0;
const ROW_HEIGHT: f64 = 30.0;
const EVENT_HEIGHT: f64 = 20.0;
const V_PADDING: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 30.0;
const COUNTER_CHART_HEIGHT: f64 = 50.0;
const COUNTER_CHART_TOP_MARGIN: f64 = 10.0;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct PositionedEvent {
    id: String,
    event: Event,
    row: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct RenderedCounter {
    name: String,
    y_offset: f64,
    points: Vec<(f64, f64)>, // (x, y) coordinates
    min_val: f64,
    max_val: f64,
    time_series: Vec<TimeSeriesDataPoint>,
}

#[derive(Clone)]
struct SpatialIndex {
    events: Vec<PositionedEvent>,
    counters: Vec<RenderedCounter>,
}

impl SpatialIndex {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            counters: Vec::new(),
        }
    }

    fn find_event_at(&self, x: f64, y: f64) -> Option<&Event> {
        for event in &self.events {
            if x >= event.x && x <= event.x + event.width && y >= event.y && y <= event.y + event.height {
                return Some(&event.event);
            }
        }
        None
    }

    fn find_counter_at(&self, x: f64, y: f64, zoom: f64, min_start_time: i64) -> Option<(String, f64)> {
        for counter in &self.counters {
            if y >= counter.y_offset && y <= counter.y_offset + COUNTER_CHART_HEIGHT {
                let timeline_x = x - TRACE_NAME_WIDTH;
                if timeline_x < 0.0 {
                    continue;
                }
                let time_us = (timeline_x / zoom) + min_start_time as f64;

                let value = match counter.time_series.binary_search_by(|p| {
                    (p.timestamp as f64).total_cmp(&time_us)
                }) {
                    Ok(i) => counter.time_series[i].value,
                    Err(i) => {
                        if i == 0 {
                            0.0
                        } else if i >= counter.time_series.len() {
                            counter.time_series.last().unwrap().value
                        } else {
                            counter.time_series[i - 1].value
                        }
                    }
                };
                return Some((counter.name.clone(), value));
            }
        }
        None
    }
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
                    id: format!("{trace_index}-{i}"),
                    event: event.clone(),
                    row: j,
                    x: 0.0, // Will be calculated later
                    y: 0.0, // Will be calculated later
                    width: 0.0, // Will be calculated later
                    height: EVENT_HEIGHT,
                    color: color_for_category(&event.category),
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
                event: event.clone(),
                row: new_row,
                x: 0.0, // Will be calculated later
                y: 0.0, // Will be calculated later
                width: 0.0, // Will be calculated later
                height: EVENT_HEIGHT,
                color: color_for_category(&event.category),
            });
            row_ends.push(end_time);
        }
    }

    (positioned_events, row_ends.len().max(1))
}

fn contrasting_text_color(hex_color: &str) -> &'static str {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return "#000000";
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;

    if luminance > 0.5 {
        "#000000"
    } else {
        "#FFFFFF"
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

#[derive(Clone)]
struct CanvasRendererState {
    spatial_index: SpatialIndex,
    canvas_width: f64,
    canvas_height: f64,
    zoom: f64,
    min_start_time: i64,
    traces_height: f64,
    counters_height: f64,
}

#[derive(Clone)]
struct CanvasRenderer {
    canvas_ref: NodeRef<html::Canvas>,
    state: CanvasRendererState,
}

impl CanvasRenderer {
    fn new(canvas_ref: NodeRef<html::Canvas>) -> Self {
        Self {
            canvas_ref,
            state: CanvasRendererState {
                spatial_index: SpatialIndex::new(),
                canvas_width: 0.0,
                canvas_height: 0.0,
                zoom: 1.0,
                min_start_time: 0,
                traces_height: 0.0,
                counters_height: 0.0,
            },
        }
    }

    fn get_canvas_and_ctx(&self) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), String> {
        let canvas = self.canvas_ref.get().ok_or("Canvas not available")?;
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2d context")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;
        Ok((canvas, ctx))
    }

    fn update_viewport(&mut self, width: f64, height: f64, zoom: f64) -> Result<(), String> {
        self.state.canvas_width = width;
        self.state.canvas_height = height;
        self.state.zoom = zoom;

        let (canvas, _) = self.get_canvas_and_ctx()?;

        // Update canvas size
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        // Set canvas style size to prevent blurring
        let element: &web_sys::Element = canvas.as_ref();
        if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
            let style = html_element.style();
            let _ = style.set_property("width", &format!("{width}px"));
            let _ = style.set_property("height", &format!("{height}px"));
        }

        Ok(())
    }

    fn clear(&self) -> Result<(), String> {
        let (_, ctx) = self.get_canvas_and_ctx()?;
        ctx.clear_rect(0.0, 0.0, self.state.canvas_width, self.state.canvas_height);
        Ok(())
    }

    fn render_events(&mut self, layouts: &[(Vec<PositionedEvent>, usize)], trace_y_offsets: &[f64]) -> Result<(), String> {
        let (_, ctx) = self.get_canvas_and_ctx()?;
        self.state.spatial_index.events.clear();

        for ((positioned_events, _), &trace_y_offset) in layouts.iter().zip(trace_y_offsets.iter()) {
            for positioned_event in positioned_events {
                let normalized_start = (positioned_event.event.start - self.state.min_start_time) as f64;
                let event_x = TRACE_NAME_WIDTH + (normalized_start * self.state.zoom);
                let event_width = ((positioned_event.event.duration.unwrap_or(1) as f64) * self.state.zoom).max(1.0);
                let event_y = X_AXIS_HEIGHT + self.state.counters_height + COUNTER_CHART_TOP_MARGIN + trace_y_offset + (positioned_event.row as f64 * ROW_HEIGHT) + V_PADDING;

                // Skip events outside viewport (viewport culling)
                if event_x + event_width < 0.0 || event_x > self.state.canvas_width {
                    continue;
                }

                // Render the event rectangle
                ctx.set_fill_style_str(&positioned_event.color);
                ctx.fill_rect(event_x, event_y, event_width, EVENT_HEIGHT);

                // Render text if event is wide enough
                if event_width > 30.0 {
                    let text_color = contrasting_text_color(&positioned_event.color);
                    ctx.set_fill_style_str(text_color);
                    ctx.set_font("12px sans-serif");
                    let _ = ctx.fill_text(&positioned_event.event.name, event_x + 5.0, event_y + EVENT_HEIGHT / 2.0 + 4.0);
                }

                // Add to spatial index
                let mut indexed_event = positioned_event.clone();
                indexed_event.x = event_x;
                indexed_event.y = event_y;
                indexed_event.width = event_width;
                self.state.spatial_index.events.push(indexed_event);
            }
        }
        Ok(())
    }

    fn render_counters(&mut self, counters: &[Counter]) -> Result<(), String> {
        let (_, ctx) = self.get_canvas_and_ctx()?;
        self.state.spatial_index.counters.clear();

        for (i, counter) in counters.iter().enumerate() {
            let y_offset = X_AXIS_HEIGHT + COUNTER_CHART_TOP_MARGIN + (i as f64 * COUNTER_CHART_HEIGHT);

            let (min_val, max_val) = counter.time_series.iter().fold(
                (f64::MAX, f64::MIN),
                |(min, max), point| (min.min(point.value), max.max(point.value))
            );

            if counter.time_series.is_empty() {
                continue;
            }

            // Build path points
            let mut points = Vec::new();
            let first_point = &counter.time_series[0];
            let first_x = TRACE_NAME_WIDTH + ((first_point.timestamp - self.state.min_start_time) as f64 * self.state.zoom);
            let first_y = y_offset + if max_val > min_val {
                COUNTER_CHART_HEIGHT - ((first_point.value - min_val) / (max_val - min_val)) * COUNTER_CHART_HEIGHT
            } else {
                COUNTER_CHART_HEIGHT / 2.0
            };

            // Start path at bottom
            ctx.begin_path();
            ctx.move_to(first_x, y_offset + COUNTER_CHART_HEIGHT);
            ctx.line_to(first_x, first_y);
            points.push((first_x, first_y));

            // Add step path points
            for j in 1..counter.time_series.len() {
                let prev_point = &counter.time_series[j - 1];
                let curr_point = &counter.time_series[j];

                let prev_y = y_offset + if max_val > min_val {
                    COUNTER_CHART_HEIGHT - ((prev_point.value - min_val) / (max_val - min_val)) * COUNTER_CHART_HEIGHT
                } else {
                    COUNTER_CHART_HEIGHT / 2.0
                };

                let curr_x = TRACE_NAME_WIDTH + ((curr_point.timestamp - self.state.min_start_time) as f64 * self.state.zoom);
                let curr_y = y_offset + if max_val > min_val {
                    COUNTER_CHART_HEIGHT - ((curr_point.value - min_val) / (max_val - min_val)) * COUNTER_CHART_HEIGHT
                } else {
                    COUNTER_CHART_HEIGHT / 2.0
                };

                ctx.line_to(curr_x, prev_y);
                ctx.line_to(curr_x, curr_y);
                points.push((curr_x, curr_y));
            }

            // Close path at bottom
            let last_x = TRACE_NAME_WIDTH + ((counter.time_series.last().unwrap().timestamp - self.state.min_start_time) as f64 * self.state.zoom);
            ctx.line_to(last_x, y_offset + COUNTER_CHART_HEIGHT);
            ctx.close_path();

            // Fill with color
            let color = color_for_category(&counter.name);
            ctx.set_fill_style_str(&color);
            ctx.set_global_alpha(0.5);
            ctx.fill();
            ctx.set_global_alpha(1.0);

            // Stroke outline
            ctx.set_stroke_style_str("#64748b"); // slate-500
            ctx.set_line_width(1.0);
            ctx.stroke();

            // Add to spatial index
            self.state.spatial_index.counters.push(RenderedCounter {
                name: counter.name.clone(),
                y_offset,
                points,
                min_val,
                max_val,
                time_series: counter.time_series.clone(),
            });
        }
        Ok(())
    }

    fn find_event_at(&self, x: f64, y: f64) -> Option<&Event> {
        self.state.spatial_index.find_event_at(x, y)
    }

    fn find_counter_at(&self, x: f64, y: f64) -> Option<(String, f64)> {
        self.state.spatial_index.find_counter_at(x, y, self.state.zoom, self.state.min_start_time)
    }
}

#[allow(non_snake_case)]
#[component]
pub fn BazelTraceChartCanvasHybrid(
    mut bazel_trace: BazelTrace,
    #[prop(default = 800)] height: u32,
) -> impl IntoView {
    // Sort traces and counters for deterministic order (same as original)
    bazel_trace.traces.sort_by(|a, b| a.pid.cmp(&b.pid).then(a.tid.cmp(&b.tid)));
    bazel_trace.counters.sort_by(|a, b| a.name.cmp(&b.name));

    // Calculate time bounds (same as original)
    let (mut min_start_time, mut max_end_time) = bazel_trace
        .traces
        .iter()
        .flat_map(|trace| &trace.events)
        .fold((i64::MAX, 0), |(min_s, max_e), event| {
            (
                min_s.min(event.start),
                max_e.max(event.start + event.duration.unwrap_or(1)),
            )
        });

    let (min_counter_time, max_counter_time) =
        bazel_trace.counters.iter().flat_map(|c| &c.time_series).fold(
            (i64::MAX, 0),
            |(min_t, max_t), point| (min_t.min(point.timestamp), max_t.max(point.timestamp)),
        );

    if min_counter_time != i64::MAX {
        min_start_time = min_start_time.min(min_counter_time);
    }
    max_end_time = max_end_time.max(max_counter_time);

    let min_start_time = if min_start_time == i64::MAX { 0 } else { min_start_time };
    let duration = (max_end_time - min_start_time).max(1) as f64;

    // Calculate layouts (same as original)
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
        l.iter().map(|(_, num_rows)| *num_rows as f64 * ROW_HEIGHT).sum::<f64>()
    });
    let total_height = traces_height + counters_height + X_AXIS_HEIGHT + COUNTER_CHART_TOP_MARGIN;

    let bazel_trace = StoredValue::new(bazel_trace);

    // Zoom and interaction state
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

    // Refs
    let container_ref = NodeRef::<html::Div>::new();
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let renderer = RwSignal::new(None::<CanvasRenderer>);

    // Initialize zoom based on container width (same as original)
    Effect::new(move |_| {
        if let Some(container) = container_ref.get() {
            let container_width = container.client_width() as f64;

            if container_width > 0.0 {
                let new_initial_zoom = if duration > 0.0 {
                    (container_width - TRACE_NAME_WIDTH) / duration
                } else {
                    1.0
                };
                initial_zoom.set(new_initial_zoom);
                set_zoom.set(new_initial_zoom);
            } else {
                // Defer measurement using requestAnimationFrame (same as original)
                let container_clone = container.clone();
                let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    let container_width = container_clone.client_width() as f64;
                    if container_width > 0.0 {
                        let new_initial_zoom = if duration > 0.0 {
                            (container_width - TRACE_NAME_WIDTH) / duration
                        } else {
                            1.0
                        };
                        initial_zoom.set(new_initial_zoom);
                        set_zoom.set(new_initial_zoom);
                    }
                }) as Box<dyn FnMut()>);

                if let Some(window) = web_sys::window() {
                    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
                }
                callback.forget();
            }
        }
    });

    // Initialize canvas renderer
    Effect::new(move |_| {
        if let Some(_canvas) = canvas_ref.get() {
            let mut canvas_renderer = CanvasRenderer::new(canvas_ref);
            canvas_renderer.state.min_start_time = min_start_time;
            canvas_renderer.state.counters_height = counters_height;
            canvas_renderer.state.traces_height = traces_height;
            renderer.set(Some(canvas_renderer));
        }
    });

    let timeline_width = Signal::derive(move || duration * zoom.get());

    // Calculate axis ticks (same as original)
    let x_axis_ticks = Memo::new(move |_| {
        let timeline_w = timeline_width.get();
        if timeline_w <= 0.0 || duration <= 0.0 {
            return Vec::new();
        }

        let (unit_label, divisor) = if duration >= 1_000_000.0 {
            ("s", 1_000_000.0)
        } else if duration >= 1_000.0 {
            ("ms", 1_000.0)
        } else {
            ("µs", 1.0)
        };

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
                    format!("{label_val:.2}{unit_label}")
                };
                ticks.push((x, display_label, current_tick));
            }
            current_tick += nice_tick_interval;
        }
        ticks
    });

    // Calculate trace Y offsets (same as original)
    let trace_y_offsets = StoredValue::new(
        layouts.with_value(|l| {
            l.iter()
                .scan(0.0, |state, (_, num_rows)| {
                    let current_y = *state;
                    *state += *num_rows as f64 * ROW_HEIGHT;
                    Some(current_y)
                })
                .collect::<Vec<f64>>()
        })
    );

    // Main render effect
    Effect::new(move |_| {
        let _ = zoom.get(); // Track zoom changes

        if let Some(container) = container_ref.get() {
            let container_width = container.client_width() as f64;

            if container_width > 0.0 {
                renderer.update(|r| {
                    if let Some(canvas_renderer) = r
                        && let Ok(()) = canvas_renderer.update_viewport(
                            TRACE_NAME_WIDTH + timeline_width.get(),
                            total_height,
                            zoom.get()
                        ) {
                        let _ = canvas_renderer.clear();

                        // Render counters
                        bazel_trace.with_value(|bt| {
                            let _ = canvas_renderer.render_counters(&bt.counters);
                        });

                        // Render events
                        layouts.with_value(|l| {
                            trace_y_offsets.with_value(|offsets| {
                                let _ = canvas_renderer.render_events(l, offsets);
                            });
                        });
                    }
                });
            }
        }
    });

    // Mouse interaction handlers
    let on_canvas_mousemove = move |ev: web_sys::MouseEvent| {
        if let Some(container) = container_ref.get() {
            let rect = container.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left() + container.scroll_left() as f64;
            let y = ev.client_y() as f64 - rect.top() + container.scroll_top() as f64;

            // Update global hover time (same as original)
            if x >= TRACE_NAME_WIDTH {
                let timeline_x = x - TRACE_NAME_WIDTH;
                let time_us = (timeline_x / zoom.get()) + min_start_time as f64;
                hover_time.set(Some(time_us));
                hover_line_text_pos.set((ev.client_x() as f64, rect.top()));
            } else {
                hover_time.set(None);
            }

            // Check for event hover
            renderer.with(|r| {
                if let Some(canvas_renderer) = r {
                    if let Some(event) = canvas_renderer.find_event_at(x, y) {
                        hovered_event.set(Some(event.clone()));
                        tooltip_pos.set((ev.client_x() as f64, ev.client_y() as f64));
                        tooltip_visible.set(true);
                        counter_tooltip_visible.set(false);
                    } else if let Some((name, value)) = canvas_renderer.find_counter_at(x, y) {
                        hovered_counter_info.set(Some((name, value)));
                        counter_tooltip_pos.set((ev.client_x() as f64, ev.client_y() as f64));
                        counter_tooltip_visible.set(true);
                        tooltip_visible.set(false);
                    } else {
                        tooltip_visible.set(false);
                        counter_tooltip_visible.set(false);
                    }
                }
            });
        }
    };

    let on_canvas_mouseleave = move |_| {
        hover_time.set(None);
        tooltip_visible.set(false);
        counter_tooltip_visible.set(false);
    };

    view! {
        <div class="relative">
            <div>
                // Zoom controls (same as original)
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
                    class="rounded overflow-auto max-w-full w-full relative"
                    on:mouseleave=on_canvas_mouseleave
                >
                    // Canvas for rendering
                    <canvas
                        node_ref=canvas_ref
                        class="absolute top-0 left-0"
                        on:mousemove=on_canvas_mousemove
                        style="cursor: crosshair;"
                    />

                    // SVG overlay for labels and UI elements
                    <svg
                        class="absolute top-0 left-0 pointer-events-none"
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
                        // X-Axis (same as original)
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
                                key=move |(_, _, tick_val)| format!("{}-{}", zoom.get(), tick_val.to_bits())
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

                        // Counter Names Sidebar (same as original)
                        <g
                            class="counter-names"
                            transform=format!(
                                "translate(0, {})",
                                X_AXIS_HEIGHT + COUNTER_CHART_TOP_MARGIN
                            )
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
                                    bazel_trace
                                        .with_value(|bt| bt.counters.clone())
                                        .into_iter()
                                        .enumerate()
                                }
                                key=|(_, counter)| counter.name.clone()
                                children=move |(i, counter)| {
                                    let y = i as f64 * COUNTER_CHART_HEIGHT;
                                    let (_, max_val) = counter
                                        .time_series
                                        .iter()
                                        .fold(
                                            (f64::MAX, f64::MIN),
                                            |(min, max), point| {
                                                (min.min(point.value), max.max(point.value))
                                            },
                                        );

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
                                                {format!("{max_val:.2}")}
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

                        // Trace Names Sidebar (same as original)
                        <g
                            class="trace-names"
                            transform=format!(
                                "translate(0, {})",
                                X_AXIS_HEIGHT + counters_height + COUNTER_CHART_TOP_MARGIN
                            )
                        >
                            <rect
                                x="0"
                                y="0"
                                width=TRACE_NAME_WIDTH
                                height=traces_height
                                class="fill-slate-50 dark:fill-slate-800"
                            />
                            {
                                bazel_trace
                                    .with_value(|bt| {
                                        bt.traces
                                            .iter()
                                            .zip(layouts.with_value(|l| l.clone()).into_iter())
                                            .zip(trace_y_offsets.with_value(|offsets| offsets.clone()).into_iter())
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

                        // Hover time line (same as original)
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
                                            attr:stroke-dasharray="4"
                                        />
                                    </g>
                                }
                            }}
                        </Show>
                    </svg>
                </div>
            </div>

            // Tooltips (same as original)
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
                        "position: fixed; left: {x}px; top: {y}px; transform: translate(10px, 10px); display: {display};",
                    )
                }
            >
                {move || hover_time.get().map(format_time)}
            </div>

            <div
                class="absolute z-10 p-2 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded shadow-lg pointer-events-none"
                style=move || {
                    let (x, y) = counter_tooltip_pos.get();
                    let display = if counter_tooltip_visible.get() { "block" } else { "none" };
                    format!(
                        "position: fixed; left: {x}px; top: {y}px; transform: translate(10px, 10px); display: {display};",
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
                                        {format!("{value:.2}")}
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
                        "position: fixed; left: {x}px; top: {y}px; transform: translate(10px, 10px); display: {display};",
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
                                                    {serde_json::to_string(&args).unwrap_or_default()}
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
