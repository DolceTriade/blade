use anyhow::anyhow;
use leptos::{either::Either, prelude::*};
use leptos_router::{components::A, hooks::use_location};
use web_sys::KeyboardEvent;

use crate::components::{
    accordion::*,
    card::Card,
    list::*,
    searchbar::Searchbar,
    statusicon::StatusIcon,
    tooltip::Tooltip,
};

fn junit_status_to_status(s: junit_parser::TestStatus) -> state::Status {
    match s {
        junit_parser::TestStatus::Success => state::Status::Success,
        junit_parser::TestStatus::Error(_) => state::Status::Fail,
        junit_parser::TestStatus::Failure(_) => state::Status::Fail,
        junit_parser::TestStatus::Skipped(_) => state::Status::Skip,
    }
}

fn status_weight(s: state::Status) -> u8 {
    match s {
        state::Status::InProgress => 1,
        state::Status::Fail => 2,
        state::Status::Skip => 3,
        state::Status::Success => 4,
        _ => 5,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortType {
    Alphabetical,
    Duration,
    NoSort,
}

impl std::str::FromStr for SortType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "alphabetical" => Ok(SortType::Alphabetical),
            "duration" => Ok(SortType::Duration),
            "nosort" => Ok(SortType::NoSort),
            _ => Err(anyhow!("failed to parse {s} into SortType")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
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

fn sort_runs(runs: &[state::TestRun]) -> Vec<&state::TestRun> {
    let mut runs = runs.iter().collect::<Vec<_>>();
    runs.sort_unstable_by(|a, b| {
        let a_s = status_weight(a.status);
        let b_s = status_weight(b.status);
        if a_s != b_s {
            return a_s.cmp(&b_s);
        }
        if a.run != b.run {
            return a.run.cmp(&b.run);
        }
        if a.shard != b.shard {
            return a.shard.cmp(&b.shard);
        }
        a.attempt.cmp(&b.attempt)
    });
    runs
}

type TestListItem = (
    String,
    String,
    junit_parser::TestStatus,
    std::time::Duration,
);

fn junit_status_weight(s: &junit_parser::TestStatus) -> u8 {
    match s {
        junit_parser::TestStatus::Error(_) => 1,
        junit_parser::TestStatus::Failure(_) => 1,
        junit_parser::TestStatus::Skipped(_) => 2,
        junit_parser::TestStatus::Success => 3,
    }
}

fn sort_test_list_items(
    cases: &[TestListItem],
    sort_by: SortType,
    sort_order: SortOrder,
) -> Vec<TestListItem> {
    let mut vec = cases.to_vec();

    vec.sort_unstable_by(|a, b| {
        let a_s = junit_status_weight(&a.2);
        let b_s = junit_status_weight(&b.2);
        if a_s != b_s {
            return a_s.cmp(&b_s);
        }

        match sort_by {
            SortType::Duration => a.3.partial_cmp(&b.3).unwrap(),
            SortType::Alphabetical => a.1.partial_cmp(&b.1).unwrap(),
            SortType::NoSort => std::cmp::Ordering::Equal,
        }
    });

    if matches!(sort_order, SortOrder::Ascending) {
        vec.reverse();
    }
    vec
}

#[allow(non_snake_case)]
#[component]
pub fn TestRunList(
    sort_by: ReadSignal<SortType>,
    set_sort_by: WriteSignal<SortType>,
    sort_order: ReadSignal<SortOrder>,
    set_sort_order: WriteSignal<SortOrder>,
    hide_success: ReadSignal<bool>,
    set_hide_success: WriteSignal<bool>,
) -> impl IntoView {
    let test = expect_context::<Memo<Result<state::Test, String>>>();
    let xml = expect_context::<LocalResource<Option<junit_parser::TestSuites>>>();
    let click = move |test: String| {
        document()
            .get_element_by_id(&test)
            .map(|el| el.scroll_into_view())
    };
    let (filter, set_filter) = signal("".to_string());
    let search_key = move |e: KeyboardEvent| {
        let value = event_target_value(&e);
        set_filter.set(value);
    };

    view! {
        <div class="p-xs flex flex-row justify-between">
            <Searchbar id="search" placeholder="Filter tests..." keyup=search_key />
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
                            <option value="NoSort">No Sort</option>
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
                            <label>Hide Success</label>
                            <input
                                class="ml-4"
                                type="checkbox"
                                prop:checked=move || hide_success.get()
                                on:change=move |ev| {
                                    let val = event_target_checked(&ev);
                                    set_hide_success(val);
                                }
                            />
                        </span>
                    </Card>
                </div>
            </div>
        </div>
        <Accordion>
            {move || {
                test.with(|test| test.as_ref().map(|test| test.runs.len() > 1).unwrap_or(false))
                    .then(move || {
                        view! {
                            <AccordionItem header=move || view! { <h3>Runs</h3> }>
                                <List>
                                    <For
                                        each=move || {
                                            test.with(move |test| {
                                                sort_runs(&test.as_ref().unwrap().runs)
                                                    .into_iter()
                                                    .cloned()
                                                    .collect::<Vec<_>>()
                                            })
                                        }

                                        key=move |r| (r.run, r.shard, r.attempt)
                                        children=move |run| {
                                            let mut q = use_location().query.get();
                                            let path = use_location().pathname;
                                            let link = path
                                                .with(move |path| {
                                                    q.replace("run", run.run.to_string());
                                                    q.replace("shard", run.shard.to_string());
                                                    q.replace("attempt", run.attempt.to_string());
                                                    format!("{}{}", path, q.to_query_string())
                                                });
                                            let tooltip = format!(
                                                "Run #{} Shard #{} Attempt #{}",
                                                run.run,
                                                run.shard,
                                                run.attempt,
                                            );
                                            let tooltip_clone = tooltip.clone();
                                            view! {
                                                <ListItem hide=Signal::derive(move || false)>
                                                    <A href=link>
                                                        <div class="flex items-center justify-start w-full">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=run.status.into()
                                                                />

                                                            </span>
                                                            <div class="pl-4 max-w-3/4 float-left overflow-hidden overflow-x-auto whitespace-nowrap">
                                                                <Tooltip tooltip=move || {
                                                                    view! { <span class="p-1">{tooltip_clone.clone()}</span> }
                                                                }>
                                                                    <div class="flex items-center max-w-full float-left text-ellipsis whitespace-nowrap overflow-hidden text-sm">
                                                                        <span class="pl-4">{format!("Run {}", run.run)}</span>
                                                                        <span class="flex items-center  pl-1">
                                                                            <img class="h-4 w-4 dark:invert" src="/assets/shard.svg" />
                                                                            {run.shard}
                                                                        </span>
                                                                        <span class="flex items-center pl-1">
                                                                            <img class="h-4 w-4 dark:invert" src="/assets/number.svg" />
                                                                            {run.attempt}
                                                                        </span>

                                                                    </div>
                                                                </Tooltip>
                                                            </div>
                                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right whitespace-nowrap">
                                                                {format!("{}", humantime::format_duration(run.duration))}
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
            }} <AccordionItem header=move || view! { <h3>Tests</h3> }>
                <Suspense fallback=move || {
                    view! { <div>Loading...</div> }
                }>
                    {move || match xml.read().as_ref().and_then(|sw| sw.as_ref().map(|_| true)) {
                        Some(_) => {
                            Either::Left(
                                view! {
                                    <List>
                                        <For
                                            each=move || {
                                                xml.try_read()
                                                    .as_ref()
                                                    .and_then(|rg| rg.as_ref())
                                                    .and_then(|sw| {
                                                        sw.as_ref().and_then(|ts| ts.suites.first())
                                                    })
                                                    .map(|c| {
                                                        c.cases
                                                            .iter()
                                                            .map(|i| (
                                                                c.name.clone(),
                                                                i.name.clone(),
                                                                i.status.clone(),
                                                                std::time::Duration::from_secs_f64(i.time),
                                                            ))
                                                            .collect::<Vec<_>>()
                                                    })
                                                    .map(|c| {
                                                        sort_test_list_items(&c, sort_by.get(), sort_order.get())
                                                            .into_iter()
                                                            .collect::<Vec<_>>()
                                                    })
                                                    .unwrap_or_default()
                                            }

                                            key=move |c| (c.0.clone(), c.1.clone())
                                            children=move |c| {
                                                let test_status_clone = c.2.clone();
                                                let test_name = c.1.clone();
                                                let tooltip = test_name.clone();
                                                let filter_name = test_name.clone();
                                                let click_name = test_name.clone();
                                                let id_name = test_name.clone();
                                                let display_name = test_name.clone();
                                                view! {
                                                    <ListItem hide=Signal::derive(move || {
                                                        !filter.get().is_empty()
                                                            && !filter_name.contains(&filter.get())
                                                            || (hide_success.get()
                                                                && matches!(
                                                                    junit_status_to_status(test_status_clone.clone()),
                                                                    state::Status::Success
                                                                ))
                                                    })>
                                                        <div
                                                            on:click=move |_| {
                                                                click(click_name.clone());
                                                            }
                                                            class="flex items-center justify-start w-full"
                                                            id=id_name
                                                        >
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=junit_status_to_status(c.2).into()
                                                                />

                                                            </span>
                                                            <span class="pl-4 max-w-3/4 float-left text-ellipsis whitespace-nowrap overflow-hidden">
                                                                <Tooltip tooltip=move || {
                                                                    view! { <span class="p-2">{tooltip.clone()}</span> }
                                                                }>
                                                                    <span class="max-w-full float-left text-ellipsis whitespace-nowrap overflow-hidden">
                                                                        {display_name}
                                                                    </span>
                                                                </Tooltip>
                                                            </span>
                                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right whitespace-nowrap">
                                                                {format!("{}", humantime::format_duration(c.3))}
                                                            </span>
                                                        </div>

                                                    </ListItem>
                                                }
                                            }
                                        />

                                    </List>
                                },
                            )
                        }
                        _ => Either::Right(view! { <div>Loading...</div> }),
                    }}

                </Suspense>
            </AccordionItem>
        </Accordion>
    }
}