use std::ops::Deref;

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use web_sys::KeyboardEvent;

use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::searchbar::Searchbar;
use crate::components::statusicon::StatusIcon;
use crate::components::tooltip::Tooltip;

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

fn sort_runs(runs: &[state::TestRun]) -> Vec<state::TestRun> {
    let mut runs = runs.to_owned();
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

fn sort_tests(cases: &[TestListItem]) -> Vec<TestListItem> {
    let mut cases = cases.to_owned();
    cases.sort_unstable_by(|a, b| {
        let a_s = junit_status_weight(&a.2);
        let b_s = junit_status_weight(&b.2);
        if a_s != b_s {
            return a_s.cmp(&b_s);
        }
        a.1.cmp(&b.1)
    });
    cases
}

#[allow(non_snake_case)]
#[component]
pub fn TestRunList() -> impl IntoView {
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
        <div class="p-xs">
            <Searchbar id="search" placeholder="Filter tests..." keyup=search_key/>
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
                                            test.with(move |test| sort_runs(
                                                &test.as_ref().unwrap().runs,
                                            ))
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
                                            view! {
                                                <ListItem hide=Signal::derive(|| false)>
                                                    <A href=link>
                                                        <div class="flex items-center justify-start w-full">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=run.status.into()
                                                                />

                                                            </span>
                                                            <div class="pl-4 max-w-3/4 float-left overflow-hidden overflow-x-scroll whitespace-nowrap">
                                                                <Tooltip tooltip=move || {
                                                                    view! { <span class="p-1">{tooltip.clone()}</span> }
                                                                }>
                                                                    <div class="flex items-center max-w-full float-left text-ellipsis whitespace-nowrap overflow-hidden text-sm">
                                                                        <span class="pl-4">{format!("Run {}", run.run)}</span>
                                                                        <span class="flex items-center  pl-1">
                                                                            <img class="h-4 w-4" src="/assets/shard.svg"/>
                                                                            {run.shard}
                                                                        </span>
                                                                        <span class="flex items-center pl-1">
                                                                            <img class="h-4 w-4" src="/assets/number.svg"/>
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
            }}
            <AccordionItem header=move || view! { <h3>Tests</h3> }>
                <Suspense fallback=move || {
                    view! { <div>Loading...</div> }
                }>
                    {move || match xml.read().as_ref().and_then(|sw| sw.deref().as_ref().map(|_| true)) {
                            Some(_) => {
                                view! {
                                    <List>
                                        <For
                                            each=move || {
                                                xml.try_read()
                                                    .as_ref()
                                                    .and_then(|rg| rg.deref().as_ref())
                                                    .and_then(|sw| {
                                                        sw.deref().clone().and_then(|ts| ts.suites.first().cloned())
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
                                                    .map(|c| sort_tests(&c))
                                                    .unwrap_or_default()
                                            }

                                            key=move |c| (c.0.clone(), c.1.clone())
                                            children=move |c| {
                                                let tooltip = c.1.clone();
                                                let id_memo = c.1.clone();
                                                let id = Memo::new(move |_| id_memo.clone());
                                                view! {
                                                    <ListItem hide=Signal::derive(move || {
                                                        !filter.get().is_empty()
                                                            && !id.with(|id| id.contains(&filter.get()))
                                                    })>
                                                        <div
                                                            on:click=move |_| {
                                                                click(id.get());
                                                            }

                                                            // TODO: Fix
                                                            // attr:test=move||id
                                                            class="flex items-center justify-start w-full"
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
                                                                        {c.1.clone()}
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
                                }
                                    .into_any()
                            }
                            _ => {
                                view! {
                                    // TODO: Fix
                                    // attr:test=move||id

                                    // TODO: Fix
                                    // attr:test=move||id

                                    // TODO: Fix
                                    // attr:test=move||id

                                    // TODO: Fix
                                    // attr:test=move||id

                                    // TODO: Fix
                                    // attr:test=move||id

                                    <div>Loading...</div>
                                }
                                    .into_any()
                            }
                        }
                    }

                </Suspense>
            </AccordionItem>
        </Accordion>
    }
}
