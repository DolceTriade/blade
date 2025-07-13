use std::collections::HashMap;

use leptos::prelude::*;

use crate::charts::{linechart::LineChart, piechart::PieChart};

#[derive(Debug, Clone, PartialEq)]
struct TestInsightsData {
    pass_fail_distribution: Vec<(String, f64)>,
    duration_distribution: Vec<(String, f64)>, // (test_name, percentage)
    duration_mapping: HashMap<String, f64>, // (test_name, actual_duration) for tooltips
    test_performance: Vec<(String, f64)>, // (test_name, duration)
}

fn analyze_test_cases(cases: &[junit_parser::TestCase]) -> TestInsightsData {
    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut duration_buckets = HashMap::new();
    let mut duration_mapping = HashMap::new();
    let mut test_performance: Vec<(String, f64)> = Vec::new();

    // Handle empty case list
    if cases.is_empty() {
        return TestInsightsData {
            pass_fail_distribution: vec![
                ("Passed".to_string(), 0.0),
                ("Failed".to_string(), 0.0),
            ],
            duration_distribution: vec![],
            duration_mapping: HashMap::new(),
            test_performance: vec![],
        };
    }

    // Calculate total duration for proportional calculation
    let total_duration: f64 = cases.iter().map(|case| case.time.max(0.001)).sum(); // Minimum 1ms per test

    for case in cases {
        // Pass/Fail distribution
        match case.status {
            junit_parser::TestStatus::Success => pass_count += 1,
            _ => fail_count += 1,
        }

        // Ensure minimum duration of 1ms to avoid division issues
        let actual_duration = case.time.max(0.001);
        
        // Duration distribution - individual test durations as proportion of total
        let proportion = if total_duration > 0.0 {
            (actual_duration / total_duration) * 100.0
        } else {
            // Fallback: if somehow total is 0, distribute equally
            100.0 / cases.len() as f64
        };
        
        // Ensure proportion is at least 0.1% to be visible in pie chart
        let final_proportion = proportion.max(0.1);
        
        duration_buckets.insert(case.name.clone(), final_proportion);
        duration_mapping.insert(case.name.clone(), case.time); // Keep original time for display

        // Test performance (name -> duration)
        test_performance.push((case.name.clone(), case.time));
    }

    // Sort test performance by duration for better visualization
    test_performance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    TestInsightsData {
        pass_fail_distribution: vec![
            ("Passed".to_string(), pass_count as f64),
            ("Failed".to_string(), fail_count as f64),
        ],
        duration_distribution: duration_buckets.into_iter().collect(),
        duration_mapping,
        test_performance,
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
                                        // Pass/Fail Distribution
                                        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                            <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                "Test Results Distribution"
                                            </h3>
                                            <PieChart
                                                data=insights.pass_fail_distribution.clone()
                                                size=250
                                                value_accessor=|item: &(String, f64)| item.1
                                                label_accessor=|item: &(String, f64)| item.0.clone()
                                                color_accessor=|item: &(String, f64)| {
                                                    match item.0.as_str() {
                                                        "Passed" => "#10b981".to_string(),
                                                        "Failed" => "#ef4444".to_string(),
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
                                                "Test Duration Distribution (% of Total Time)"
                                            </h3>
                                            {
                                                // Create a combined data structure with both percentage and actual duration
                                                let combined_data: Vec<(String, f64, f64)> = insights.duration_distribution
                                                    .iter()
                                                    .map(|(name, percentage)| {
                                                        let actual_duration = insights.duration_mapping.get(name).unwrap_or(&0.0);
                                                        (name.clone(), *percentage, *actual_duration)
                                                    })
                                                    .collect();
                                                    
                                                view! {
                                                    <PieChart
                                                        data=combined_data
                                                        size=250
                                                        value_accessor=|item: &(String, f64, f64)| item.1
                                                        label_accessor=|item: &(String, f64, f64)| item.0.clone()
                                                        color_accessor=|item: &(String, f64, f64)| {
                                                            // Generate colors based on percentage of total time
                                                            match item.1 {
                                                                p if p < 10.0 => "#10b981".to_string(),   // Small portion - green
                                                                p if p < 25.0 => "#3b82f6".to_string(),   // Medium portion - blue
                                                                p if p < 50.0 => "#f59e0b".to_string(),   // Large portion - yellow
                                                                _ => "#ef4444".to_string(),               // Very large portion - red
                                                            }
                                                        }
                                                        tooltip_content_accessor=|item: &(String, f64, f64)| {
                                                            format!("{}: {:.1}% ({:.2}s)", item.0, item.1, item.2)
                                                        }
                                                    />
                                                }
                                            }
                                        </div>
                                    </div>

                                    // Test Performance Scatter Plot (full width)
                                    {(insights.test_performance.len() > 1).then(|| {
                                        // Sample every nth test to avoid overcrowding
                                        let step = (insights.test_performance.len() / 20).max(1);
                                        let sampled_data: Vec<(String, f64)> = insights.test_performance
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
                                                    "Test Performance (Duration by Test Name)"
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

                                    {(insights.test_performance.len() <= 1).then(|| {
                                        view! {
                                            <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
                                                <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                                                    "Test Performance"
                                                </h3>
                                                <p class="text-gray-500 text-center py-8">
                                                    "Not enough test data to display performance chart"
                                                </p>
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
