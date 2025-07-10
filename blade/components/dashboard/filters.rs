use leptos::{leptos_dom::helpers::event_target_value, prelude::*};
use state::{TestFilter, TestFilterItem, TestFilterOp};

#[allow(non_snake_case)]
#[component]
pub fn FilterControls(
    set_test_name: WriteSignal<String>,
    set_filters: WriteSignal<Vec<TestFilter>>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (metadata, set_metadata) = signal(String::new());

    let apply_filters = move |_| {
        set_test_name.set(name.get());
        let mut new_filters = Vec::new();
        if !metadata.get().is_empty()
            && let Some((key, value)) = metadata.get().split_once('=')
        {
            new_filters.push(TestFilter {
                op: TestFilterOp::Equals,
                invert: false,
                filter: TestFilterItem::Metadata {
                    key: key.to_string(),
                    value: value.to_string(),
                },
            });
        }
        set_filters.set(new_filters);
    };

    view! {
        <div class="bg-white dark:bg-gray-700 p-4 rounded-lg shadow-md mb-6">
            <div class="flex items-center space-x-4">
                <input
                    type="text"
                    placeholder="//test:name"
                    class="flex-grow p-2 bg-gray-50 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white"
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    prop:value=name
                />
                <input
                    type="text"
                    placeholder="branch=main"
                    class="p-2 bg-gray-50 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white"
                    on:input=move |ev| set_metadata.set(event_target_value(&ev))
                    prop:value=metadata
                />
                <button
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-md font-semibold transition-colors"
                    on:click=apply_filters
                >
                    "Search"
                </button>
            </div>
        </div>
    }
}
