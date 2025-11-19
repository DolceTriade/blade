use leptos_dom::helpers::window;

/// Opens a URL in a new browser tab
pub fn open_in_new_tab(url: &str) { let _ = window().open_with_url_and_target(url, "_blank"); }
