use leptos::*;
use leptos_router::*;

#[derive(PartialEq, Params)]
struct TestParams {
    target: Option<String>,
}

#[allow(non_snake_case)]
#[component]
pub fn Test() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let params = use_query::<TestParams>();
    let test = create_memo(move|_| {
        with!(|invocation, params| {
            match params {
                Ok(params) => {
                    match &params.target {
                        Some(target) => {
                            if let Some(test) = invocation.tests.get(target) {
                                return Ok(test.clone());
                            }
                            return Err(format!("{} not found", target).to_string());
                        }
                        None => {
                            return Err("No target specified in URL".to_string());
                        }
                    }
                }
                Err(e) => return Err(format!("No target specified in the URL: {e}").to_string()),
            }
        })
    });

    {
        move || {
            with!(|test| match test {
                Ok(test) => view!{
                    <div>
                        {format!("{test:#?}")}
                    </div>
                },
                Err(e) => view!{
                    <div>
                        {format!("{e:#?}")}
                    </div>
                },
            })
        }
    }
}
