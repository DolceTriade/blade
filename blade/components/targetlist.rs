use leptos::*;
use leptos_meta::*;
use state::InvocationResults;
use std::string::ToString;
use std::rc::Rc;
use state;
use crate::components::statusicon::StatusIcon;
use crate::components::list::*;

pub fn TargetList(
) -> impl IntoView
{
    let i = use_context::<Rc<state::InvocationResults>>();
    i.map(move |i| {
        view! {
            <div>
            <List>

                {{
                    i.targets
                        .clone()
                        .into_iter()
                        .map(|t| {
                            view! {
                                <ListItem>
                                    <div class="flex items-center justify-start">
                                        <span>
                                            <StatusIcon class="h-4" status=t.1.status.clone().into()/>

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
            </div>
        }}).unwrap_or(view!{<div/>})
}