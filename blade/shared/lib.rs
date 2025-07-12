// Shared types and functions used across multiple crates
#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::{prelude::*, server_fn::ServerFnError};
#[cfg(feature = "ssr")]
use state::Global;

/// Dark mode state shared between components and routes
#[derive(Clone, Copy, Debug)]
pub struct DarkMode(pub bool);

/// Dark mode utilities
pub mod darkmode {
    use cfg_if::cfg_if;
    #[cfg(feature = "hydrate")]
    use leptos::tachys::dom::window;

    const BLADE_LOCALSTORAGE_KEY: &str = "blade_dark_mode";

    fn storage() -> Option<web_sys::Storage> {
        cfg_if! {
            if #[cfg(feature = "hydrate")] {
                window().local_storage().ok().flatten()
            } else {
                None
            }
        }
    }

    fn is_system_dark_mode() -> bool {
        cfg_if! {
            if #[cfg(feature = "hydrate")] {
                window()
                    .match_media("(prefers-color-scheme: dark)")
                    .ok()
                    .flatten()
                    .map(|x| x.matches())
                    .unwrap_or(false)
            } else {
                false
            }
        }
    }

    pub fn get() -> bool {
        storage()
            .and_then(|storage| storage.get_item(BLADE_LOCALSTORAGE_KEY).ok().flatten())
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(is_system_dark_mode)
    }

    pub fn set(is_dark: bool) -> Result<(), String> {
        storage()
            .ok_or("no storage")?
            .set_item(BLADE_LOCALSTORAGE_KEY, &is_dark.to_string())
            .map_err(|e| format!("{e:?}"))
    }
}

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
