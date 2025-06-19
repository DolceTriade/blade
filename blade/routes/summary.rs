use leptos::prelude::*;

use crate::components::{
    card::Card,
    shellout::ShellOut,
    summaryheader::SummaryHeader,
    targetlist::TargetList,
};

#[allow(non_snake_case)]
#[component]
pub fn Summary() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let (output, _) = slice!(invocation.output);

    view! {
        <div class="flex flex-col m-1 p-1 dark:bg-gray-800">
            <Card class="flex p-3 m-2">
                <SummaryHeader />
            </Card>

            <div class="flex items-start justify-start justify-items-center">
                <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-1 m-1 flex-1 overflow-x-auto">
                    {TargetList()}
                </Card>
                <Card class="h-full max-w-full w-full p-1 m-1 flex-1 overflow-x-auto">
                    <ShellOut text=output />
                </Card>
            </div>
        </div>
    }
}
