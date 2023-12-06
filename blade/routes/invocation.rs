use crate::components::card::Card;
use crate::components::list::*;
use crate::components::nav::Nav;
use crate::components::shellout::ShellOut;
use crate::components::statusicon::StatusIcon;
use ansi_to_html;
use leptos::*;
use leptos_router::*;
use log;
use state;
use std::sync::Arc;

#[server]
pub async fn get_invocation(uuid: String) -> Result<state::InvocationResults, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let mut map = global.sessions.lock().await;
    if let Some(invocation) = map.get(&uuid) {
        log::info!("Sending {:#?}", invocation);
        return Ok(invocation.lock().await.results.clone());
    }
    return Err(ServerFnError::ServerError(format!("Invocation {uuid} not found").into()));
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
        move |id| async move { 
            get_invocation(id).await
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
                            view! {
                                <div>
                                    <Card>
                                        <StatusIcon status=i.status.into() class="h-4 w-4"/>
                                    </Card>
                                    <Card>
                                        <List>

                                            {{
                                                i.targets
                                                    .into_iter()
                                                    .map(|t| {
                                                        view! {
                                                            <ListItem>
                                                                <div class="flex items-center justify-start">
                                                                    <span>
                                                                        <StatusIcon class="h-4" status=t.1.status.into()/>

                                                                    </span>
                                                                    <span class="pl-4">{t.1.name.clone()}</span>
                                                                    <span class="text-gray-400 text-xs pl-2 ml-auto">
                                                                        {format!(
                                                                            "{:#?}",
                                                                            (t.1.end.unwrap().duration_since(t.1.start).unwrap()),
                                                                        )}

                                                                    </span>
                                                                </div>
                                                            </ListItem>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            }}

                                        </List>
                                    </Card>
                                    <div>
                                        <Card>
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
