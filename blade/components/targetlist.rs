use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::statusicon::StatusIcon;
use leptos::*;
use leptos_dom::{document, helpers::event_target};
use state;
use std::collections::HashMap;
use std::string::ToString;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

fn format_time(start: &std::time::SystemTime, end: Option<&std::time::SystemTime>) -> String {
    if end.is_none() {
        return "".to_string();
    }
    let e = end.unwrap();
    e.duration_since(*start)
        .map(|d| format!("{:#?}", d))
        .unwrap_or_default()
}

fn status_weight(s: &state::Status) -> u8 {
    match s {
        state::Status::InProgress => 0,
        state::Status::Fail => 1,
        state::Status::Skip => 2,
        state::Status::Success => 3,
        state::Status::Unknown => 4,
    }
}

fn sorted_targets(targets: &HashMap<String, state::Target>) -> Vec<state::Target> {
    let mut vec = targets.values().collect::<Vec<_>>();
    vec.sort_unstable_by(|a, b| {
        let a_status = status_weight(&a.status);
        let b_status = status_weight(&b.status);
        if a_status != b_status {
            return a_status.partial_cmp(&b_status).unwrap();
        }
        a.name.partial_cmp(&b.name).unwrap()
    });
    vec.into_iter().cloned().collect::<Vec<_>>()
}

fn sorted_tests(tests: &HashMap<String, state::Test>) -> Vec<state::Test> {
    let mut vec = tests.values().collect::<Vec<_>>();
    vec.sort_unstable_by(|a, b| {
        if a.success == b.success {
            return a.name.partial_cmp(&b.name).unwrap();
        }
        match a.success {
            true => std::cmp::Ordering::Greater,
            false => std::cmp::Ordering::Less,
        }
    });
    vec.into_iter().cloned().collect::<Vec<_>>()
}

#[allow(non_snake_case)]
pub fn TargetList() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let (tests, _) = slice!(invocation.tests);
    let (targets, _) = slice!(invocation.targets);

    let hover = move |e: leptos::ev::MouseEvent| {
        let el = event_target::<web_sys::HtmlSpanElement>(&e);
        el.next_element_sibling()
            .map(|s| {
                let body = document().body().unwrap().get_bounding_client_rect();
                let span = s.unchecked_into::<web_sys::HtmlSpanElement>();
                span.get_bounding_client_rect().y() - body.y()
            })
            .map(|t| el.set_attribute("style", &format!("top: {}px", t)).ok());
    };

    let (filter, set_filter) = create_signal("".to_string());
    let search_changed = move |e: KeyboardEvent| {
        let value = event_target_value(&e);
        set_filter.set(value);
    };
    view! {
        <div>
            <div class="p-xs">
                <div class="relative">
                    <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
                        <svg
                            class="w-4 h-4 text-gray-500 dark:text-gray-400"
                            aria-hidden="true"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 20 20"
                        >
                            <path
                                stroke="currentColor"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"
                            ></path>
                        </svg>
                    </div>
                    <input
                        on:keyup=search_changed
                        type="search"
                        id="search"
                        class="block w-full p-4 ps-10 text-sm text-gray-900 border border-gray-300 rounded-2xlg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                        placeholder="Filter targets..."
                        required
                    />
                </div>
            </div>
            <Accordion>

                {move||with!(|tests| tests.is_empty())
                    .then(move || {
                        view! {
                            <AccordionItem header=move || view! { <h3>Tests</h3> }>
                                <List>
                                    <For
                                        each=move || with!(|tests| sorted_tests(tests))
                                        key=|t| (t.name.to_string(), t.success)
                                        children=move |t| {
                                            let label = t.name.clone();
                                            view! {
                                                <ListItem hide=Signal::derive(move || {
                                                    !filter.get().is_empty() && !label.contains(&filter.get())
                                                })>
                                                    <div class="group flex items-center justify-start w-full">
                                                        <span class="float-left">
                                                            <StatusIcon
                                                                class="h-4 w-4 max-w-fit"
                                                                status=if t.success {
                                                                    state::Status::Success.into()
                                                                } else {
                                                                    state::Status::Fail.into()
                                                                }
                                                            />

                                                        </span>
                                                        <span
                                                            class="label-name pl-4 max-w-3/4 float-left text-ellipsis overflow-hidden group-hover:overflow-visible group-hover:absolute group-hover:bg-slate-200 group-hover:w-fit group-hover:rounded-md"
                                                            on:mouseenter=hover
                                                        >
                                                            {t.name.clone()}
                                                        </span>
                                                        <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                            {format!("{:#?}", t.duration)}
                                                        </span>
                                                    </div>
                                                </ListItem>
                                            }
                                        }
                                    />

                                </List>
                            </AccordionItem>
                        }
                    })}
                <AccordionItem header=move || view! { <h3>Targets</h3> }>
                    <List>
                        <For
                            each=move || with!(|targets| sorted_targets(targets))
                            key=|t| (t.name.to_string(), t.status.clone())
                            children=move |t| {
                                let label = t.name.clone();
                                view! {
                                    <ListItem hide=Signal::derive(move || {
                                        !filter.get().is_empty() && !label.contains(&filter.get())
                                    })>
                                        <div class="group flex items-center justify-start w-full">
                                            <span class="float-left">
                                                <StatusIcon
                                                    class="h-4 w-4 max-w-fit"
                                                    status=t.status.clone().into()
                                                />

                                            </span>
                                            <span
                                                class="label-name pl-4 max-w-3/4 float-left text-ellipsis overflow-hidden group-hover:overflow-visible group-hover:absolute group-hover:bg-slate-200 group-hover:w-fit group-hover:rounded-md"
                                                on:mouseenter=hover
                                            >
                                                {t.name.clone()}
                                            </span>
                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                {format_time(&t.start, t.end.as_ref())}

                                            </span>
                                        </div>
                                    </ListItem>
                                }
                            }
                        />

                    </List>
                </AccordionItem>
            </Accordion>
        </div>
    }
}
