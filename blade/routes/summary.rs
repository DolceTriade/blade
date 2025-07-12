#[cfg(feature = "ssr")]
use std::sync::Arc;

use components::{
    card::Card,
    shellout::ShellOut,
    summaryheader::SummaryHeader,
    targetlist::TargetList,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params;

use crate::invocation::InvocationParams;

#[cfg(feature = "ssr")]
pub(crate) fn internal_err<T: std::fmt::Display>(e: T) -> ServerFnError {
    ServerFnError::ServerError(format!("Invocation {e} not found"))
}

#[server]
pub async fn get_output(uuid: String) -> Result<String, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let mut db = global.db_manager.get().map_err(internal_err)?;
    db.get_progress(&uuid).map_err(internal_err)
}

#[allow(non_snake_case)]
#[component]
pub fn Summary() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let (output, set_output) = signal("".to_string());
    let output_res = LocalResource::new(move || {
        let id = params
            .with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default())
            .unwrap_or_default();
        async move {
            if id.is_empty() {
                return "".to_string();
            }
            match get_output(id).await {
                Ok(v) => v,
                Err(e) => format!("{e:#?}"),
            }
        }
    });
    Effect::new(move || {
        let out = output_res.read();
        match out.as_ref() {
            Some(out) => set_output(out.clone()),
            None => {
                if output.with(|o| o.is_empty()) {
                    set_output("Loading...".to_string());
                }
            },
        }
        let done = matches!(
            invocation.read_untracked().status,
            state::Status::Success | state::Status::Fail | state::Status::Skip
        );
        if !done {
            set_timeout(
                move || output_res.refetch(),
                std::time::Duration::from_secs(10),
            );
        }
    });

    view! {
        <div class="flex flex-col m-1 p-1 dark:bg-gray-800">
            <Card class="flex p-3 m-2">
                <SummaryHeader />
            </Card>

            <div class="h-[73dvh] flex items-start justify-start justify-items-center">
                <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                    {TargetList()}
                </Card>
                <Card class="h-full max-w-full w-full p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                    <ShellOut text=output />
                </Card>
            </div>
        </div>
    }
}
