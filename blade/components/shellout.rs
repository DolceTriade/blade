use std::hash::{DefaultHasher, Hash, Hasher};

use leptos::{html::*, prelude::*};
use web_sys::{Blob, Url, js_sys::Array, window};

const TRUNCATE_THRESHOLD: usize = 500;
const MAX_DISPLAY_LINES: usize = TRUNCATE_THRESHOLD * 2;

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let (truncated, set_truncated) = signal(false);
    let (force_show, set_force_show) = signal(false);
    let lines = Memo::new(move |_| {
        let text = text.read();
        let lines: Vec<&str> = text.lines().collect();
        let mut should_truncate = lines.len() > MAX_DISPLAY_LINES;
        if force_show.get() {
            should_truncate = false;
        }
        set_truncated(should_truncate);
        lines
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| {
                if should_truncate {
                    *i < TRUNCATE_THRESHOLD || *i > lines.len() - TRUNCATE_THRESHOLD
                } else {
                    true
                }
            })
            .map(|(i, s)| (i, s.to_string()))
            .collect::<Vec<(usize, String)>>()
    });

    let open_raw_output = move |_| {
        let array = Array::new();
        let js_text: wasm_bindgen::JsValue = text.get().into();
        array.push(&js_text);
        // Create the blob
        let blob = Blob::new_with_str_sequence(&array).unwrap();

        // Create the blob URL
        let blob_url = Url::create_object_url_with_blob(&blob).unwrap();

        if let Some(window) = window() {
            let _ = window.open_with_url_and_target(&blob_url, "_blank");
        }
    };

    let show_all = move |_| {
        set_force_show(true);
    };

    view! {
        <div class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto">
            <Show when=move || { truncated.get() } fallback=|| view! { <></> }>
                <div class="bg-yellow-600 text-white p-2 mb-2 rounded">
                    "Output truncated. Showing first " {TRUNCATE_THRESHOLD} " and last "
                    {TRUNCATE_THRESHOLD} " lines."
                    <button
                        on:click=open_raw_output
                        class="ml-4 px-3 py-1 bg-blue-700 hover:bg-blue-800 rounded text-sm"
                    >
                        "View Full Raw Output"
                    </button>
                    <button
                        on:click=show_all
                        class="ml-4 px-3 py-1 bg-blue-700 hover:bg-blue-800 rounded text-sm"
                    >
                        "Load Full Output (slow)"
                    </button>
                </div>
            </Show>
            <For
                each=move || lines.get()
                key=|line| {
                    let mut h = DefaultHasher::new();
                    line.0.hash(&mut h);
                    line.1.hash(&mut h);
                    h.finish()
                }
                children=move |line| {
                    let converted = ansi_to_html::convert_escaped(&line.1)
                        .unwrap_or_else(|_| line.1.clone());
                    view! { <div class="whitespace-pre font-mono" inner_html=converted></div> }
                }
            />
        </div>
    }
}
