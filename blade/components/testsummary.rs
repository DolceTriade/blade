use leptos::*;
use leptos_router::*;

use crate::components::statusicon::StatusIcon;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: usize, suffix: S) -> impl IntoView
where
    S: std::fmt::Display + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move || num.to_string()}</span>
            <span class="text-xs">{move || format!("{}", suffix)}</span>
        </div>
    }
}

#[derive(Debug, Default, Clone)]
struct TestCounts {
    passing: usize,
    failing: usize,
    skipped: usize,
}

fn get_test_counts(cases: &[junit_parser::TestCase]) -> TestCounts {
    let mut tc = TestCounts::default();
    cases.iter().for_each(|c| match c.status {
        junit_parser::TestStatus::Success => tc.passing += 1,
        junit_parser::TestStatus::Error(_) => tc.failing += 1,
        junit_parser::TestStatus::Failure(_) => tc.failing += 1,
        junit_parser::TestStatus::Skipped(_) => tc.skipped += 1,
    });
    tc
}

#[allow(non_snake_case)]
#[component]
fn RunSummary() -> impl IntoView {
    let run = expect_context::<Memo<Option<state::TestRun>>>();
    let xml = expect_context::<RwSignal<Option<Option<junit_parser::TestSuites>>>>();
    view! {
        {move || {
            run.with(|run| {
                run.as_ref()
                    .map(|run| {
                        view! {
                            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center">
                                <div>
                                    <StatusIcon class="h-5 w-5" status=run.status.into()/>
                                </div>
                                <div class="w-fit h-fit grid grid-rows-1 grid-flow-col items-center justify-center text-s">
                                    <span class="pl-4">{format!("Run #{}", run.run)}</span>
                                    <span class="pl-4">{format!("Shard #{}", run.shard)}</span>
                                    <span class="pl-4">{format!("Attempt #{}", run.attempt)}</span>
                                </div>
                                <div class="pl-1 text-s">{format!("in {:#?}", run.duration)}</div>
                                {move || {
                                    xml.with(|ts| {
                                        ts.clone()
                                            .flatten()
                                            .and_then(|ts| {
                                                ts.suites.first().map(|s| get_test_counts(&s.cases))
                                            })
                                            .map(|tc| {
                                                view! {
                                                    {(tc.passing > 0)
                                                        .then(|| {
                                                            view! {
                                                                <span>
                                                                    <SummaryItem num=tc.passing suffix="Passing"/>
                                                                </span>
                                                            }
                                                        })}

                                                    {(tc.failing > 0)
                                                        .then(|| {
                                                            view! {
                                                                <span>
                                                                    <SummaryItem num=tc.failing suffix="Failing"/>
                                                                </span>
                                                            }
                                                        })}

                                                    {(tc.skipped > 0)
                                                        .then(|| {
                                                            view! {
                                                                <span>
                                                                    <SummaryItem num=tc.skipped suffix="Skipped"/>
                                                                </span>
                                                            }
                                                        })}
                                                }
                                                    .into_view()
                                            })
                                            .unwrap_or_default()
                                    })
                                }}

                            </div>
                        }
                            .into_view()
                    })
                    .unwrap_or_default()
            })
        }}
    }
}

#[allow(non_snake_case)]
#[component]
pub fn TestSummary() -> impl IntoView
where
{
    let link = create_memo(move |_| {
        let loc = use_location();
        let mut path = loc
            .pathname
            .with(move |p| p.split('/').map(|s| s.to_string()).collect::<Vec<_>>());
        path.pop();
        path.join("/")
    });

    let test = expect_context::<Memo<Result<state::Test, String>>>();
    view! {
        <div class="w-screen h-fit grid grid-rows-2 grid-flow-col items-center justify-center divide-y">
            {move || {
                test.with(|test| {
                    test.as_ref()
                        .ok()
                        .map(|test| {
                            view! {
                                <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center p-2">
                                    <A class="absolute float-left" href=move || link.get()>
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="h-8 w-8"
                                            height="24"
                                            viewBox="0 0 24 24"
                                            width="24"
                                        >
                                            <path d="M0 0h24v24H0z" fill="none"></path>
                                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"></path>
                                        </svg>
                                    </A>
                                    <div>
                                        <StatusIcon class="h-8 w-8" status=test.status.into()/>
                                    </div>
                                    <div class="pl-4">
                                        <b>{test.name.clone()}</b>
                                    </div>
                                    <div class="pl-1 text-s">
                                        {format!("in {:#?}", test.duration)}
                                    </div>
                                </div>
                                <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center p-2">
                                    <RunSummary/>
                                </div>
                            }
                                .into_view()
                        })
                        .unwrap_or_default()
                })
            }}

        </div>
    }
}
