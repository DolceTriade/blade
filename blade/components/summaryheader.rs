use crate::components::statusicon::StatusIcon;
use leptos::*;
use std::rc::Rc;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: usize, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display,
{
    view! {
        <div class="pl-4 pr-4"><span class="text-m">{num.to_string()}</span><span class="text-xs">{format!("{}{}", suffix, if num != 1 {"s"} else {""})}</span></div>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn SummaryHeader() -> impl IntoView {
    let invocation = use_context::<Rc<state::InvocationResults>>();
    invocation.map(move |inv| {
        let num_targets = inv.targets.len();
        let mut passing_targets:usize = 0;
        let mut failing_targets:usize = 0;
        inv
            .targets
            .values().for_each(|t| match t.status {
                state::Status::Success => passing_targets += 1,
                state::Status::Fail => failing_targets += 1,
                _ => {},
            });
        let num_tests = inv.tests.len();
        let passing_tests = inv.tests.values().filter(|v| v.success).count();
        let failing_tests = num_tests - passing_tests;

        view! {
            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center divide-x">
                <div>
                    <StatusIcon class="h-8 w-8" status={inv.status.clone().into()}/>
                </div>
                <SummaryItem num={num_targets} suffix="Total Target" />
                <SummaryItem num={passing_targets} suffix="Passing Target" />
                {(failing_targets > 0)
                    .then(|| {
                        view! {
                            <SummaryItem num={failing_targets} suffix="Failing Target" />
                        }
                    })}
                {(num_tests > 0)
                    .then(|| {
                        view! {
                            <SummaryItem num={num_tests} suffix="Total Test" />
                        }
                    })}
                {(passing_tests > 0)
                    .then(|| {
                        view! {
                            <SummaryItem num={passing_tests} suffix="Passing Test" />
                        }
                    })}
                {(failing_tests > 0)
                    .then(|| {
                        view! {
                            <SummaryItem num={failing_tests} suffix="Failing Test" />
                        }
                    })}
            </div>
        }
    })
}
