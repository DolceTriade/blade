use leptos::*;
use leptos_router::*;

use crate::components::list::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UndeclaredOutput {
    pub name: String,
    pub size: String,
    pub kind: String,
    pub uri: String,
}

#[allow(non_snake_case)]
#[component]
pub fn TestArtifactList() -> impl IntoView {
    let test_run = expect_context::<Memo<Option<state::TestRun>>>();
    let manifest = create_local_resource(
        move || test_run.with(|test_run| test_run.as_ref().map(|test_run| test_run.files.clone())),
        move |files| async move {
            let files = files?;
            let uri = files.get("test.outputs_manifest__MANIFEST")?.uri.clone();
            let zip_uri = &files.get("test.outputs__outputs.zip")?.uri;
            crate::routes::test::get_artifact(uri)
                .await
                .ok()
                .as_ref()
                .and_then(|v| {
                    let manifest = String::from_utf8_lossy(v);
                    let lines = manifest.split('\n').collect::<Vec<_>>();
                    let mut out = vec![];
                    for l in lines {
                        if l.is_empty() {
                            continue;
                        }
                        let items = l.split('\t').collect::<Vec<_>>();
                        let name = items.first()?;
                        let size = items.get(1)?;
                        let kind = items.get(2)?;
                        out.push(UndeclaredOutput {
                            name: name.to_string(),
                            size: size.to_string(),
                            kind: kind.to_string(),
                            uri: zip_uri.clone(),
                        });
                    }
                    if out.is_empty() {
                        return None;
                    }
                    Some(out)
                })
        },
    );

    view! {
        <Suspense>
            {move || match manifest.with(|manifest| manifest.as_ref().map(|o| o.is_some())) {
                Some(true) => {
                    view! {
                        <h1 class="font-bold text-lg">Undeclared Outputs</h1>
                        <List>
                            <For
                                each=move || {
                                    manifest.with(|manifest| manifest.clone().flatten().unwrap())
                                }

                                key=move |r| r.name.clone()
                                children=move |r| {
                                    let query = format!(
                                        "../artifact?{}",
                                        url_escape::encode_query(
                                            &format!("uri={}&zip={}", r.uri, r.name),
                                        ),
                                    );
                                    view! {
                                        <ListItem hide=Signal::derive(|| false)>
                                            <A href=query>
                                                {format!("{} -- ({} bytes)", r.name, r.size)}
                                            </A>
                                        </ListItem>
                                    }
                                }
                            />

                        </List>
                    }
                        .into_view()
                }
                _ => view! {}.into_view(),
            }}

        </Suspense>

        <h1 class="font-bold text-lg">Artifacts</h1>
        <List>
            <For
                each=move || {
                    test_run
                        .with(|test_run| {
                            test_run.as_ref().map(|tr| tr.files.clone()).unwrap_or_default()
                        })
                }

                key=move |r| r.1.uri.clone()
                children=move |r| {
                    let query = format!(
                        "../artifact?{}",
                        url_escape::encode_query(&format!("uri={}", r.1.uri)),
                    );
                    view! {
                        <ListItem hide=Signal::derive(|| false)>
                            <A href=query>{r.0.clone()}</A>
                        </ListItem>
                    }
                }
            />

        </List>
    }
}
