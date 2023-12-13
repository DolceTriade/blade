use leptos::*;
use leptos_router::*;
use state;

use crate::routes::summary::Summary;

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[server]
pub async fn get_invocation(uuid: String) -> Result<state::InvocationResults, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let map = global.sessions.lock().await;
    if let Some(invocation) = map.get(&uuid) {
        return Ok(invocation.lock().await.results.clone());
    }
    return Err(ServerFnError::ServerError(format!(
        "Invocation {uuid} not found"
    )));
}

#[derive(PartialEq, Params)]
struct InvocationParams {
    id: Option<String>,
}

#[component]
pub fn Invocation() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let invocation = create_rw_signal(state::InvocationResults::default());
    provide_context(invocation);
    let load_invocation = move|id: String| async move {
        get_invocation(id).await.map(|i| {
            invocation.set(i);
        })
    };
    let res = create_resource(
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
    let local = create_local_resource(
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

    let cancel_or = create_local_resource(
        move || (),
        move |_| async move {
            set_interval_with_handle(
                move || {
                    local.refetch();
                },
                std::time::Duration::from_secs(5),
            )
            .ok()
        },
    );
    create_render_effect(move|_| {
        with!(|invocation| {
            match invocation.status {
                state::Status::Success | state::Status::Fail => {
                    cancel_or
                        .map(|c| {
                            if let Some(cancel) = c {
                                cancel.clear()
                            }
                        });
                }
                _ => {}
            }
        });
    });

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || match res.get() {
                None => view! { <div>"Loading..."</div> }.into_view(),
                Some(i_or) => {
                    match i_or {
                        Ok(_) => {
                            view! {
                                <Summary />
                            }
                        }
                        Err(e) => view! { <div>{format!("{:#?}", e)}</div> }.into_view(),
                    }
                }
            }}

        </Transition>
    }
}
