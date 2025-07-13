// Shared types and functions used across multiple crates
#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::{prelude::*, server_fn::ServerFnError};
#[cfg(feature = "ssr")]
use state::Global;

#[server]
pub async fn get_artifact(uri: String) -> Result<Vec<u8>, ServerFnError<String>> {
    let global: Arc<Global> = use_context::<Arc<Global>>().unwrap();
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

#[server]
pub async fn search_test_names(
    pattern: String,
    limit: Option<usize>,
) -> Result<Vec<String>, ServerFnError<String>> {
    let global: Arc<Global> = use_context::<Arc<Global>>().unwrap();
    let mut db = global
        .db_manager
        .get()
        .map_err(|e| ServerFnError::<String>::ServerError(format!("failed to get db: {e}")))?;

    let search_limit = limit.unwrap_or(10).min(50); // Cap at 50 results
    db.search_test_names(&pattern, search_limit).map_err(|e| {
        ServerFnError::<String>::ServerError(format!("failed to search test names: {e}"))
    })
}
