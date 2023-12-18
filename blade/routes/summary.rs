use leptos::*;

use crate::components::card::Card;
use crate::components::shellout::ShellOut;
use crate::components::summaryheader::SummaryHeader;
use crate::components::targetlist::TargetList;

#[component]
pub fn Summary() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let (output, _) = slice!(invocation.output);

    view! {
        <div class="flex flex-col">
            <Card>
                <SummaryHeader/>
            </Card>

            <div class="h-[80vh] flex items-start justify-start justify-items-center">
                <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-0 m-0 flex-1 overflow-x-auto overflow-auto">
                    {TargetList()}
                </Card>
                <Card class="h-full w-3/4 p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                    <ShellOut text=output/>
                </Card>
            </div>
        </div>
    }
}
