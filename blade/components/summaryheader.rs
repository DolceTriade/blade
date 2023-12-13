use crate::components::statusicon::StatusIcon;
use leptos::*;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move||num.get().to_string()}</span>
            <span class="text-xs">{move||format!("{}{}", suffix, if num.get() != 1 { "s" } else { "" })}</span>
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
}

#[allow(non_snake_case)]
#[component]
pub fn SummaryHeader() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let counts = create_memo(move |_| {
        with!(|invocation| {
        let num_targets = invocation.targets.len();
        let mut passing_targets: usize = 0;
        let mut failing_targets: usize = 0;
        invocation.targets.values().for_each(|t| match t.status {
            state::Status::Success => passing_targets += 1,
            state::Status::Fail => failing_targets += 1,
            _ => {}
        });
        let num_tests = invocation.tests.len();
        let passing_tests = invocation.tests.values().filter(|v| v.success).count();
        let failing_tests = num_tests - passing_tests;
        Counts {
            num_targets,
            passing_targets,
            failing_targets,
            num_tests,
            passing_tests,
            failing_tests,
        }})
    });
    move|| {
    let num_targets = Signal::derive(move||with!(|counts| counts.num_targets));
    let passing_targets = Signal::derive(move||with!(|counts| counts.passing_targets));
    let failing_targets = Signal::derive(move||with!(|counts| counts.failing_targets));
    let num_tests = Signal::derive(move||with!(|counts| counts.num_tests));
    let passing_tests = Signal::derive(move||with!(|counts| counts.passing_tests));
    let failing_tests = Signal::derive(move||with!(|counts| counts.failing_tests));
    let status = Signal::derive(move||with!(|invocation| invocation.status.clone()));
    view! {
        <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center divide-x">
            <div>
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
    }}
}
