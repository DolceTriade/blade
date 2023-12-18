use leptos::*;
use leptos_router::*;

use crate::components::accordion::*;
use crate::components::list::*;
use crate::components::statusicon::StatusIcon;

#[allow(non_snake_case)]
#[component]
pub fn TestRunList() -> impl IntoView {
    let test = expect_context::<Memo<Result<state::Test, String>>>();
    let _xml = expect_context::<Resource<Option<String>, Option<junit_parser::TestSuites>>>();
    view! {
            <Accordion>
                {move||with!(|test|test.as_ref().map(|test|test.runs.len() > 1).unwrap_or(false)).then(move||view! {
                    <AccordionItem header=move || view! { <h3>Runs</h3> }>
                    <List>
                    <For
                        each=move||with!(move|test| test.as_ref().unwrap().runs.clone())
                        key=move|r|(r.run, r.shard, r.attempt)
                        children=move|run| {
                            let mut q = use_location().query.get();
                            let path = use_location().pathname;
                            let link = with!(move|path| {
                                let runq = q.0.entry("run".to_string()).or_insert("".to_string());
                                *runq = run.run.to_string();
                                let shard = q.0.entry("shard".to_string()).or_insert("".to_string());
                                *shard = run.shard.to_string();
                                let attempt = q.0.entry("attempt".to_string()).or_insert("".to_string());
                                *attempt = run.attempt.to_string();
                                format!("{}{}", path, q.to_query_string())
                            });

                            view! {
                            <ListItem hide=Signal::derive(||false)>
                            <A href=link>
                            <div class="flex items-center justify-start w-full hover:bg-slate-100">
                                <span class="float-left">
                                    <StatusIcon
                                        class="h-4 w-4 max-w-fit"
                                        status=run.status.into()
                                    />

                                </span>
                                <div
                                    class="label-name pl-4 max-w-3/4 float-left overflow-hidden overflow-x-scroll whitespace-nowrap text-xs"
                                >
                                    <span class="pl-4">
                                        {format!("Run #{}", run.run)}
                                    </span>
                                    <span class="pl-4">
                                        {format!("Shard #{}", run.shard)}
                                    </span>
                                    <span class="pl-4">
                                        {format!("Attempt #{}", run.attempt)}
                                    </span>
                                </div>
                            </div>
                        </A>
                        </ListItem>
                        }} />
                    </List>
                    </AccordionItem>
                })
            }
            </Accordion>
    }
}
