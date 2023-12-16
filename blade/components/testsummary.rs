use leptos::*;

use crate::components::statusicon::StatusIcon;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + std::fmt::Display + std::cmp::Eq + std::hash::Hash + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move||num.get().to_string()}</span>
            <span class="text-xs">{move||format!("{}{}", suffix, if num.get() != 1 { "s" } else { "" })}</span>
        </div>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn TestSummary() -> impl IntoView
where
{
    let test = expect_context::<Memo<Result<state::Test, String>>>();
    let _run = expect_context::<Memo<Option<state::TestRun>>>();
    view! {
        <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center divide-x">
        {move|| {
            with!(|test| test.as_ref().map(|test| view! {
                <div><StatusIcon class="h-8 w-8" status=test.status.into() /></div>
                <div class="pl-4"><b>{test.name.clone()}</b></div>
            }.into_view()).ok().unwrap_or_default())
        }}
        </div>
    }
}
