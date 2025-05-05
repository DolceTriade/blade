use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};
use time::macros::format_description;

use crate::components::statusicon::StatusIcon;
use crate::components::clipboard::CopyToClipboard;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + std::marker::Send + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move || num.get().to_string()}</span>
            <span class="pl-1 text-xs">
                {move || format!("{}{}", suffix, if num.get() != 1 { "s" } else { "" })}
            </span>
        </div>
    }
}

#[derive(PartialEq)]
struct Counts {
    num_targets: usize,
    passing_targets: usize,
    failing_targets: usize,
    num_tests: usize,
    passing_tests: usize,
    failing_tests: usize,
    status: state::Status,
}

fn format_time(t: &std::time::SystemTime) -> String {
    let ts: time::OffsetDateTime = (*t).into();
    ts.format(&format_description!(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]"
    ))
    .unwrap_or(format!("{ts:#?}"))
}

fn ucfirst(s: &str) -> String {
    let mut new = s.to_owned();
    if let Some(start) = new.get_mut(0..1) {
        start.make_ascii_lowercase();
    }
    new
}

#[allow(non_snake_case)]
#[component]
pub fn SummaryHeader() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let counts = Memo::new(move |_| {
        invocation.with(|invocation| {
            let num_targets = invocation.targets.len();
            let mut passing_targets: usize = 0;
            let mut failing_targets: usize = 0;
            invocation.targets.values().for_each(|t| match t.status {
                state::Status::Success => passing_targets += 1,
                state::Status::Fail => failing_targets += 1,
                _ => {},
            });
            let num_tests = invocation.tests.len();
            let mut passing_tests: usize = 0;
            let mut failing_tests: usize = 0;
            invocation.tests.values().for_each(|t| match t.status {
                state::Status::Success => passing_tests += 1,
                state::Status::InProgress => {},
                _ => failing_tests += 1,
            });

            Counts {
                num_targets,
                passing_targets,
                failing_targets,
                num_tests,
                passing_tests,
                failing_tests,
                status: invocation.status,
            }
        })
    });
    move || {
        let num_targets = Signal::derive(move || counts.read().num_targets);
        let passing_targets = Signal::derive(move || counts.read().passing_targets);
        let failing_targets = Signal::derive(move || counts.read().failing_targets);
        let num_tests = Signal::derive(move || counts.read().num_tests);
        let passing_tests = Signal::derive(move || counts.read().passing_tests);
        let failing_tests = Signal::derive(move || counts.read().failing_tests);
        let status = Signal::derive(move || counts.read().status);
        let cmd = ucfirst(&invocation.read().command);
        let patterns = invocation.read().pattern.join(" ");
        let start = format_time(&invocation.read().start);
        let location = use_location();
        let duration = invocation
            .read()
            .end
            .map(|end| {
                let duration = end
                    .duration_since(invocation.read().start)
                    .unwrap_or_default();
                format!("Took {}", humantime::format_duration(duration))
            })
            .unwrap_or_default();
        view! {
            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col content-start divide-x">
                <div class="grid grid-rows-1 grid-flow-col place-content-start">
                    <div class="p-4 place-content-center self-center">
                        <StatusIcon class="h-8 w-8" status=status />
                    </div>
                    <div class="grid grid-rows-3 items-start self-center place-content-center">
                        <div class="flex gap-3 content-start place-items-center">
                            <div class="place-content-center">
                                <img class="h-6 w-6" src="/assets/bazel.svg"/>
                            </div>
                            <span class="text-lg">
                                <b>{cmd}</b>
                            </span>
                            <span class="inline-flex overflow-auto whitespace-nowrap">
                                {patterns.clone()}
                            </span>
                            <span>
                                <CopyToClipboard text=patterns attr:class="h-4 w-4 rounded-lg hover:bg-gray-500" />
                            </span>
                        </div>
                        <div class="text-grey-400 text-sm self-center">{start}</div>
                        <div class="flex gap-2 items-center">
                            {duration}
                            <A href=move || {
                                location
                                    .pathname
                                    .read()
                                    .strip_suffix("/details")
                                    .unwrap_or("details")
                                    .to_string()
                            }>
                                <span class="text-blue-500 underline">(details)</span>
                            </A>
                        </div>
                    </div>
                </div>
                <div class="content-center place-self-end self-center grid grid-rows-1 grid-flow-col">
                    <div class="p-4 place-content-center">
                        <img class="h-10 w-10" src="/assets/code.svg"/>
                    </div>
                    <div class="p-4 place-content-center">
                        <SummaryItem num=num_targets suffix="Total Target" />
                        <SummaryItem num=passing_targets suffix="Passing Target" />
                        {(failing_targets.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=failing_targets suffix="Failing Target" /> }
                            })}
                    </div>
                </div>
                <div class="content-center place-self-center self-center grid grid-rows-1 grid-flow-col">
                    {{(num_tests.get() > 0).then(||view!{
                        <div class="p-4 place-content-center">
                            <img class="h-10 w-10" src="/assets/test.svg"/>
                        </div>
                    })}}
                    <div class="p-4 place-content-center">
                        {(num_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=num_tests suffix="Total Test" /> }
                            })}

                        {(passing_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=passing_tests suffix="Passing Test" /> }
                            })}

                        {(failing_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=failing_tests suffix="Failing Test" /> }
                            })}
                    </div>
                </div>
            </div>
        }
    }
}
