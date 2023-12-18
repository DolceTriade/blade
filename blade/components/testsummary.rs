use leptos::*;

use crate::components::statusicon::StatusIcon;

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + std::cmp::Eq + std::hash::Hash + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move || num.get().to_string()}</span>
            <span class="text-xs">
                {move || format!("{}{}", suffix, if num.get() != 1 { "s" } else { "" })}
            </span>
        </div>
    }
}

#[allow(non_snake_case)]
#[component]
fn RunSummary() -> impl IntoView {
    let run = expect_context::<Memo<Option<state::TestRun>>>();
    let _xml = expect_context::<Resource<Option<String>, Option<junit_parser::TestSuites>>>();
    view! {
        {move||with!(|run| run.as_ref().map(|run| view! {
            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center">
                <div>
                    <StatusIcon class="h-5 w-5" status=run.status.into() />
                </div>
                <div class="w-fit h-fit grid grid-rows-1 grid-flow-col items-center justify-center text-s">
                    <span class="pl-4">
                        {format!("Run #{}", run.run)}
                    </span>
                    <span class="pl-4">
                        {format!("Shard #{}", run.shard)}
                    </span>
                    <span class="pl-4">
                        {format!("Attempt #{}", run.attempt)}
                    </span>
                </div>
                <div class="pl-1 text-s">
                    {format!("in {:#?}", run.duration)}
                </div>
            </div>
        }.into_view()).unwrap_or_default())
    }}
}

#[allow(non_snake_case)]
#[component]
pub fn TestSummary() -> impl IntoView
where
{
    let test = expect_context::<Memo<Result<state::Test, String>>>();
    view! {
        <div class="w-screen h-fit grid grid-rows-2 grid-flow-col items-center justify-center divide-y">
            {move || {
                with!(|test| test.as_ref().ok().map(|test| view! {
                        <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center p-2">
                            <div>
                                <StatusIcon class="h-8 w-8" status=test.status.into() />
                            </div>
                            <div class="pl-4">
                                <b>{test.name.clone()}</b>
                            </div>
                            <div class="pl-1 text-s">
                                {format!("in {:#?}", test.duration)}
                            </div>
                        </div>
                        <div class="w-screen h-fit grid grid-rows-1 grid-flow-col items-center justify-center p-2">
                            <RunSummary />
                        </div>
                    }.into_view()).unwrap_or_default()
                )
            }}

        </div>
    }
}
