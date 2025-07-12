// Dark mode utilities and state management
use cfg_if::cfg_if;

/// Dark mode state shared between components and routes
#[derive(Clone, Copy, Debug)]
pub struct DarkMode(pub bool);

const BLADE_LOCALSTORAGE_KEY: &str = "blade_dark_mode";

#[cfg(feature = "hydrate")]
use leptos::tachys::dom::window;

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
