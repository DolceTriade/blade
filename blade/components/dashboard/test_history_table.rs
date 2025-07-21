use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use state::{Status, TestHistory};

use crate::summaryheader::format_time;

#[derive(Debug, Clone)]
struct RuntimeStats {
    min: f64,
    max: f64,
    avg: f64,
    std_dev: f64,
}

fn calculate_runtime_stats(history: &TestHistory) -> Option<RuntimeStats> {
    if history.history.is_empty() {
        return None;
    }

    let durations: Vec<f64> = history
        .history
        .iter()
        .map(|point| point.test.duration.as_secs_f64())
        .collect();

    let count = durations.len();
    let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let sum: f64 = durations.iter().sum();
    let avg = sum / count as f64;

    // Calculate standard deviation
    let variance: f64 = durations.iter().map(|&x| (x - avg).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    Some(RuntimeStats {
        min,
        max,
        avg,
        std_dev,
    })
}

#[allow(non_snake_case)]
#[component]
fn RuntimeStatsCard(stats: RuntimeStats) -> impl IntoView {
    view! {
        <div class="mb-6 bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
            <h3 class="text-lg font-semibold mb-3 text-gray-900 dark:text-white">
                "Runtime Statistics"
            </h3>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div class="text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
                        {format!("{:.3}s", stats.min)}
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">"Min Runtime"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold text-red-600 dark:text-red-400">
                        {format!("{:.3}s", stats.max)}
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">"Max Runtime"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-400">
                        {format!("{:.3}s", stats.avg)}
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">"Avg Runtime"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
                        {format!("{:.3}s", stats.std_dev)}
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">"Std Deviation"</div>
                </div>
            </div>
        </div>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn TestHistoryTable(history: TestHistory) -> impl IntoView {
    let stats = calculate_runtime_stats(&history);

    view! {
        <div class="mt-8">
            {stats.map(|s| view! { <RuntimeStatsCard stats=s /> })}
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-2xl font-bold">"Raw Test Results"</h2>
                <div class="text-sm text-gray-600 dark:text-gray-400">
                    {format!(
                        "Showing {} of {} results",
                        history.history.len(),
                        history.total_found,
                    )}
                    {if history.was_truncated {
                        Some(
                            view! {
                                <span class="ml-2 px-2 py-1 bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded-md text-xs">
                                    "Results truncated"
                                </span>
                            },
                        )
                    } else {
                        None
                    }}
                </div>
            </div>
            {if history.was_truncated {
                Some(
                    view! {
                        <div class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md">
                            <p class="text-sm text-yellow-800 dark:text-yellow-200">
                                "Too many results found. Consider narrowing your date range or adding more specific filters to see all results."
                            </p>
                        </div>
                    },
                )
            } else {
                None
            }} <div class="overflow-x-auto">
                <table class="min-w-full bg-white dark:bg-gray-700 rounded-lg shadow-md">
                    <thead>
                        <tr class="bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 uppercase text-sm leading-normal">
                            <th class="py-3 px-6 text-left">"Invocation ID"</th>
                            <th class="py-3 px-6 text-left">"Status"</th>
                            <th class="py-3 px-6 text-left">"Duration"</th>
                            <th class="py-3 px-6 text-left">"Date"</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700 dark:text-gray-300 text-sm font-light">
                        {history
                            .history
                            .into_iter()
                            .map(|point| {
                                let duration_secs = point.test.duration.as_secs_f64();
                                view! {
                                    <tr
                                        class="border-b border-gray-200 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-600 cursor-pointer"
                                        on:click=move |_| {
                                            let navigate = use_navigate();
                                            let url = format!("/invocation/{}", &point.invocation_id);
                                            navigate(&url, Default::default());
                                        }
                                    >
                                        <td class="py-3 px-6 text-left whitespace-nowrap">
                                            {point.invocation_id.clone()}
                                        </td>
                                        <td class="py-3 px-6 text-left">
                                            <span class=move || {
                                                let base_class = "px-2 inline-flex text-xs leading-5 font-semibold rounded-full";
                                                match point.test.status {
                                                    Status::Success => {
                                                        format!("{base_class} bg-green-100 text-green-800")
                                                    }
                                                    Status::Fail => {
                                                        format!("{base_class} bg-red-100 text-red-800")
                                                    }
                                                    Status::Skip => {
                                                        format!("{base_class} bg-yellow-100 text-yellow-800")
                                                    }
                                                    _ => format!("{base_class} bg-gray-100 text-gray-800"),
                                                }
                                            }>{point.test.status.to_string()}</span>
                                        </td>
                                        <td class="py-3 px-6 text-left">
                                            {format!("{duration_secs:.3} s")}
                                        </td>
                                        <td class="py-3 px-6 text-left">
                                            {format_time(&point.start)}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
