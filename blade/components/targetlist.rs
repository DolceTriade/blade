use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::statusicon::StatusIcon;
use leptos::*;
use leptos_dom::helpers::event_target;
use state;
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
                    let span = s.unchecked_into::<web_sys::HtmlSpanElement>();
                    span.offset_top() - span.scroll_top()
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
                                            each=move || tests.tests.clone()
                                            key=|t| t.0.to_string()
                                            children=move |t| {
                                                view! {
                                                    <ListItem>
                                                        <div class="group flex items-center justify-start w-full">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=if t.1.success {
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
                                                                {t.1.name.clone()}
                                                            </span>
                                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                                {format!("{:#?}", t.1.duration)}
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
                                each=move || targets.targets.clone()
                                key=|t| t.0.to_string()
                                children=move |t| {
                                    view! {
                                        <ListItem>
                                            <div class="group flex items-center justify-start w-full">
                                                <span class="float-left">
                                                    <StatusIcon
                                                        class="h-4 w-4 max-w-fit"
                                                        status=t.1.status.clone().into()
                                                    />

                                                </span>
                                                <span
                                                    class="pl-4 max-w-3/4 float-left text-ellipsis overflow-hidden group-hover:overflow-visible group-hover:absolute group-hover:bg-slate-200 group-hover:w-fit group-hover:rounded-md"
                                                    on:mouseenter=hover
                                                >
                                                    {t.1.name.clone()}
                                                </span>
                                                <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                    {format_time(&t.1.start, t.1.end.as_ref())}

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
