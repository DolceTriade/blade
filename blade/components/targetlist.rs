use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::statusicon::StatusIcon;
use leptos::*;
use leptos_dom::{document, helpers::event_target};
use state;
use std::collections::HashMap;
use std::rc::Rc;
use std::string::ToString;
use wasm_bindgen::JsCast;

fn format_time(start: &std::time::SystemTime, end: Option<&std::time::SystemTime>) -> String {
    if end.is_none() {
        return "".to_string();
    }
    let e = end.unwrap();
    e.duration_since(*start)
        .map(|d| format!("{:#?}", d))
        .unwrap_or_default()
}

fn status_weight(s: &state::Status) -> u8 {
    match s {
        state::Status::InProgress => 0,
        state::Status::Fail => 1,
        state::Status::Skip => 2,
        state::Status::Success => 3,
        state::Status::Unknown => 4,
    }
}

fn sorted_targets(targets: &HashMap<String, state::Target>) -> Vec<state::Target> {
    let mut vec = targets.values().collect::<Vec<_>>();
    vec.sort_unstable_by(|a, b| {
        let a_status = status_weight(&a.status);
        let b_status = status_weight(&b.status);
        if a_status != b_status {
            return a_status.partial_cmp(&b_status).unwrap();
        }
        a.name.partial_cmp(&b.name).unwrap()
    });
    vec.into_iter().cloned().collect::<Vec<_>>()
}

fn sorted_tests(tests: &HashMap<String, state::Test>) -> Vec<state::Test> {
    let mut vec = tests.values().collect::<Vec<_>>();
    vec.sort_unstable_by(|a, b| {
        if a.success == b.success {
            return a.name.partial_cmp(&b.name).unwrap()
        }
        match a.success {
            true => std::cmp::Ordering::Greater,
            false => std::cmp::Ordering::Less,
        }
    });
    vec.into_iter().cloned().collect::<Vec<_>>()
}

#[allow(non_snake_case)]
pub fn TargetList() -> impl IntoView {
    let invocation = use_context::<Rc<state::InvocationResults>>();
    invocation.map(move |inv| {
        let tests = inv.clone();
        let targets = inv.clone();
        let hover = move|e: leptos::ev::MouseEvent|{
            let el = event_target::<web_sys::HtmlSpanElement>(&e);
            el.next_element_sibling()
                .map(|s|{
                    let body = document().body().unwrap().get_bounding_client_rect();
                    let span = s.unchecked_into::<web_sys::HtmlSpanElement>();
                    span.get_bounding_client_rect().y() - body.y()
                })
                .map(|t|{ 
                    el.set_attribute("style", &format!("top: {}px", t)).ok()
                });
        };
        view! {
            <div>
                <Accordion>

                    {(!tests.tests.is_empty())
                        .then(move || {
                            view! {
                                <AccordionItem header=move || view! { <h3>Tests</h3> }>
                                    <List>
                                        <For
                                            each=move || sorted_tests(&tests.tests)
                                            key=|t| t.name.to_string()
                                            children=move |t| {
                                                view! {
                                                    <ListItem>
                                                        <div class="group flex items-center justify-start w-full">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=if t.success {
                                                                        state::Status::Success.into()
                                                                    } else {
                                                                        state::Status::Fail.into()
                                                                    }
                                                                />

                                                            </span>
                                                            <span
                                                                class="pl-4 max-w-3/4 float-left text-ellipsis overflow-hidden group-hover:overflow-visible group-hover:absolute group-hover:bg-slate-200 group-hover:w-fit group-hover:rounded-md"
                                                                on:mouseenter=hover
                                                            >
                                                                {t.name.clone()}
                                                            </span>
                                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                                {format!("{:#?}", t.duration)}
                                                            </span>
                                                        </div>
                                                    </ListItem>
                                                }
                                            }
                                        />

                                    </List>
                                </AccordionItem>
                            }
                        })}
                    <AccordionItem header=move || view! { <h3>Targets</h3> }>
                        <List>
                            <For
                                each=move || sorted_targets(&targets.targets)
                                key=|t| t.name.to_string()
                                children=move |t| {
                                    view! {
                                        <ListItem>
                                            <div class="group flex items-center justify-start w-full">
                                                <span class="float-left">
                                                    <StatusIcon
                                                        class="h-4 w-4 max-w-fit"
                                                        status=t.status.clone().into()
                                                    />

                                                </span>
                                                <span
                                                    class="pl-4 max-w-3/4 float-left text-ellipsis overflow-hidden group-hover:overflow-visible group-hover:absolute group-hover:bg-slate-200 group-hover:w-fit group-hover:rounded-md"
                                                    on:mouseenter=hover
                                                >
                                                    {t.name.clone()}
                                                </span>
                                                <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                    {format_time(&t.start, t.end.as_ref())}

                                                </span>
                                            </div>
                                        </ListItem>
                                    }
                                }
                            />

                        </List>
                    </AccordionItem>
                </Accordion>
            </div>
        }}).unwrap_or(view! { <div></div> })
}
