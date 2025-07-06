#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};
use leptos_router::{
    NavigateOptions,
    components::Redirect,
    hooks::{use_location, use_query},
    params::Params,
};

use crate::components::{
    card::Card,
    shellout::ShellOut,
    testartifactlist::TestArtifactList,
    testresults,
    testrunlist::{SortOrder, SortType, TestRunList},
    testsummary::TestSummary,
};

#[server]
pub async fn get_artifact(uri: String) -> Result<Vec<u8>, ServerFnError<String>> {
    let global: Arc<state::Global> = use_context::<Arc<state::Global>>().unwrap();
    let parsed = url::Url::parse(&uri)
        .map_err(|e| ServerFnError::<String>::ServerError(format!("{e:#?}")))?;
    match parsed.scheme() {
        "file" => {
            if !global.allow_local {
                return Err(ServerFnError::ServerError("not implemented".to_string()));
            }
            let path = parsed
                .to_file_path()
                .map_err(|e| ServerFnError::<String>::ServerError(format!("{e:#?}")))?;
            std::fs::read(path).map_err(|_| ServerFnError::<String>::ServerError("bad path".into()))
        },
        "bytestream" | "http" | "https" => global
            .bytestream_client
            .download_file(&uri)
            .await
            .map_err(|e| ServerFnError::ServerError(format!("failed to get artifact: {e}"))),
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

fn flatten_test_suites(mut suites: junit_parser::TestSuites) -> junit_parser::TestSuites {
    if suites.suites.len() <= 1 {
        return suites;
    }

    let (first, others): (Vec<_>, Vec<_>) = suites
        .suites
        .into_iter()
        .enumerate()
        .partition(|(i, _)| *i == 0);
    let mut first = first.into_iter().next().unwrap().1;

    others.into_iter().for_each(|(_, other)| {
        first.cases.extend(other.cases);
        first.suites.extend(other.suites);
        first.time += other.time;
        first.tests += other.tests;
        first.errors += other.errors;
        first.failures += other.failures;
        first.skipped += other.skipped;

        if let Some(other_assertions) = other.assertions {
            first.assertions = Some(first.assertions.unwrap_or(0) + other_assertions);
        }

        if first.system_out.is_none() {
            first.system_out = other.system_out;
        } else if let Some(other_out) = other.system_out {
            first.system_out.as_mut().unwrap().push('\n');
            first.system_out.as_mut().unwrap().push_str(&other_out);
        }

        if first.system_err.is_none() {
            first.system_err = other.system_err;
        } else if let Some(other_err) = other.system_err {
            first.system_err.as_mut().unwrap().push('\n');
            first.system_err.as_mut().unwrap().push_str(&other_err);
        }
    });

    suites.suites = vec![first];
    suites
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
    let test = Memo::new(move |_| match &*params.read() {
        Ok(params) => match &params.target {
            Some(target) => {
                if let Some(test) = invocation.read().tests.get(target) {
                    return Ok(test.clone());
                }
                Err(format!("{target} not found").to_string())
            },
            None => Err("No target specified in URL".to_string()),
        },
        Err(e) => Err(format!("No target specified in the URL: {e}").to_string()),
    });
    let run = Memo::new(move |_| params.read().as_ref().ok().and_then(|params| params.run));
    let shard = Memo::new(move |_| params.read().as_ref().ok().and_then(|params| params.shard));
    let attempt = Memo::new(move |_| {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.attempt)
    });

    let test_run = Memo::new(move |_| {
        let run = *run.read();
        let shard = *shard.read();
        let attempt = *attempt.read();
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

    let test_xml = LocalResource::new(move || {
        let uri = test_run
            .read()
            .as_ref()
            .and_then(|tr| tr.files.get("test.xml").map(|a| a.uri.clone()));
        async move {
            match uri {
                None => None,
                Some(uri) => get_artifact(uri.to_string())
                    .await
                    .ok()
                    .as_ref()
                    .and_then(|v| {
                        let c = std::io::Cursor::new(v);
                        junit_parser::from_reader(c)
                            .inspect_err(|e| {
                                tracing::warn!("could not fetch test.xml: {e:#?}");
                            })
                            .ok()
                            .map(flatten_test_suites)
                    }),
            }
        }
    });
    let test_out = LocalResource::new(move || {
        let uri = test_run
            .read()
            .as_ref()
            .and_then(|test_run| test_run.files.get("test.log").map(|a| a.uri.clone()));
        async move {
            match uri {
                None => None,
                Some(uri) => get_artifact(uri.to_string())
                    .await
                    .ok()
                    .as_ref()
                    .map(|v| String::from_utf8_lossy(v).to_string()),
            }
        }
    });
    provide_context(test);
    provide_context(test_run);
    provide_context(test_xml);

    let (sort_by, set_sort_by) = signal(SortType::NoSort);
    let (sort_order, set_sort_order) = signal(SortOrder::Descending);
    let (hide_success, set_hide_success) = signal(false);

    {
        // This block is for the view! macro
        move || match *test_run.read() {
            Some(_) => Either::Left(view! {
                <div class="flex flex-col m-1 p-1 dark:bg-gray-800">
                    <Card class="flex p-3 m-2">
                        <TestSummary />
                    </Card>

                    <div class="h-[73dvh] flex items-start justify-start justify-items-center">
                        <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                            <TestRunList
                                sort_by=sort_by
                                set_sort_by=set_sort_by
                                sort_order=sort_order
                                set_sort_order=set_sort_order
                                hide_success=hide_success
                                set_hide_success=set_hide_success
                            />
                        </Card>
                        <Card class="h-full w-full max-w-full p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                            <testresults::TestResults
                                sort_by=sort_by
                                sort_order=sort_order
                                hide_success=hide_success
                            />
                            <Suspense fallback=move || {
                                view! { <div>Loading...</div> }
                            }>
                                {move || match test_out.get() {
                                    Some(Some(s)) => {
                                        Either::Left(
                                            view! {
                                                <div>
                                                    <ShellOut text=s />
                                                </div>
                                            },
                                        )
                                    }
                                    _ => Either::Right(view! { <div>No test output</div> }),
                                }}

                            </Suspense>
                            <TestArtifactList />
                        </Card>
                    </div>
                </div>
            }),
            None => Either::Right(view! {
                <div>
                    {move || match test.read().as_ref() {
                        Ok(test) => {
                            if test.runs.is_empty() {
                                return EitherOf3::A(view! { <div>RIP</div> });
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
                            EitherOf3::B(
                                view! {
                                    <Redirect
                                        path=format!("{}{}", path.get(), q.to_query_string())
                                        options=NavigateOptions {
                                            replace: true,
                                            ..Default::default()
                                        }
                                    />
                                },
                            )
                        }
                        Err(e) => EitherOf3::C(view! { <div>{e.to_string()}</div> }),
                    }}

                </div>
            }),
        }
    }
}
