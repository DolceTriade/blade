use leptos::*;
use leptos_router::*;

use crate::components::card::Card;
use crate::components::testsummary::TestSummary;

#[server]
pub async fn get_artifact(_uri: String) -> Result<Vec<u8>, ServerFnError> {
    Ok(vec![])
}

#[allow(dead_code)]
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

#[derive(PartialEq, Params)]
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

    provide_context(test);
    provide_context(test_run);

    {
        move || {
            with!(|test| match test {
                Ok(_test) => view! {
                    <div class="flex flex-col">
                    <Card>
                        <TestSummary/>
                    </Card>

                    <div class="h-[80vh] flex items-start justify-start justify-items-center">
                        <Card class="h-full w-1/4 max-w-1/4 md:max-w-xs p-0 m-0 flex-1 overflow-x-auto overflow-auto">
                            List
                        </Card>
                        <Card class="h-full w-3/4 p-1 m-1 flex-1 overflow-x-auto overflow-auto">
                            Output
                        </Card>
                    </div>
                    </div>
                },
                Err(e) => view! {
                    <div>
                        {format!("{e:#?}")}
                    </div>
                },
            })
        }
    }
}
