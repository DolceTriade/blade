use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::statusicon::StatusIcon;

fn junit_status_to_status(s: junit_parser::TestStatus) -> state::Status {
    match s {
        junit_parser::TestStatus::Success => state::Status::Success,
        junit_parser::TestStatus::Error(_) => state::Status::Fail,
        junit_parser::TestStatus::Failure(_) => state::Status::Fail,
        junit_parser::TestStatus::Skipped(_) => state::Status::Skip,
    }
}

fn sort_runs(runs: &[state::TestRun]) -> Vec<state::TestRun> {
    let mut runs = runs.to_owned();
    runs.sort_unstable_by(|a, b| {
        if a.run != b.run {
            return a.run.cmp(&b.run);
        }
        if a.shard != b.shard {
            return a.shard.cmp(&b.shard);
        }
        a.attempt.cmp(&b.attempt)
    });
    runs
}

type TestListItem = (String, String, junit_parser::TestStatus, f64);

fn status_weight(s: &junit_parser::TestStatus) -> u8 {
    match s {
        junit_parser::TestStatus::Error(_) => 1,
        junit_parser::TestStatus::Failure(_) => 1,
        junit_parser::TestStatus::Skipped(_) => 2,
        junit_parser::TestStatus::Success => 3,
    }
}

fn sort_tests(cases: &[TestListItem]) -> Vec<TestListItem> {
    let mut cases = cases.to_owned();
    cases.sort_unstable_by(|a, b| {
        let a_s = status_weight(&a.2);
        let b_s = status_weight(&b.2);
        if a_s != b_s {
            return a_s.cmp(&b_s);
        }
        a.1.cmp(&b.1)
    });
    cases
}

#[allow(non_snake_case)]
#[component]
pub fn TestRunList() -> impl IntoView {
    let test = expect_context::<Memo<Result<state::Test, String>>>();
    let xml = expect_context::<Resource<Option<String>, Option<junit_parser::TestSuites>>>();
    let hover = move |e: leptos::ev::MouseEvent| {
        let el = event_target::<web_sys::HtmlSpanElement>(&e);
        el.parent_element()
            .map(|s| {
                let body = document().body().unwrap().get_bounding_client_rect();
                let span = s.unchecked_into::<web_sys::HtmlSpanElement>();
                span.get_bounding_client_rect().y() - body.y()
            })
            .map(|t| el.set_attribute("style", &format!("top: {}px", t)).ok());
    };

    view! {
        <Accordion>
            {move || {
                test.with(|test| test.as_ref().map(|test| test.runs.len() > 1).unwrap_or(false))
                    .then(move || {
                        view! {
                            <AccordionItem header=move || view! { <h3>Runs</h3> }>
                                <List>
                                    <For
                                        each=move || {
                                            with!(
                                                move | test | sort_runs(& test.as_ref().unwrap().runs)
                                            )
                                        }

                                        key=move |r| (r.run, r.shard, r.attempt)
                                        children=move |run| {
                                            let mut q = use_location().query.get();
                                            let path = use_location().pathname;
                                            let link = with!(
                                                move | path | { let runq = q.0.entry("run".to_string())
                                                .or_insert("".to_string()); * runq = run.run.to_string();
                                                let shard = q.0.entry("shard".to_string()).or_insert(""
                                                .to_string()); * shard = run.shard.to_string(); let attempt
                                                = q.0.entry("attempt".to_string()).or_insert(""
                                                .to_string()); * attempt = run.attempt.to_string();
                                                format!("{}{}", path, q.to_query_string()) }
                                            );
                                            view! {
                                                <ListItem hide=Signal::derive(|| false)>
                                                    <A href=link>
                                                        <div class="flex items-center justify-start w-full hover:bg-slate-100">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=run.status.into()
                                                                />

                                                            </span>
                                                            <div
                                                                class="pl-4 max-w-3/4 float-left overflow-hidden overflow-x-scroll whitespace-nowrap text-xs hover:overflow-visible hover:absolute hover:bg-slate-200 hover:w-fit hover:rounded-md"
                                                                on:mouseenter=hover
                                                            >
                                                                <span class="pl-4">{format!("Run #{}", run.run)}</span>
                                                                <span class="pl-4">{format!("Shard #{}", run.shard)}</span>
                                                                <span class="pl-4">
                                                                    {format!("Attempt #{}", run.attempt)}
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </A>
                                                </ListItem>
                                            }
                                        }
                                    />

                                </List>
                            </AccordionItem>
                        }
                    })
            }}
            <AccordionItem header=move || view! { <h3>Tests</h3> }>
                <Suspense fallback=move || {
                    view! { <div>Loading...</div> }
                }>
                    {move || {
                        xml.with(|x| match x.as_ref() {
                            Some(Some(_)) => {
                                view! {
                                    <List>
                                        <For
                                            each=move || {
                                                xml.with(|x| {
                                                    x.clone()
                                                        .flatten()
                                                        .as_ref()
                                                        .and_then(|x| x.suites.get(0))
                                                        .map(|c| {
                                                            c.cases
                                                                .iter()
                                                                .map(|i| (
                                                                    c.name.clone(),
                                                                    i.name.clone(),
                                                                    i.status.clone(),
                                                                    i.time,
                                                                ))
                                                                .collect::<Vec<_>>()
                                                        })
                                                        .map(|c| sort_tests(&c))
                                                        .unwrap_or_default()
                                                })
                                            }

                                            key=move |c| (c.0.clone(), c.1.clone())
                                            children=move |c| {
                                                view! {
                                                    <ListItem hide=Signal::derive(|| false)>
                                                        <div class="flex items-center justify-start w-full">
                                                            <span class="float-left">
                                                                <StatusIcon
                                                                    class="h-4 w-4 max-w-fit"
                                                                    status=junit_status_to_status(c.2).into()
                                                                />

                                                            </span>
                                                            <span
                                                                class="pl-4 max-w-3/4 float-left text-ellipsis whitespace-nowrap overflow-hidden hover:overflow-visible hover:absolute hover:bg-slate-200 hover:w-fit hover:rounded-md"
                                                                on:mouseenter=hover
                                                            >
                                                                {c.1.clone()}
                                                            </span>
                                                            <span class="text-gray-400 text-xs pl-2 ml-auto float-right">
                                                                {format!("{:.2}s", c.3)}
                                                            </span>
                                                        </div>

                                                    </ListItem>
                                                }
                                            }
                                        />

                                    </List>
                                }
                                    .into_view()
                            }
                            _ => view! { <div>Loading...</div> }.into_view(),
                        })
                    }}

                </Suspense>
            </AccordionItem>
        </Accordion>
    }
}
