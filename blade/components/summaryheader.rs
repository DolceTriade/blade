use crate::components::statusicon::StatusIcon;
use leptos::*;
use leptos_router::A;
use time::macros::format_description;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + 'static,
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
    let counts = create_memo(move |_| {
        invocation.with(|invocation| {
            let num_targets = invocation.targets.len();
            let mut passing_targets: usize = 0;
            let mut failing_targets: usize = 0;
            invocation.targets.values().for_each(|t| match t.status {
                state::Status::Success => passing_targets += 1,
                state::Status::Fail => failing_targets += 1,
                _ => {}
            });
            let num_tests = invocation.tests.len();
            let mut passing_tests: usize = 0;
            let mut failing_tests: usize = 0;
            invocation.tests.values().for_each(|t| match t.status {
                state::Status::Success => passing_tests += 1,
                state::Status::InProgress => {}
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
        let num_targets = Signal::derive(move || with!(|counts| counts.num_targets));
        let passing_targets = Signal::derive(move || with!(|counts| counts.passing_targets));
        let failing_targets = Signal::derive(move || with!(|counts| counts.failing_targets));
        let num_tests = Signal::derive(move || with!(|counts| counts.num_tests));
        let passing_tests = Signal::derive(move || with!(|counts| counts.passing_tests));
        let failing_tests = Signal::derive(move || with!(|counts| counts.failing_tests));
        let status = Signal::derive(move || with!(|counts| counts.status));
        let cmd = with!(|invocation| ucfirst(&invocation.command));
        let patterns = with!(|invocation| invocation.pattern.join(","));
        let start = with!(|invocation| format_time(&invocation.start));
        let duration = with!(|invocation| {
            let Some(end) = invocation.end else {
                return "".to_string();
            };
            let duration = end.duration_since(invocation.start).unwrap_or_default();
            format!("Took {}", humantime::format_duration(duration))
        });
        view! {
            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center divide-x">
                <div class="absolute">
                    <div class="flex gap-2 items-center">
                        <span class="text-lg">
                            <b>{cmd}</b>
                        </span>
                        <span class="inline-flex max-w-1/4 overflow-auto">{patterns}</span>
                        <span>@</span>
                        <span class="text-grey-400 text-sm">{start}</span>
                    </div>
                    <div class="flex gap-2 items-center">
                        {duration} <A class="text-blue-500 underline" href="details">
                            (details)
                        </A>
                    </div>
                </div>
                <div class="p-4">
                    <StatusIcon class="h-8 w-8" status=status.into()/>
                </div>
                <SummaryItem num=num_targets suffix="Total Target"/>
                <SummaryItem num=passing_targets suffix="Passing Target"/>
                {(failing_targets.get() > 0)
                    .then(|| {
                        view! { <SummaryItem num=failing_targets suffix="Failing Target"/> }
                    })}

                {(num_tests.get() > 0)
                    .then(|| {
                        view! { <SummaryItem num=num_tests suffix="Total Test"/> }
                    })}

                {(passing_tests.get() > 0)
                    .then(|| {
                        view! { <SummaryItem num=passing_tests suffix="Passing Test"/> }
                    })}

                {(failing_tests.get() > 0)
                    .then(|| {
                        view! { <SummaryItem num=failing_tests suffix="Failing Test"/> }
                    })}

            </div>
        }
    }
}
