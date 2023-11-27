use crate::components::nav::Nav;
use crate::components::card::Card;
use crate::components::statusicon::StatusIcon;
use crate::components::list::*;
use crate::components::shellout::ShellOut;
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
        return Ok(invocation.lock().await.results.clone());
    }
    return Err(ServerFnError::ServerError("mistake".into()))
}

#[derive(PartialEq, Params)]
struct InvocationParams {
    id: Option<String>,
}

#[component]
pub fn Invocation() -> impl IntoView {
    let params = use_params::<InvocationParams>();
    let (invocation, set_invocation) = create_signal(None);
    spawn_local(async move {
        let id =  params.with(|params| {
            params
                .as_ref()
                .map(|params| params.id.clone())
                .unwrap_or_default()
                .unwrap_or_default()
        });

       if let Ok(i) = get_invocation(id).await {
            set_invocation.set(Some(i));
       }
    });

    view! {
        {move || match invocation.get() {
            None => view! { <div>"Not found"</div> },
            Some(i) => {
                view! {
                    <div>
                        <Card>{i.success}</Card>
                        <Card>
                            <List>

                                {{
                                    i.targets
                                        .into_iter()
                                        .map(|t| {
                                            view! {
                                                <ListItem>
                                                    <div class="flex items-center justify-start"><span><StatusIcon class="h-4" success=t.success/></span><span class="pl-4">{t.name.clone()}</span></div>
                                                </ListItem>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                }}

                            </List>
                        </Card>
                        <div><Card><ShellOut text={i.output.into()}/></Card></div>
                    </div>
                }
            }
        }}
    }
}
