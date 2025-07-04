use std::collections::HashMap;

use anyhow::anyhow;
use leptos::prelude::*;
use leptos_router::components::A;
use web_sys::KeyboardEvent;

use crate::components::{
    accordion::*,
    card::Card,
    list::*,
    searchbar::Searchbar,
    statusicon::StatusIcon,
    tooltip::Tooltip,
};

fn format_time(start: &std::time::SystemTime, end: Option<&std::time::SystemTime>) -> String {
    if end.is_none() {
        return "".to_string();
    }
    let e = end.unwrap();
    e.duration_since(*start)
        .map(|d| humantime::format_duration(d).to_string())
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

#[derive(Clone, Debug, PartialEq)]
enum SortType {
    Alphabetical,
    Timestamp,
    Duration,
}

impl std::str::FromStr for SortType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "alphabetical" => Ok(SortType::Alphabetical),
            "timestamp" => Ok(SortType::Timestamp),
            "duration" => Ok(SortType::Duration),
            _ => Err(anyhow!("failed to parse {s} into SortType")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SortOrder {
    Descending,
    Ascending,
}

impl std::str::FromStr for SortOrder {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "descending" => Ok(SortOrder::Descending),
            "ascending" => Ok(SortOrder::Ascending),
            _ => Err(anyhow!("failed to parse {s} into SortOrder")),
        }
    }
}

fn sorted_targets(
    targets: &HashMap<String, state::Target>,
    sort_by: SortType,
    sort_order: SortOrder,
    failed_first: bool,
) -> Vec<state::Target> {
    let mut vec = targets.values().cloned().collect::<Vec<_>>();

    vec.sort_unstable_by(|a, b| {
        if failed_first {
            let a_status = status_weight(&a.status);
            let b_status = status_weight(&b.status);
            if a_status != b_status {
                return a_status.partial_cmp(&b_status).unwrap();
            }
        }

        match sort_by {
            SortType::Timestamp => a
                .end
                .cmp(&b.end)
                .then_with(|| a.name.partial_cmp(&b.name).unwrap()),
            SortType::Duration => match (a.end, b.end) {
                (Some(a_end), Some(b_end)) => a_end.cmp(&b_end),
                (None, None) => a.name.partial_cmp(&b.name).unwrap(),
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
            },
            SortType::Alphabetical => a.name.partial_cmp(&b.name).unwrap(),
        }
    });
    if matches!(sort_order, SortOrder::Ascending) {
        vec.reverse();
    }
    vec
}

fn sorted_tests(
    tests: &HashMap<String, state::Test>,
    sort_by: SortType,
    sort_order: SortOrder,
    failed_first: bool,
) -> Vec<state::Test> {
    let mut vec = tests.values().cloned().collect::<Vec<_>>();

    vec.sort_unstable_by(|a, b| {
        if failed_first {
            let a_status = status_weight(&a.status);
            let b_status = status_weight(&b.status);
            if a_status != b_status {
                return a_status.partial_cmp(&b_status).unwrap();
            }
        }

        match sort_by {
            SortType::Timestamp => a
                .end
                .cmp(&b.end)
                .then_with(|| a.name.partial_cmp(&b.name).unwrap()),
            SortType::Duration => a
                .duration
                .cmp(&b.duration)
                .then_with(|| a.name.partial_cmp(&b.name).unwrap()),
            SortType::Alphabetical => a.name.partial_cmp(&b.name).unwrap(),
        }
    });

    if matches!(sort_order, SortOrder::Ascending) {
        vec.reverse();
    }
    vec
}

#[allow(non_snake_case)]
pub fn TargetList() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let (tests, _) = slice!(invocation.tests);
    let (targets, _) = slice!(invocation.targets);

    let (filter, set_filter) = signal("".to_string());
    let search_key = move |e: KeyboardEvent| {
        let value = event_target_value(&e);
        set_filter.set(value);
    };

    let (sort_by, set_sort_by) = signal(SortType::Timestamp);
    let (sort_order, set_sort_order) = signal(SortOrder::Descending);
    let (failed_first, set_failed_first) = signal(true);

    // Memoized sorted data to avoid recomputation
    let sorted_tests_memo = Memo::new(move |_| {
        tests.with(|t| sorted_tests(t, sort_by.get(), sort_order.get(), failed_first.get()))
    });

    let sorted_targets_memo = Memo::new(move |_| {
        targets.with(|t| sorted_targets(t, sort_by.get(), sort_order.get(), failed_first.get()))
    });
    view! {
        <div>
            <div class="p-xs flex flex-row justify-between">
                <Searchbar id="search" placeholder="Filter targets..." keyup=search_key />
                <div class="group p-1 place-self-center align-self-center flex flex-col">
                    <img class="h-4 w-4 dark:invert" src="/assets/sort.svg" />
                    <div class="hidden group-hover:flex group-hover:fixed flex-row">
                        <Card class="p-2">
                            <label>Sorting</label>
                            <select
                                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                                id="sort_type_dropdown"
                                on:change=move |ev| {
                                    let val = event_target_value(&ev).parse::<SortType>().unwrap();
                                    if sort_by.read() == val {
                                        return;
                                    }
                                    set_sort_by.set(val);
                                }
                            >
                                <option value="Timestamp">End Time</option>
                                <option value="Alphabetical">Alphabetical</option>
                                <option value="Duration">Duration</option>
                            </select>
                            <select
                                id="sort_order_dropdown"
                                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                                on:change=move |ev| {
                                    let val = event_target_value(&ev).parse::<SortOrder>().unwrap();
                                    if sort_order.read() == val {
                                        return;
                                    }
                                    set_sort_order.set(val);
                                }
                            >
                                <option value="Descending">Descending</option>
                                <option value="Ascending">Ascending</option>
                            </select>
                            <span>
                                <label>Failed first</label>
                                <input
                                    class="ml-4"
                                    type="checkbox"
                                    prop:checked=move || failed_first.get()
                                    on:change=move |ev| {
                                        let val = event_target_checked(&ev);
                                        set_failed_first(val);
                                    }
                                />
                            </span>
                        </Card>
                    </div>
                </div>
            </div>

            <Accordion>

                {move || {
                    tests
                        .with(|tests| !tests.is_empty())
                        .then(move || {
                            view! {
                                <AccordionItem header=move || view! { <h3>Tests</h3> }>
                                    <List>
                                        <For
                                            each=move || sorted_tests_memo.get()
                                            key=|t| (t.name.clone(), t.status)
                                            children=move |t| {
                                                let test_name = t.name.clone();
                                                let test_name_filter = test_name.clone();
                                                let test_name_tooltip = test_name.clone();
                                                let query = format!("test?target={test_name}");
                                                let link = url_escape::encode_query(&query).to_string();
                                                view! {
                                                    <ListItem hide=Signal::derive(move || {
                                                        !filter.get().is_empty()
                                                            && !test_name_filter.contains(&filter.get())
                                                    })>
                                                        <A href=link>
                                                            <div class="flex items-center justify-start w-full">
                                                                <span class="float-left">
                                                                    <StatusIcon
                                                                        class="h-4 w-4 max-w-fit"
                                                                        status=t.status.into()
                                                                    />

                                                                </span>
                                                                <span class="pl-4 max-w-3/4 float-left whitespace-nowrap text-ellipsis overflow-hidden">
                                                                    <Tooltip tooltip=move || {
                                                                        view! {
                                                                            <span class="p-2">{test_name_tooltip.clone()}</span>
                                                                        }
                                                                    }>
                                                                        <span class="max-w-full float-left text-ellipsis whitespace-nowrap overflow-hidden">
                                                                            {test_name}
                                                                        </span>
                                                                    </Tooltip>
                                                                </span>
                                                                <span class="text-gray-400 text-xs pl-1 ml-auto float-right whitespace-nowrap">
                                                                    {format!("{:.2?}", t.duration)}
                                                                </span>
                                                            </div>
                                                        </A>
                                                    </ListItem>
                                                }
                                            }
                                        />

                                    </List>
                                </AccordionItem>
                            }
                        })
                }} <AccordionItem header=move || view! { <h3>Targets</h3> }>
                    <List>
                        <For
                            each=move || sorted_targets_memo.get()
                            key=|t| (t.name.clone(), t.status)
                            children=move |t| {
                                let target_name = t.name.clone();
                                let target_name_filter = target_name.clone();
                                let target_name_tooltip = target_name.clone();
                                view! {
                                    <ListItem hide=Signal::derive(move || {
                                        !filter.get().is_empty()
                                            && !target_name_filter.contains(&filter.get())
                                    })>
                                        <div class="flex items-center justify-start w-full">
                                            <span class="float-left">
                                                <StatusIcon
                                                    class="h-4 w-4 max-w-fit"
                                                    status=t.status.into()
                                                />

                                            </span>
                                            <span class="pl-4 max-w-3/4 float-left">
                                                <Tooltip tooltip=move || {
                                                    view! {
                                                        <span class="p-2">{target_name_tooltip.clone()}</span>
                                                    }
                                                }>
                                                    <span class="max-w-full float-left text-ellipsis whitespace-nowrap overflow-hidden">
                                                        {target_name}
                                                    </span>
                                                </Tooltip>
                                            </span>
                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right whitespace-nowrap">
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
