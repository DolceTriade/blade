use leptos::{leptos_dom::helpers::event_target_value, prelude::*};
use state::{Status, TestFilter, TestFilterItem, TestFilterOp};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
struct FilterBuilder {
    filter_type: String, // "Duration", "Status", "Metadata", "BazelFlags", "LogOutput"
    operation: TestFilterOp,
    invert: bool,
    // Values for different filter types
    duration_seconds: f64,
    status: Status,
    metadata_key: String,
    metadata_value: String,
    bazel_flag: String,
    bazel_value: String,
    log_output: String,
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self {
            filter_type: "Duration".to_string(),
            operation: TestFilterOp::Equals,
            invert: false,
            duration_seconds: 0.0,
            status: Status::Success,
            metadata_key: String::new(),
            metadata_value: String::new(),
            bazel_flag: String::new(),
            bazel_value: String::new(),
            log_output: String::new(),
        }
    }
}

impl FilterBuilder {
    fn to_test_filter(&self) -> Option<TestFilter> {
        let filter_item = match self.filter_type.as_str() {
            "Duration" => {
                TestFilterItem::Duration(std::time::Duration::from_secs_f64(self.duration_seconds))
            },
            "Status" => TestFilterItem::Status(self.status),
            "Metadata" => {
                if self.metadata_key.is_empty() || self.metadata_value.is_empty() {
                    return None;
                }
                TestFilterItem::Metadata {
                    key: self.metadata_key.clone(),
                    value: self.metadata_value.clone(),
                }
            },
            "BazelFlags" => {
                if self.bazel_flag.is_empty() {
                    return None;
                }
                TestFilterItem::BazelFlags {
                    flag: self.bazel_flag.clone(),
                    value: self.bazel_value.clone(),
                }
            },
            "LogOutput" => {
                if self.log_output.is_empty() {
                    return None;
                }
                TestFilterItem::LogOutput(self.log_output.clone())
            },
            _ => return None,
        };

        Some(TestFilter {
            op: self.operation.clone(),
            invert: self.invert,
            filter: filter_item,
        })
    }
}

#[allow(non_snake_case)]
#[component]
pub fn FilterControls(set_filters: WriteSignal<Vec<TestFilter>>) -> impl IntoView {
    let (filter_builders, set_filter_builders) = signal(vec![FilterBuilder::default()]);

    let add_filter = move |_| {
        set_filter_builders.update(|builders| {
            builders.push(FilterBuilder::default());
        });
    };

    let remove_filter = move |index: usize| {
        set_filter_builders.update(|builders| {
            if builders.len() > 1 {
                builders.remove(index);
            }
        });
    };

    let apply_filters = move |_| {
        let filters: Vec<TestFilter> = filter_builders
            .get()
            .iter()
            .filter_map(|builder| builder.to_test_filter())
            .collect();
        set_filters.set(filters);
    };

    view! {
        <div class="bg-white dark:bg-gray-700 p-6 rounded-lg shadow-md mb-6">
            <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                "Advanced Filters"
            </h3>

            // Dynamic filter builders
            <div class="space-y-4">
                <For
                    each=move || filter_builders.get().into_iter().enumerate()
                    key=|(i, _)| *i
                    children=move |(index, builder)| {
                        view! {
                            <FilterRow
                                builder=builder
                                index=index
                                on_remove=remove_filter
                                on_update=move |updated_builder| {
                                    set_filter_builders
                                        .update(|builders| {
                                            if let Some(b) = builders.get_mut(index) {
                                                *b = updated_builder;
                                            }
                                        });
                                }
                            />
                        }
                    }
                />
            </div>

            // Action buttons
            <div class="flex items-center justify-between mt-6">
                <button
                    class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-md font-medium transition-colors"
                    on:click=add_filter
                >
                    "+ Add Filter"
                </button>
                <button
                    class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-semibold transition-colors"
                    on:click=apply_filters
                >
                    "Apply Filters"
                </button>
            </div>
        </div>
    }
}

#[allow(non_snake_case)]
#[component]
fn FilterRow(
    builder: FilterBuilder,
    index: usize,
    on_remove: impl Fn(usize) + Copy + 'static,
    on_update: impl Fn(FilterBuilder) + Copy + 'static,
) -> impl IntoView {
    let (current_builder, set_current_builder) = signal(builder);

    // Update parent when local state changes
    Effect::new(move |_| {
        on_update(current_builder.get());
    });

    view! {
        <div class="p-4 bg-gray-50 dark:bg-gray-600 rounded-lg border border-gray-200 dark:border-gray-500">
            <div class="grid grid-cols-1 md:grid-cols-6 gap-4 items-end">
                // Filter type dropdown
                <div class="md:col-span-2">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        "Filter Type"
                    </label>
                    <select
                        class="w-full p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_current_builder.update(|b| b.filter_type = value);
                        }
                        prop:value=move || current_builder.get().filter_type
                    >
                        <option value="Duration">"Duration"</option>
                        <option value="Status">"Status"</option>
                        <option value="Metadata">"Metadata"</option>
                        <option value="BazelFlags">"Bazel Flags"</option>
                        <option value="LogOutput">"Log Output"</option>
                    </select>
                </div>

                // Operation dropdown
                <div class="md:col-span-2">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        "Operation"
                    </label>
                    <select
                        class="w-full p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let op = match value.as_str() {
                                "Contains" => TestFilterOp::Contains,
                                "GreaterThan" => TestFilterOp::GreaterThan,
                                "LessThan" => TestFilterOp::LessThan,
                                _ => TestFilterOp::Equals,
                            };
                            set_current_builder.update(|b| b.operation = op);
                        }
                    >
                        <option value="Equals">"Equals"</option>
                        <option value="Contains">"Contains"</option>
                        <option value="GreaterThan">"Greater Than"</option>
                        <option value="LessThan">"Less Than"</option>
                    </select>
                </div>

                // Dynamic value input based on filter type
                <div class="md:col-span-2">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        "Value"
                    </label>
                    {move || {
                        let builder = current_builder.get();
                        match builder.filter_type.as_str() {
                            "Duration" => {
                                view! {
                                    <input
                                        type="number"
                                        step="0.001"
                                        placeholder="Duration in seconds"
                                        class="w-full p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                        on:input=move |ev| {
                                            if let Ok(value) = event_target_value(&ev).parse::<f64>() {
                                                set_current_builder.update(|b| b.duration_seconds = value);
                                            }
                                        }
                                        prop:value=move || {
                                            current_builder.get().duration_seconds.to_string()
                                        }
                                    />
                                }
                                    .into_any()
                            }
                            "Status" => {
                                view! {
                                    <select
                                        class="w-full p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            let status = Status::parse(&value);
                                            set_current_builder.update(|b| b.status = status);
                                        }
                                    >
                                        <option value="Success">"Success"</option>
                                        <option value="Fail">"Fail"</option>
                                        <option value="Skip">"Skip"</option>
                                        <option value="InProgress">"In Progress"</option>
                                        <option value="Unknown">"Unknown"</option>
                                    </select>
                                }
                                    .into_any()
                            }
                            "Metadata" => {
                                view! {
                                    <div class="flex space-x-2">
                                        <input
                                            type="text"
                                            placeholder="Key"
                                            class="flex-1 p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev);
                                                set_current_builder.update(|b| b.metadata_key = value);
                                            }
                                            prop:value=move || current_builder.get().metadata_key
                                        />
                                        <input
                                            type="text"
                                            placeholder="Value"
                                            class="flex-1 p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev);
                                                set_current_builder.update(|b| b.metadata_value = value);
                                            }
                                            prop:value=move || current_builder.get().metadata_value
                                        />
                                    </div>
                                }
                                    .into_any()
                            }
                            "BazelFlags" => {
                                view! {
                                    <div class="flex space-x-2">
                                        <input
                                            type="text"
                                            placeholder="Flag (e.g., --test_env)"
                                            class="flex-1 p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev);
                                                set_current_builder.update(|b| b.bazel_flag = value);
                                            }
                                            prop:value=move || current_builder.get().bazel_flag
                                        />
                                        <input
                                            type="text"
                                            placeholder="Value (optional)"
                                            title="Leave empty to match any occurrence of the flag"
                                            class="flex-1 p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev);
                                                set_current_builder.update(|b| b.bazel_value = value);
                                            }
                                            prop:value=move || current_builder.get().bazel_value
                                        />
                                    </div>
                                }
                                    .into_any()
                            }
                            "LogOutput" => {
                                view! {
                                    <input
                                        type="text"
                                        placeholder="Search in log output"
                                        class="w-full p-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-500 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            set_current_builder.update(|b| b.log_output = value);
                                        }
                                        prop:value=move || current_builder.get().log_output
                                    />
                                }
                                    .into_any()
                            }
                            _ => {
                                view! {
                                    <input
                                        type="text"
                                        disabled=true
                                        class="w-full p-2 bg-gray-200 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md"
                                        placeholder="Select filter type"
                                    />
                                }
                                    .into_any()
                            }
                        }
                    }}
                </div>

                // Controls column
                <div class="flex items-center space-x-2">
                    // Invert checkbox
                    <label class="flex items-center">
                        <input
                            type="checkbox"
                            class="mr-2 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                            prop:checked=move || current_builder.get().invert
                            on:change=move |ev| {
                                let checked = ev
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                    .map(|input| input.checked())
                                    .unwrap_or(false);
                                set_current_builder.update(|b| b.invert = checked);
                            }
                        />
                        <span class="text-sm text-gray-700 dark:text-gray-300">"Not"</span>
                    </label>

                    // Remove button
                    <button
                        class="px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm font-medium transition-colors"
                        on:click=move |_| on_remove(index)
                    >
                        "×"
                    </button>
                </div>
            </div>
        </div>
    }
}
