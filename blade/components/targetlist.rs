use leptos::*;
use std::string::ToString;
use std::rc::Rc;
use state;
use crate::components::accordion::*;
use crate::components::statusicon::StatusIcon;
use crate::components::list::*;

fn format_time(start: &std::time::SystemTime, end: Option<&std::time::SystemTime>) -> String {
    if end.is_none() {
        return "".to_string();
    }
    let e = end.unwrap();
    e.duration_since(*start).map(|d| format!("{:#?}", d)).unwrap_or_default()
}

#[allow(non_snake_case)]
pub fn TargetList() -> impl IntoView
{
    let invocation = use_context::<Rc<state::InvocationResults>>();
    invocation.map(move |inv| {
        let tests = inv.clone();
        let targets = inv.clone();
        view! {
            <div>
                <Accordion>
                    <AccordionItem header=move || view! { <h3>Tests</h3> }>
                        <List>

                            {(!tests.tests.is_empty())
                                .then(move || {
                                    tests
                                        .tests
                                        .clone()
                                        .into_iter()
                                        .map(|t| {
                                            view! {
                                                <ListItem>
                                                    <div class="flex items-center justify-start overflow-x-auto">
                                                        <span>
                                                            <StatusIcon
                                                                class="h-4"
                                                                status=if t.1.success {
                                                                    state::Status::Success.into()
                                                                } else {
                                                                    state::Status::Fail.into()
                                                                }
                                                            />

                                                        </span>
                                                        <span class="pl-4">{t.1.name.clone()}</span>
                                                        <span class="text-gray-400 text-xs pl-2 ml-auto">
                                                            {format!("{:#?}", t.1.duration)}
                                                        </span>
                                                    </div>
                                                </ListItem>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                })}

                        </List>
                    </AccordionItem>

                    <AccordionItem header=move || view! { <h3>Targets</h3> }>
                        <List>

                            {targets
                                .targets
                                .clone()
                                .into_iter()
                                .map(|t| {
                                    view! {
                                        <ListItem>
                                            <div class="flex items-center justify-start overflow-x-auto">
                                                <span>
                                                    <StatusIcon class="h-4" status=t.1.status.clone().into()/>

                                                </span>
                                                <span class="pl-4">{t.1.name.clone()}</span>
                                                <span class="text-gray-400 text-xs pl-2 ml-auto">
                                                    {format_time(&t.1.start, t.1.end.as_ref())}

                                                </span>
                                            </div>
                                        </ListItem>
                                    }
                                })
                                .collect::<Vec<_>>()}

                        </List>
                    </AccordionItem>
                </Accordion>
            </div>
        }}).unwrap_or(view! { <div></div> })
}