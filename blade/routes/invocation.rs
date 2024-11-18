use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use std::sync::Arc;
use std::time::Duration;

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
    let res = Resource::new(
        move || {
            params.with(|p| {
                p.as_ref()
                    .map(|p| p.id.clone())
                    .unwrap_or_default()
                    .unwrap_or_default()
            })
        },
        load_invocation,
    );

    let local = Resource::local(
        move || {
            params.with(|p| {
                p.as_ref()
                    .map(|p| p.id.clone())
                    .unwrap_or_default()
                    .unwrap_or_default()
            })
        },
        load_invocation,
    );

    let refetch = Resource::local(
        move || (),
        move |_| async move {
            set_interval_with_handle(
                move || {
                    local.refetch();
                },
                Duration::from_secs(5),
            )
            .ok()
        },
    );

    create_effect(move |_| {
        local.with(move |i| {
            if let Some(Ok(i)) = i {
                invocation.set(i.clone());
            }
        });
        invocation.with_untracked(move |i| match i.status {
            state::Status::Success | state::Status::Fail => {
                refetch.map(|refetch| {
                    if let Some(refetch) = refetch {
                        refetch.clear();
                    }
                });
            }
            _ => {}
        });
    });

    view! {
        <Title text=move || {
            params
                .with(|p| p.as_ref().map(|p| p.id.clone().unwrap_or_default()).unwrap_or_default())
        }/>
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                res.with(|i| match i {
                    None => view! { <div>"Loading..."</div> }.into_view(),
                    Some(Ok(i)) => {
                        invocation.set(i.clone());
                        view! { <Outlet/> }
                    }
                    Some(Err(e)) => view! { <div>{format!("{:#?}", e)}</div> }.into_view(),
                })
            }}

        </Transition>
    }
}
