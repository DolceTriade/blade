use leptos::prelude::*;
use trace_event_parser::{BazelTrace, Event};

const TRACE_NAME_WIDTH: f64 = 200.0;
const ROW_HEIGHT: f64 = 30.0;
const EVENT_HEIGHT: f64 = 20.0;
const V_PADDING: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 20.0;

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
    #[prop(into)] zoom: Signal<f64>,
    #[prop(default = 1000)] width: u32,
    #[prop(default = 800)] height: u32,
) -> impl IntoView {
    // Sort traces by pid and tid to ensure deterministic order, mitigating non-determinism from
    // HashMap iteration in the trace parser.
    bazel_trace.traces.sort_by(|a, b| a.pid.cmp(&b.pid).then(a.tid.cmp(&b.tid)));

    let max_end_time = bazel_trace
        .traces
        .iter()
        .flat_map(|trace| {
            trace
                .events
                .iter()
                .map(|event| event.start + event.duration.unwrap_or(1))
        })
        .max()
        .unwrap_or(0) as f64;

    let layouts: Vec<_> = bazel_trace
        .traces
        .iter()
        .map(|trace| calculate_layout(&trace.events))
        .collect();

    let total_height = layouts
        .iter()
        .map(|(_, num_rows)| *num_rows as f64 * ROW_HEIGHT)
        .sum::<f64>()
        + X_AXIS_HEIGHT;

    let scale = move || zoom.get();
    let timeline_width = move || max_end_time * scale();

    view! {
        <div style=format!("overflow: auto; width: {}px; height: {}px; border: 1px solid #ccc;", width, height)>
            <svg
                class="bazel-trace-chart"
                xmlns="http://www.w3.org/2000/svg"
                width=move || TRACE_NAME_WIDTH + timeline_width()
                height=total_height
                viewBox=move || format!("0 0 {} {}", TRACE_NAME_WIDTH + timeline_width(), total_height)
            >
                // X-Axis
                <g class="x-axis" transform=format!("translate({}, {})", TRACE_NAME_WIDTH, X_AXIS_HEIGHT)>
                    <line x1="0" y1="0" x2=timeline_width y2="0" stroke="black" />
                    {(0..=10).map(|i| {
                        let time = (max_end_time / 10.0) * i as f64;
                        let x = time * scale();
                        view! {
                            <g>
                                <line x1=x y1="-5" x2=x y2="0" stroke="black" />
                                <text x=x y="-8" text-anchor="middle" font-size="10">{format!("{:.1}ms", time / 1000.0)}</text>
                            </g>
                        }
                    }).collect_view()}
                </g>

                // Traces
                <g class="traces" transform=format!("translate(0, {})", X_AXIS_HEIGHT)>
                    <g>
                        {
                            let trace_y_offsets: Vec<f64> = layouts
                                .iter()
                                .scan(0.0, |state, (_, num_rows)| {
                                    let current_y = *state;
                                    *state += *num_rows as f64 * ROW_HEIGHT;
                                    Some(current_y)
                                })
                                .collect();

                            bazel_trace.traces.into_iter()
                                .zip(layouts.into_iter())
                                .zip(trace_y_offsets.into_iter())
                                .map(|((trace, (positioned_events, num_rows)), current_y)| {
                                    let trace_height = num_rows as f64 * ROW_HEIGHT;

                                    view! {
                                        <g class="trace-group" transform=format!("translate(0, {})", current_y)>
                                            // Trace Border
                                            <rect x="0" y="0" width=move || TRACE_NAME_WIDTH + timeline_width() height=trace_height fill="none" stroke="#eee" />

                                            // Trace Name
                                            <text x="10" y=trace_height / 2.0 dominant-baseline="middle" font-size="12">{trace.name.clone()}</text>

                                            // Timeline
                                            <g class="timeline" transform=format!("translate({}, 0)", TRACE_NAME_WIDTH)>
                                                <For
                                                    each=move || positioned_events.clone()
                                                    key=|p_event| p_event.id
                                                    children=move |p_event| {
                                                        let event = p_event.event;
                                                        let start = event.start as f64 * scale();
                                                        let duration = event.duration.unwrap_or(1) as f64 * scale();
                                                        let y = p_event.row as f64 * ROW_HEIGHT + V_PADDING;
                                                        let color = color_for_category(&event.category);

                                                        view! {
                                                            <rect
                                                                x=start
                                                                y=y
                                                                width=duration
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
                                }).collect_view()
                        }
                    </g>
                </g>
            </svg>
        </div>
    }
}

fn show_tooltip(details: &str) {
    tracing::info!("Tooltip: {}", details);
}

fn hide_tooltip() {
    tracing::info!("Tooltip hidden");
}
