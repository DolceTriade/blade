use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::hooks::use_query;
use leptos_router::hooks::use_location;
use leptos_router::NavigateOptions;
use leptos_router::components::Redirect;

use crate::components::card::Card;
use crate::components::shellout::ShellOut;
use crate::components::testartifactlist::TestArtifactList;
use crate::components::testresults::TestResults;
use crate::components::testrunlist::TestRunList;
use crate::components::testsummary::TestSummary;

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[server]
pub async fn get_artifact(uri: String) -> Result<Vec<u8>, ServerFnError<String>> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let parsed =
        url::Url::parse(&uri).map_err(|e| ServerFnError::<String>::ServerError(format!("{e:#?}")))?;
    match parsed.scheme() {
        "file" => {
            if !global.allow_local {
                return Err(ServerFnError::ServerError("not implemented".to_string()));
            }
            let path = parsed
                .to_file_path()
                .map_err(|e| ServerFnError::<String>::ServerError(format!("{e:#?}")))?;
            std::fs::read(path).map_err(|_| ServerFnError::<String>::ServerError("bad path".into()))
        }
        "bytestream" | "http" | "https" => {
            global
                .bytestream_client
                .download_file(&uri)
                .await
                .map_err(|e| ServerFnError::ServerError(format!("failed to get artifact: {e}")))
        }
        _ => Err(ServerFnError::ServerError("not implemented".to_string())),
    }
}

fn get_run(
    run: &Option<i32>,
    shard: &Option<i32>,
    attempt: &Option<i32>,
    runs: &Vec<state::TestRun>,
) -> (i32, i32, i32) {
    let run = run.unwrap_or(0);
    let shard = shard.unwrap_or(0);
    let attempt = attempt.unwrap_or(0);

    if run == 0 {
        return (runs[0].run, runs[0].shard, runs[0].attempt);
    }

    let mut best_candidate: Option<&state::TestRun> = None;
    for r in runs {
        if run != r.run {
            continue;
        }
        if best_candidate.is_none() {
            best_candidate = Some(r);
        }
        if shard == 0 {
            return (r.run, r.shard, r.attempt);
        }
        if shard == r.shard {
            best_candidate = Some(r);
        }
        if shard != r.shard {
            continue;
        }
        if attempt == 0 || attempt == r.attempt {
            return (r.run, r.shard, r.attempt);
        }
    }
    match best_candidate {
        None => (runs[0].run, runs[0].shard, runs[0].attempt),
        Some(r) => (r.run, r.shard, r.attempt),
    }
}

#[derive(PartialEq, Params, Debug)]
struct TestParams {
    target: Option<String>,
    run: Option<i32>,
    shard: Option<i32>,
    attempt: Option<i32>,
}

#[allow(non_snake_case)]
#[component]
pub fn Test() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let params = use_query::<TestParams>();
    tracing::info!("Got params: {:#?}", params);
    let test = Memo::new(move |_| match &*params.read() {
        Ok(params) => match &params.target {
            Some(target) => {
                if let Some(test) = invocation.read().tests.get(target) {
                    return Ok(test.clone());
                }
                return Err(format!("{} not found", target).to_string());
            }
            None => {
                return Err("No target specified in URL".to_string());
            }
        },
        Err(e) => return Err(format!("No target specified in the URL: {e}").to_string()),
    });
    let run = Memo::new(move |_| {
        params.read().as_ref().ok().and_then(|params| params.run)
    });
    let shard = Memo::new(move |_| {
        params.read().as_ref().ok().and_then(|params| params.shard)
    });
    let attempt = Memo::new(move |_| {
        params.read().as_ref().ok().and_then(|params| params.attempt)
    });

    let test_run = Memo::new(move|_| {
        let run = run.read().clone();
        let shard = shard.read().clone();
        let attempt = attempt.read().clone();
        let test = test.read();
        let test = test.as_ref();
        if run.is_none() || shard.is_none() || attempt.is_none() {
            return Option::None;
        }
        if test.is_err() {
            return Option::None;
        }
        let run = run.unwrap();
        let shard = shard.unwrap();
        let attempt = attempt.unwrap();
        let test = test.as_ref().unwrap();
        for test_run in &test.runs {
            if test_run.run == run && test_run.shard == shard && test_run.attempt == attempt {
                return Some(test_run.clone());
            }
        }
        Option::None
    });

    let test_xml = LocalResource::new(
        move || {
        let uri = test_run.with(|tr| tr.as_ref().and_then(|tr| tr.files.get("test.xml").map(|a| a.uri.clone())));
        async move {
            match uri {
                None => None,
                Some(uri) => get_artifact(uri.to_string())
                    .await
                    .ok()
                    .as_ref()
                    .and_then(|v| {
                        let c = std::io::Cursor::new(v);
                        junit_parser::from_reader(c).ok()
                    }),
            }
        }},
    );
    let test_out = Resource::new(
        move || {
            test_run.read()
                .as_ref()
                .and_then(|test_run| test_run.files.get("test.log").map(|a| a.uri.clone()))
        },
        move |uri| async move {
            match uri {
                None => None,
                Some(uri) => get_artifact(uri.to_string())
                    .await
                    .ok()
                    .as_ref()
                    .map(|v| String::from_utf8_lossy(v).to_string()),
            }
        },
    );
    let test_xml_signal = RwSignal::new(None);
    provide_context(test);
    provide_context(test_run);
    provide_context(test_xml_signal);

    {
        move || {
            match *test_run.read() {
                Some(_) => {
                    test_xml_signal.set(test_xml.get());
                    view! {
                        <div class="flex flex-col">
                            <Card class="p-0 m-0">
                                <TestSummary/>
                            </Card>

                            <div class="h-[80vh] flex items-start justify-start justify-items-center">
                                <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-0 m-0 flex-1 overflow-x-auto overflow-auto">
                                    <TestRunList/>
                                </Card>
                                <Card class="h-full w-3/4 p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                                    <TestResults/>
                                    <Suspense fallback=move || {
                                        view! { <div>Loading...</div> }
                                    }>
                                        {move || match test_out.get() {
                                            Some(Some(s)) => {
                                                view! {
                                                    <div>
                                                        <ShellOut text=s/>
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            _ => view! { <div>No test output</div> }.into_any(),
                                        }}

                                    </Suspense>
                                    <TestArtifactList/>
                                </Card>
                            </div>
                        </div>
                    }.into_any()
                }
                None => view! {
                    <div>
                        {move || {
                            if let Ok(test) = test.read().as_ref() {
                                if test.runs.is_empty() {
                                    return view! { <div>RIP</div> }.into_any();
                                }
                                let (r, s, a) = get_run(
                                    &run.read(),
                                    &shard.read(),
                                    &attempt.read(),
                                    &test.runs,
                                );
                                let mut q = use_location().query.get();
                                let path = use_location().pathname;
                                q.replace("run", r.to_string());
                                q.replace("shard", s.to_string());
                                q.replace("attempt", a.to_string());
                                view! {
                                    <Redirect
                                        path=format!("{}{}", path.get(), q.to_query_string())
                                        options=NavigateOptions {
                                            replace: true,
                                            ..Default::default()
                                        }
                                    />
                                }
                                    .into_any()
                            } else {
                                view! { <div>RIP</div> }.into_any()
                            }
                        }}

                    </div>
                }.into_any(),
            }
        }
    }
}
