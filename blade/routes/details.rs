use std::collections::HashMap;
#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::prelude::*;

use crate::components::{accordion::*, card::Card, list::*, summaryheader::SummaryHeader};

#[server]
pub async fn get_options(uuid: String) -> Result<state::BuildOptions, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let mut db = global
        .db_manager
        .get()
        .map_err(crate::routes::invocation::internal_err)?;
    db.get_options(&uuid)
        .map_err(crate::routes::invocation::internal_err)
}

#[allow(non_snake_case)]
#[component]
pub fn Details() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let res = Resource::new(
        move || invocation.with(|invocation| invocation.id.clone()),
        get_options,
    );

    view! {
        <div class="flex flex-col m-1 p-1 dark:bg-gray-800">
            <Card class="p-3 m-2">
                <SummaryHeader />
            </Card>

            <div class="h-[73dvh] flex items-start justify-start justify-items-center overflow-auto">
                <Card class="p-3 m-2 max-w-full w-full">
                    <Suspense fallback=move || {
                        view! { <div>Loading...</div> }
                    }>
                        {move || {
                            res.with(|res| match res {
                                None => view! { <div>Loading...</div> }.into_any(),
                                Some(Ok(opts)) => {
                                    let opts = opts.clone();
                                    view! {
                                        <Accordion>
                                            {(!opts.build_metadata.is_empty())
                                                .then(move || {
                                                    view! {
                                                        <AccordionItem
                                                            hide=false
                                                            header=move || {
                                                                view! { <h3>Build Metadata</h3> }
                                                            }
                                                        >

                                                            <BuildMetadata md=opts.build_metadata />
                                                        </AccordionItem>
                                                    }
                                                })}
                                            <AccordionItem
                                                hide=false
                                                header=move || {
                                                    view! { <h3>Explicit Command Line</h3> }
                                                }
                                            >

                                                <OptionsList opts=opts.explicit_cmd_line />
                                            </AccordionItem>
                                            <AccordionItem
                                                hide=false
                                                header=move || {
                                                    view! { <h3>Command Line</h3> }
                                                }
                                            >

                                                <OptionsList opts=opts.cmd_line />
                                            </AccordionItem>
                                            <AccordionItem
                                                hide=false
                                                header=move || {
                                                    view! { <h3>Unstructured Command Line</h3> }
                                                }
                                            >

                                                <OptionsList opts=opts.unstructured />
                                            </AccordionItem>
                                            <AccordionItem
                                                hide=false
                                                header=move || {
                                                    view! { <h3>Explicit Startup Command Line</h3> }
                                                }
                                            >

                                                <OptionsList opts=opts.explicit_startup />
                                            </AccordionItem>
                                            <AccordionItem
                                                hide=false
                                                header=move || {
                                                    view! { <h3>Startup Command Line</h3> }
                                                }
                                            >

                                                <OptionsList opts=opts.startup />
                                            </AccordionItem>

                                        </Accordion>
                                    }
                                        .into_any()
                                }
                                Some(Err(e)) => view! { <div>{format!("{e:#?}")}</div> }.into_any(),
                            })
                        }}

                    </Suspense>
                </Card>
            </div>
        </div>
    }
}

#[component]
fn OptionsList(opts: Vec<String>) -> impl IntoView {
    view! {
        <List>
            <For
                each=move || opts.clone()
                key=move |o| o.clone()
                children=move |o| {
                    view! {
                        <ListItem hide=Signal::derive(|| false)>
                            <span class="text-sm font-mono">{o.clone()}</span>
                        </ListItem>
                    }
                }
            />

        </List>
    }
}

#[component]
fn BuildMetadata(md: HashMap<String, String>) -> impl IntoView {
    view! {
        <List>
            <For
                each=move || md.clone()
                key=move |o| o.clone()
                children=move |o| {
                    view! {
                        <ListItem hide=Signal::derive(|| false)>
                            <span class="text-sm font-mono">
                                {o.0.clone()}=> {linkify(o.1.clone())}
                            </span>
                        </ListItem>
                    }
                }
            />

        </List>
    }
}

fn linkify(link: String) -> impl IntoView {
    if url::Url::parse(&link).is_ok() {
        let href = link.clone();
        println!("HREF = {href}");
        view! {
            <a href=href.clone() class="text-blue-500 underline">
                {link}
            </a>
        }
        .into_any()
    } else {
        view! { <span class="text-sm font-mono">{link}</span> }.into_any()
    }
}
