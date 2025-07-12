use leptos::{leptos_dom::helpers::event_target_value, prelude::*};
use shared::search_test_names;

#[allow(non_snake_case)]
#[component]
pub fn TestSearchInput(
    test_name: ReadSignal<String>,
    set_test_name: WriteSignal<String>,
) -> impl IntoView {
    let (search_input, set_search_input) = signal(String::new());
    let (show_suggestions, set_show_suggestions) = signal(false);
    let (suggestions, set_suggestions) = signal(Vec::<String>::new());
    let (loading, set_loading) = signal(false);

    // Debounced search resource
    let search_resource = Resource::new(
        move || search_input.get(),
        move |query| async move {
            if query.len() < 2 {
                return Vec::new();
            }
            search_test_names(query, Some(10)).await.unwrap_or_default()
        },
    );

    // Update suggestions when search resource changes
    Effect::new(move |_| {
        if let Some(results) = search_resource.get() {
            set_suggestions.set(results);
            set_loading.set(false);
        }
    });

    let on_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_input.set(value.clone());
        set_test_name.set(value.clone());

        if !value.is_empty() && value.len() >= 2 {
            set_loading.set(true);
            set_show_suggestions.set(true);
        } else {
            set_show_suggestions.set(false);
        }
    };

    let select_suggestion = move |suggestion: String| {
        set_test_name.set(suggestion.clone());
        set_search_input.set(suggestion);
        set_show_suggestions.set(false);
    };

    let on_focus = move |_| {
        if !search_input.get().is_empty() && !suggestions.get().is_empty() {
            set_show_suggestions.set(true);
        }
    };

    let on_blur = move |_| {
        // Delay hiding suggestions to allow for clicks
        set_timeout(
            move || set_show_suggestions.set(false),
            std::time::Duration::from_millis(150),
        );
    };

    view! {
        <div class="bg-white dark:bg-gray-700 p-6 rounded-lg shadow-md mb-6">
            <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">"Search Tests"</h3>

            <div class="relative">
                <div class="relative">
                    <input
                        type="text"
                        placeholder="Search for test names (e.g., //path/to/test:target)"
                        class="w-full p-3 bg-gray-50 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white pr-10"
                        on:input=on_input
                        on:focus=on_focus
                        on:blur=on_blur
                        prop:value=move || test_name.get()
                    />

                    // Loading spinner
                    {move || {
                        loading
                            .get()
                            .then(move || {
                                view! {
                                    <div class="absolute right-3 top-1/2 transform -translate-y-1/2">
                                        <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500"></div>
                                    </div>
                                }
                            })
                    }}
                </div>

                // Suggestions dropdown
                {move || {
                    (show_suggestions.get() && !suggestions.get().is_empty())
                        .then(move || {
                            view! {
                                <div class="absolute z-10 w-full mt-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md shadow-lg max-h-60 overflow-y-auto">
                                    <For
                                        each=move || suggestions.get()
                                        key=|suggestion| suggestion.clone()
                                        children=move |suggestion| {
                                            let suggestion_clone = suggestion.clone();
                                            view! {
                                                <div
                                                    class="px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 cursor-pointer text-gray-900 dark:text-white border-b border-gray-100 dark:border-gray-600 last:border-b-0"
                                                    on:click=move |_| select_suggestion(
                                                        suggestion_clone.clone(),
                                                    )
                                                >
                                                    <div class="font-medium">{suggestion.clone()}</div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }
                        })
                }}
            </div>

            <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                "Start typing to search for test names. Results will appear as you type."
            </p>
        </div>
    }
}
