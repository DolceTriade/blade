use leptos::*;
use leptos_router::*;

use crate::components::card::Card;
use crate::components::shellout::ShellOut;
use crate::components::testresults::TestResults;
use crate::components::testrunlist::TestRunList;
use crate::components::testsummary::TestSummary;

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[server]
pub async fn get_artifact(uri: String) -> Result<Vec<u8>, ServerFnError> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let parsed =
        url::Url::parse(&uri).map_err(|e| ServerFnError::ServerError(format!("{e:#?}")))?;
    match parsed.scheme() {
        "file" => {
            if !global.allow_local {
                return Err(ServerFnError::ServerError("not implemented".to_string()));
            }
            let path = parsed
                .to_file_path()
                .map_err(|e| ServerFnError::ServerError(format!("{e:#?}")))?;
            return std::fs::read(path).map_err(|e| ServerFnError::ServerError(format!("{e:#?}")));
        }
        "bytestream" | "http" | "https" => {
            return global
                .bytestream_client
                .download_file(&uri)
                .await
                .map_err(|e| ServerFnError::ServerError(format!("failed to get artifact: {e}")));
        }
        _ => {
            Err(ServerFnError::ServerError("not implemented".to_string()))
        }
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
    let test = create_memo(move |_| {
        with!(|invocation, params| {
            match params {
                Ok(params) => match &params.target {
                    Some(target) => {
                        if let Some(test) = invocation.tests.get(target) {
                            return Ok(test.clone());
                        }
                        return Err(format!("{} not found", target).to_string());
                    }
                    None => {
                        return Err("No target specified in URL".to_string());
                    }
                },
                Err(e) => return Err(format!("No target specified in the URL: {e}").to_string()),
            }
        })
    });
    let run = create_memo(move |_| {
        with!(|params| { params.as_ref().ok().and_then(|params| params.run) })
    });
    let shard = create_memo(move |_| {
        with!(|params| { params.as_ref().ok().and_then(|params| params.shard) })
    });
    let attempt = create_memo(move |_| {
        with!(|params| { params.as_ref().ok().and_then(|params| params.attempt) })
    });

    let test_run = create_memo(move |_| {
        with!(|run, shard, attempt, test| {
            if run.is_none() || shard.is_none() || attempt.is_none() {
                return None;
            }
            if test.is_err() {
                return None;
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
            None
        })
    });

    let test_xml = create_local_resource(
        move || {
            with!(|test_run| test_run
                .as_ref()
                .and_then(|test_run| test_run.files.get("test.xml").map(|a| a.uri.clone())))
        },
        move |uri| async move {
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
        },
    );
    let test_out = create_resource(
        move || {
            with!(|test_run| test_run
                .as_ref()
                .and_then(|test_run| test_run.files.get("test.log").map(|a| a.uri.clone())))
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
    let test_xml_signal = create_rw_signal(None);
    provide_context(test);
    provide_context(test_run);
    provide_context(test_xml_signal);

    {
        move || {
            with!(|test_run, test_xml| match test_run {
                Some(_) => {
                    test_xml_signal.set(test_xml.clone());
                    view! {
                    <div class="flex flex-col">
                    <Card class="p-0 m-0">
                        <TestSummary/>
                    </Card>

                    <div class="h-[80vh] flex items-start justify-start justify-items-center">
                        <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-0 m-0 flex-1 overflow-x-auto overflow-auto">
                            <TestRunList />
                        </Card>
                        <Card class="h-full w-3/4 p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                        <TestResults/>
                        <Suspense
                        fallback=move||view!{<div>Loading...</div>}>
                            {move||match test_out.get() {
                                Some(Some(s)) => view!{ <div><ShellOut text={s} /></div> },
                                _ => view!{ <div>No test output</div> },
                            }}
                        </Suspense>
                        </Card>
                    </div>
                    </div>
                }},
                None => view! {
                    <div>
                        {move||with!(|test, run, shard, attempt| if let Ok(test) = test {
                            let (r, s, a) = get_run(run, shard, attempt, &test.runs);
                            let mut q = use_location().query.get();
                            let path = use_location().pathname;
                            with!(move|path| {
                                let run = q.0.entry("run".to_string()).or_insert("".to_string());
                                *run = r.to_string();
                                let shard = q.0.entry("shard".to_string()).or_insert("".to_string());
                                *shard = s.to_string();
                                let attempt = q.0.entry("attempt".to_string()).or_insert("".to_string());
                                *attempt = a.to_string();
                                view!{<Redirect path=format!("{}{}", path, q.to_query_string()) options={NavigateOptions {replace: true, ..Default::default()}}/>}
                            })
                        } else {
                            view!{<div> RIP </div>}.into_view()
                        })}
                    </div>
                },
            })
        }
    }
}
