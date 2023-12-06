use leptos::*;
use leptos_meta::*;
use state::InvocationResults;
use std::string::ToString;
use crate::components::statusicon::StatusIcon;
use crate::components::list::*;

#[component]
pub fn TargetList(
    invocation: Resource<String, Result<InvocationResults, ServerFnError>>,
) -> impl IntoView
{
    view!{<div></div>}
}