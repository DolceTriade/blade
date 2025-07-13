use std::collections::HashMap;

use leptos::prelude::*;

use crate::charts::{linechart::LineChart, piechart::PieChart};

#[derive(Debug, Clone, PartialEq)]
struct TestInsightsData {
    duration_distribution: Vec<(String, f64)>,
    status_distribution: Vec<(String, f64)>,
    failure_types: Vec<(String, f64)>,
    performance_by_name_length: Vec<(f64, f64)>,
    alphabetical_performance: Vec<(String, f64)>,
}

fn analyze_test_cases(cases: &[junit_parser::TestCase]) -> TestInsightsData {
    let mut duration_buckets = HashMap::new();
    let mut status_counts = HashMap::new();
    let mut failure_types = HashMap::new();
    let mut name_length_durations: Vec<(usize, f64)> = Vec::new();
    let mut alphabetical_durations: Vec<(String, f64)> = Vec::new();

    for case in cases {
        // Duration distribution
        let duration_bucket = match case.time {
            t if t < 1.0 => "Fast (<1s)",
            t if t < 5.0 => "Medium (1-5s)",
            t if t < 10.0 => "Slow (5-10s)",
            _ => "Very Slow (>10s)",
        };
        *duration_buckets
            .entry(duration_bucket.to_string())
            .or_insert(0.0) += 1.0;

        // Status distribution
        let status = match case.status {
            junit_parser::TestStatus::Success => "Passing",
            junit_parser::TestStatus::Error(_) => "Error",
            junit_parser::TestStatus::Failure(_) => "Failure",
            junit_parser::TestStatus::Skipped(_) => "Skipped",
        };
        *status_counts.entry(status.to_string()).or_insert(0.0) += 1.0;

        // Failure type analysis
        match &case.status {
            junit_parser::TestStatus::Error(e) => {
                let error_type = if e.error_type.is_empty() {
                    "Unknown Error"
                } else {
                    &e.error_type
                };
                *failure_types.entry(error_type.to_string()).or_insert(0.0) += 1.0;
            },
            junit_parser::TestStatus::Failure(f) => {
                let failure_type = if f.failure_type.is_empty() {
                    "Unknown Failure"
                } else {
                    &f.failure_type
                };
                *failure_types.entry(failure_type.to_string()).or_insert(0.0) += 1.0;
            },
            _ => {},
        }

        // Name length vs duration
        name_length_durations.push((case.name.len(), case.time));

        // Alphabetical performance
        alphabetical_durations.push((case.name.clone(), case.time));
    }

    // Sort alphabetical performance
    alphabetical_durations.sort_by(|a, b| a.0.cmp(&b.0));

    // Calculate average duration by name length buckets
    let mut length_buckets: HashMap<usize, Vec<f64>> = HashMap::new();
    for (length, duration) in name_length_durations {
        let bucket = (length / 10) * 10; // Group by 10s: 0-9, 10-19, etc.
        length_buckets.entry(bucket).or_default().push(duration);
    }

    let performance_by_name_length: Vec<(f64, f64)> = length_buckets
        .iter()
        .map(|(bucket, durations)| {
            let avg = durations.iter().sum::<f64>() / durations.len() as f64;
            (*bucket as f64, avg)
        })
        .collect();

    TestInsightsData {
        duration_distribution: duration_buckets.into_iter().collect(),
        status_distribution: status_counts.into_iter().collect(),
        failure_types: failure_types.into_iter().collect(),
        performance_by_name_length,
        alphabetical_performance: alphabetical_durations,
    }
}

#[allow(non_snake_case)]
#[component]
pub fn TestInsights() -> impl IntoView {
    let xml = expect_context::<LocalResource<Option<junit_parser::TestSuites>>>();

    let insights = Memo::new(move |_| {
        xml.read()
            .as_ref()
            .and_then(|sw| sw.as_ref())
            .and_then(|ts| ts.suites.first())
            .map(|suite| analyze_test_cases(&suite.cases))
    });

    view! {
        <Suspense fallback=move || {
            view! { <div class="p-4">Loading insights...</div> }
        }>
            {move || {
                insights.with(|insights_opt| {
                    match insights_opt.as_ref() {
                        Some(insights) => {
                            view! {
                                <div class="p-6 space-y-8">
                                    <h2 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">
                                        "Test Insights"
                                    </h2>

                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                                        // Test Status Distribution
                                        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                            <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                "Test Status Distribution"
                                            </h3>
                                            <PieChart
                                                data=insights.status_distribution.clone()
                                                size=250
                                                value_accessor=|item: &(String, f64)| item.1
                                                label_accessor=|item: &(String, f64)| item.0.clone()
                                                color_accessor=|item: &(String, f64)| {
                                                    match item.0.as_str() {
                                                        "Passing" => "#10b981".to_string(),
                                                        "Error" => "#f97316".to_string(),
                                                        "Failure" => "#ef4444".to_string(),
                                                        "Skipped" => "#6b7280".to_string(),
                                                        _ => "#9ca3af".to_string(),
                                                    }
                                                }
                                                tooltip_content_accessor=|item: &(String, f64)| {
                                                    format!("{}: {}", item.0, item.1 as i32)
                                                }
                                            />
                                        </div>

                                        // Duration Distribution
                                        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                            <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                "Test Duration Distribution"
                                            </h3>
                                            <PieChart
                                                data=insights.duration_distribution.clone()
                                                size=250
                                                value_accessor=|item: &(String, f64)| item.1
                                                label_accessor=|item: &(String, f64)| item.0.clone()
                                                color_accessor=|item: &(String, f64)| {
                                                    match item.0.as_str() {
                                                        "Fast (<1s)" => "#10b981".to_string(),
                                                        "Medium (1-5s)" => "#3b82f6".to_string(),
                                                        "Slow (5-10s)" => "#f59e0b".to_string(),
                                                        "Very Slow (>10s)" => "#ef4444".to_string(),
                                                        _ => "#9ca3af".to_string(),
                                                    }
                                                }
                                                tooltip_content_accessor=|item: &(String, f64)| {
                                                    format!("{}: {} tests", item.0, item.1 as i32)
                                                }
                                            />
                                        </div>

                                        // Failure Types (only show if there are failures)
                                        {(!insights.failure_types.is_empty()).then(|| {
                                            view! {
                                                <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                                    <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                        "Failure Type Analysis"
                                                    </h3>
                                                    <PieChart
                                                        data=insights.failure_types.clone()
                                                        size=250
                                                        value_accessor=|item: &(String, f64)| item.1
                                                        label_accessor=|item: &(String, f64)| item.0.clone()
                                                        color_accessor=|item: &(String, f64)| {
                                                            let colors = ["#ef4444", "#f97316", "#f59e0b", "#eab308", "#84cc16", "#22c55e"];
                                                            let index = item.0.len() % colors.len();
                                                            colors[index].to_string()
                                                        }
                                                        tooltip_content_accessor=|item: &(String, f64)| {
                                                            format!("{}: {} failures", item.0, item.1 as i32)
                                                        }
                                                    />
                                                </div>
                                            }
                                        })}

                                        // Performance by Name Length
                                        {(!insights.performance_by_name_length.is_empty()).then(|| {
                                            view! {
                                                <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                                    <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                        "Performance by Test Name Length"
                                                    </h3>
                                                    <LineChart
                                                        data=insights.performance_by_name_length.clone()
                                                        width=400
                                                        height=250
                                                        x_accessor=|item: &(f64, f64)| item.0
                                                        y_accessor=|item: &(f64, f64)| item.1
                                                        line_color="#3b82f6"
                                                        point_color_accessor=|_: &(f64, f64)| "#3b82f6".to_string()
                                                        tooltip_content_accessor=|item: &(f64, f64)| {
                                                            format!("Length: {}, Avg Duration: {:.2}s", item.0 as i32, item.1)
                                                        }
                                                        x_axis_label="Name Length (chars)"
                                                        y_axis_label="Avg Duration (s)"
                                                    />
                                                </div>
                                            }
                                        })}
                                    </div>

                                    // Alphabetical Performance (full width)
                                    {(!insights.alphabetical_performance.is_empty() && insights.alphabetical_performance.len() > 5).then(|| {
                                        // Sample every nth test to avoid overcrowding
                                        let step = (insights.alphabetical_performance.len() / 20).max(1);
                                        let sampled_data: Vec<(String, f64)> = insights.alphabetical_performance
                                            .iter()
                                            .step_by(step)
                                            .take(20)
                                            .map(|(name, duration)| {
                                                // Truncate long test names for display
                                                let display_name = if name.len() > 30 {
                                                    format!("{}...", &name[..27])
                                                } else {
                                                    name.clone()
                                                };
                                                (display_name, *duration)
                                            })
                                            .collect();

                                        view! {
                                            <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                                <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                    "Test Performance (Alphabetical Sample)"
                                                </h3>
                                                <div class="overflow-x-auto">
                                                    <LineChart
                                                        data=sampled_data
                                                        width=800
                                                        height=300
                                                        x_accessor=|item: &(String, f64)| item.0.len() as f64  // Use string length as x position for now
                                                        y_accessor=|item: &(String, f64)| item.1
                                                        line_color="#3b82f6"
                                                        point_color_accessor=|_: &(String, f64)| "#3b82f6".to_string()
                                                        tooltip_content_accessor=|item: &(String, f64)| {
                                                            format!("{}: {:.2}s", item.0, item.1)
                                                        }
                                                        x_axis_label="Test Name"
                                                        y_axis_label="Duration (s)"
                                                    />
                                                </div>
                                            </div>
                                        }
                                    })}
                                </div>
                            }.into_any()
                        }
                        None => {
                            view! {
                                <div class="p-6 space-y-8">
                                    <div class="text-center text-gray-500 dark:text-gray-400">
                                        "No test data available for insights"
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}
