use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params;
use leptos_router::nested_router::Outlet;
use leptos_router::params::Params;

use anyhow::anyhow;

#[cfg(feature = "ssr")]
use std::sync::Arc;

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
struct InvocationParams {
    id: Option<String>,
}

#[allow(non_snake_case)]
#[component]
pub fn Invocation() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let invocation = RwSignal::new(state::InvocationResults::default());
    provide_context(invocation);
    let load_invocation = move |id: String| async move { get_invocation(id).await };
    let res = LocalResource::new(move || {
        let id = params.with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default());
        async move {
            match id {
                None => Err(anyhow!("no id")),
                Some(id) => load_invocation(id)
                    .await
                    .map_err(|e| anyhow!("failed to get invocation: {e:#?}")),
            }
        }
    });

    // TODO: Fix refetch

    view! {
        <Title text=move || {
            params
                .with(|p| p.as_ref().map(|p| p.id.clone().unwrap_or_default()).unwrap_or_default())
        } />
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                res.with(|i| match i {
                    None => view! { <div>"Loading..."</div> }.into_any(),
                    Some(i) => {
                        match i {
                            Ok(i) => {
                                invocation.set(i.clone());
                                view! { <Outlet /> }.into_any()
                            }
                            Err(e) => {
                                view! {
                                    <div>
                                        <pre>{format!("{e:#?}")}</pre>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }
                })
            }}

        </Transition>
    }
}
