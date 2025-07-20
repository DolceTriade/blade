#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::either::EitherOf3;
// use anyhow::anyhow;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{hooks::use_params, nested_router::Outlet, params::Params};

#[cfg(feature = "ssr")]
pub(crate) fn internal_err<T: std::fmt::Display>(e: T) -> ServerFnError {
    ServerFnError::ServerError(format!("Invocation {e} not found"))
}

#[server]
pub async fn get_invocation(uuid: String) -> Result<state::InvocationResults, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let mut db = global.db_manager.get().map_err(internal_err)?;
    db.get_invocation(&uuid).map_err(internal_err)
}

#[derive(PartialEq, Params)]
pub(crate) struct InvocationParams {
    pub(crate) id: Option<String>,
}

#[allow(non_snake_case)]
#[component]
pub fn Invocation() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let invocation = RwSignal::new(state::InvocationResults::default());
    let (loaded, set_loaded) = signal(false);
    let (error, set_error) = signal(None);
    provide_context(invocation);
    let load_invocation = move |id: String| async move { get_invocation(id).await };
    let res = LocalResource::new(move || {
        let id = params.with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default());
        async move {
            match id {
                None => Err("no id".to_string()),
                Some(id) => load_invocation(id)
                    .await
                    .map_err(|e| format!("failed to get invocation: {e:#?}")),
            }
        }
    });
    Effect::new(move || {
        match (*res.read()).as_ref() {
            None => {
                return;
            },
            Some(Err(e)) => {
                set_error(Some(format!("{e:#?}")));
                set_loaded(true);
                return;
            },
            Some(Ok(inv)) => {
                if !*loaded.read() {
                    set_loaded(true);
                }
                let done = matches!(
                    inv.status,
                    state::Status::Success
                        | state::Status::Fail
                        | state::Status::Skip
                        | state::Status::Unknown
                );
                invocation.set(inv.clone());
                if done {
                    return;
                }
            },
        }
        set_timeout(move || res.refetch(), std::time::Duration::from_secs(2));
    });

    view! {
        <Title text=move || {
            params
                .with(|p| p.as_ref().map(|p| p.id.clone().unwrap_or_default()).unwrap_or_default())
        } />
        {move || {
            if *loaded.read() {
                match error.read().as_ref() {
                    None => EitherOf3::A(view! { <Outlet /> }),
                    Some(e) => {
                        EitherOf3::B(
                            view! {
                                <div>
                                    <pre>{e.clone()}</pre>
                                </div>
                            },
                        )
                    }
                }
            } else {
                EitherOf3::C(
                    view! {
                        <div class="flex flex-col place-content-center place-items-center w-screen h-screen">
                            <img
                                class="w-64 h-64 text-gray-200 animate-spin fill-blue-600 self-center"
                                src="/assets/logo.svg"
                            />
                            <p>Loading...</p>
                        </div>
                    },
                )
            }
        }}
    }
}
