#[cfg(feature = "ssr")]
use std::sync::Arc;

use components::dashboard::{
    filters::FilterControls,
    graphs::HistoryGraphs,
    test_history_table::TestHistoryTable,
    test_search::TestSearchInput,
};
use leptos::{either::Either, prelude::*};
use leptos_router::{hooks::use_query, params::Params};
use state::{TestFilter, TestHistory};

#[server]
pub async fn get_test_history(
    test_name: String,
    filters: Option<Vec<TestFilter>>,
) -> Result<TestHistory, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let mut db = global
        .db_manager
        .get()
        .map_err(crate::invocation::internal_err)?;
    db.get_test_history(&test_name, &filters.unwrap_or_default(), 50) // Limit to 50 results for now
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[derive(PartialEq, Params)]
pub(crate) struct DashboardParams {
    pub(crate) test_name: Option<String>,
}

#[allow(non_snake_case)]
#[component]
pub fn Dashboard() -> impl IntoView {
    // Read test_name from URL query parameter
    let params = use_query::<DashboardParams>();
    let initial_test_name = params.with_untracked(|p| {
        p.as_ref()
            .ok()
            .map(|params| params.test_name.clone().unwrap_or_default())
            .inspect(|s| tracing::info!("Initial test name: {}", s))
            .unwrap_or_default()
    });

    let (test_name, set_test_name) = signal(initial_test_name);
    let (filters, set_filters) = signal(Vec::<TestFilter>::new());

    let history_resource = Resource::new(
        move || (test_name.get(), filters.get()),
        |(test_name, filters)| async move {
            if test_name.is_empty() {
                return None;
            }
            get_test_history(test_name, Some(filters))
                .await
                .inspect_err(|e| {
                    tracing::warn!("Failed to get test history: {e:#?}");
                })
                .ok()
        },
    );

    view! {
        <div class="p-4 bg-white dark:bg-gray-800 text-gray-900 dark:text-white h-[calc(100vh-64px)] overflow-y-auto">
            <div class="container mx-auto">
                <h1 class="text-3xl font-bold mb-6">"Test History Dashboard"</h1>
                <TestSearchInput test_name=test_name set_test_name=set_test_name />
                <FilterControls set_filters=set_filters />
                <Suspense fallback=|| {
                    view! { <p class="text-gray-400">"Loading..."</p> }
                }>
                    {move || {
                        history_resource
                            .read()
                            .as_ref()
                            .map(|data| match data {
                                Some(history) => {
                                    Either::Right(
                                        view! {
                                            <HistoryGraphs history=history.clone() />
                                            <TestHistoryTable history=history.clone() />
                                        },
                                    )
                                }
                                None => {
                                    Either::Left(
                                        view! {
                                            <p class="text-gray-500 mt-8 text-center">
                                                "Enter a test label to see its history."
                                            </p>
                                        },
                                    )
                                }
                            })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
