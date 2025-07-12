use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use state::{Status, TestHistory};

use crate::summaryheader::format_time;

#[allow(non_snake_case)]
#[component]
pub fn TestHistoryTable(history: TestHistory) -> impl IntoView {
    view! {
        <div class="mt-8">
            <h2 class="text-2xl font-bold mb-4">"Raw Test Results"</h2>
            <div class="overflow-x-auto">
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
