use leptos::{either::Either, prelude::*};
use state::TestHistory;

use crate::components::dashboard::{duration_chart::DurationChart, pass_fail_chart::PassFailChart};

#[allow(non_snake_case)]
#[component]
pub fn HistoryGraphs(history: TestHistory) -> impl IntoView {
    if history.history.is_empty() {
        return Either::Left(
            view! { <p class="text-gray-500 mt-8 text-center">"No history found for this test."</p> },
        );
    }

    Either::Right(view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mt-8">
            <div class="bg-white dark:bg-gray-700 p-6 rounded-lg shadow-lg">
                <h2 class="text-xl font-semibold mb-4">"Pass/Fail History"</h2>
                <PassFailChart history=history.clone() />
            </div>
            <div class="bg-white dark:bg-gray-700 p-6 rounded-lg shadow-lg">
                <h2 class="text-xl font-semibold mb-4">"Duration History (ms)"</h2>
                <DurationChart history=history />
            </div>
        </div>
    })
}
