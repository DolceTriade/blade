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
                .map(|media| media.matches())
                .unwrap_or_default()
        } else {
            false
        }
    }
}

pub fn get() -> bool {
    storage()
        .and_then(|s| s.get(BLADE_LOCALSTORAGE_KEY).ok().flatten())
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or_else(is_system_dark_mode)
}

pub fn set(b: bool) -> anyhow::Result<()> {
    let val: String = b.to_string();
    storage()
        .map(|s| {
            s.set(BLADE_LOCALSTORAGE_KEY, &val)
                .map_err(|e| anyhow::anyhow!("{e:#?}"))
        })
        .unwrap_or(Err(anyhow::anyhow!("No local storage")))
}
