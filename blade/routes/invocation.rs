use crate::components::card::Card;
use crate::components::shellout::ShellOut;
use crate::components::statusicon::StatusIcon;
use crate::components::targetlist::TargetList;
use leptos::*;
use leptos_router::*;
use state;
use std::rc::Rc;

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[server]
pub async fn get_invocation(uuid: String) -> Result<state::InvocationResults, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let map = global.sessions.lock().await;
    if let Some(invocation) = map.get(&uuid) {
        log::info!("Sending {:#?}", invocation);
        return Ok(invocation.lock().await.results.clone());
    }
    return Err(ServerFnError::ServerError(
        format!("Invocation {uuid} not found"),
    ));
}

#[derive(PartialEq, Params)]
struct InvocationParams {
    id: Option<String>,
}

#[component]
pub fn Invocation() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let res = create_resource(
        move || {
            params.with(|p| {
                p.as_ref()
                    .map(|p| p.id.clone())
                    .unwrap_or_default()
                    .unwrap_or_default()
            })
        },
        move |id| async move { get_invocation(id).await },
    );

    let cancel_or = create_local_resource(
        move || (),
        move |_| async move {
            set_interval_with_handle(
                move || {
                    res.refetch();
                },
                std::time::Duration::from_secs(15),
            )
            .ok()
        },
    );

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || match res.get() {
                None => view! { <div>"Loading..."</div> }.into_view(),
                Some(i_or) => {
                    match i_or {
                        Ok(i) => {
                            provide_context(Rc::new(i.clone()));
                            match i.status {
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
                            view! {
                                <div>
                                    <Card>
                                        <StatusIcon status=i.status.into() class="h-4 w-4"/>
                                    </Card>

                                    <div class="flex items-start justify-start justify-items-center">
                                        <Card class="overflow-auto overflow-x-auto p-0 m-0 text-ellipsis">
                                            {TargetList()}
                                        </Card>
                                        <Card class="overflow-auto overflow-x-auto grow">
                                            <ShellOut text=i.output.into()/>
                                        </Card>
                                    </div>
                                </div>
                            }
                                .into_view()
                        }
                        Err(e) => view! { <div>{format!("{:#?}", e)}</div> }.into_view(),
                    }
                }
            }}

        </Transition>
    }
}
