use leptos::*;
use crate::components::statusicon::StatusIcon;

#[allow(non_snake_case)]
#[component]
pub fn SummaryHeader(
) -> impl IntoView
{
    view! { <StatusIcon class="h-8" status=state::Status::Success.into()/> }
}
