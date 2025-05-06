use std::io::{Cursor, prelude::Read};

use leptos::prelude::*;
use leptos_router::{hooks::use_query, params::Params};

use crate::components::shellout::ShellOut;

#[derive(PartialEq, Params, Debug, Clone)]
struct ArtifactParams {
    uri: Option<String>,
    zip: Option<String>,
}

fn stringify(e: impl std::fmt::Debug) -> String { format!("{e:#?}") }

#[component]
pub fn Artifact() -> impl IntoView {
    let params = use_query::<ArtifactParams>();
    let artifact = LocalResource::new(move || async move {
        let Ok(ArtifactParams { uri, zip }) = params.get() else {
            return Err("error parsing query string".into());
        };
        let Some(uri) = uri else {
            return Err("empty uri".into());
        };
        let bytes = crate::routes::test::get_artifact(uri)
            .await
            .map_err(stringify)?;
        if let Some(zip) = zip {
            let cur = Cursor::new(bytes);
            let mut arc = zip::ZipArchive::new(cur).map_err(stringify)?;
            let mut file = arc.by_name(&zip).map_err(stringify)?;
            let mut out = "".to_string();
            file.read_to_string(&mut out).map_err(stringify)?;
            return Ok::<String, String>(out);
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    });
    view! {
        <div class="h-[73dvh] flex items-start justify-start justify-items-center overflow-auto overflow-x-auto">
            <Suspense fallback=move || {
                view! { <div>Loading...</div> }
            }>
                {move || Suspend::new(async move {
                    let t: String = match artifact.await {
                        Ok(t) => t,
                        Err(t) => t,
                    };
                    view! { <ShellOut text=t /> }
                })}

            </Suspense>
        </div>
    }
}
