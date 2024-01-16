use std::io::{prelude::Read, Cursor};

use leptos::*;
use leptos_router::*;

use crate::components::shellout::ShellOut;

#[derive(PartialEq, Params, Debug, Clone)]
struct ArtifactParams {
    uri: Option<String>,
    zip: Option<String>,
}

fn stringify(e: impl std::fmt::Debug) -> String {
    format!("{e:#?}")
}

#[component]
pub fn Artifact() -> impl IntoView {
    let params = use_query::<ArtifactParams>();
    let artifact = create_local_resource(
        move || params.get(),
        move |params| async move {
            let uri = params
                .as_ref()
                .map_err(stringify)?
                .uri
                .clone()
                .ok_or("missing params".to_string())?;
            let zip = params.as_ref().map_err(stringify)?.zip.as_ref();
            let bytes = crate::routes::test::get_artifact(uri)
                .await
                .map_err(stringify)?;
            if let Some(zip) = zip {
                let cur = Cursor::new(bytes);
                let mut arc = zip::ZipArchive::new(cur).map_err(stringify)?;
                let mut file = arc.by_name(zip).map_err(stringify)?;
                let mut out = "".to_string();
                file.read_to_string(&mut out).map_err(stringify)?;
                log::info!("{}", out);
                return Ok::<String, String>(out);
            }
            Ok(String::from_utf8_lossy(&bytes).to_string())
        },
    );
    view! {
        <div class="h-[80vh] flex items-start justify-start justify-items-center overflow-auto overflow-x-auto">
            <Suspense fallback=move || view! { <div>Loading...</div> }>
                <ShellOut text=match artifact.get() {
                    Some(Ok(t)) => t,
                    Some(Err(t)) => t,
                    None => "RIP".into(),
                }/>
            </Suspense>
        </div>
    }
}
