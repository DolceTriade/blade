use leptos::*;
use leptos_meta::*;
use std::string::ToString;
use crate::components::statusicon::StatusIcon;

#[component]
pub fn SummaryHeader(
) -> impl IntoView
{
    view! { <StatusIcon class="h-8" status=state::Status::Success.into()/> }
}
