use leptos::{html::*, prelude::*};
use web_sys::{Blob, Url, js_sys::Array, window};

const TRUNCATE_THRESHOLD: usize = 500;
const MAX_DISPLAY_LINES: usize = TRUNCATE_THRESHOLD * 2;

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let full_lines = Memo::new(move |_| text.get().lines().map(String::from).collect::<Vec<_>>());

    let (display_lines, set_display_lines) = signal(Vec::<String>::new());
    let (is_truncated, set_is_truncated) = signal(false);

    Effect::new(move |_| {
        let lines_vec = full_lines.get();
        let total_lines = lines_vec.len();

        if total_lines > MAX_DISPLAY_LINES {
            set_is_truncated.set(true);
            let mut truncated_lines = Vec::with_capacity(MAX_DISPLAY_LINES);
            truncated_lines.extend_from_slice(&lines_vec[0..TRUNCATE_THRESHOLD]);
            truncated_lines
                .extend_from_slice(&lines_vec[total_lines - TRUNCATE_THRESHOLD..total_lines]);
            set_display_lines.set(truncated_lines);
        } else {
            set_is_truncated.set(false);
            set_display_lines.set(lines_vec);
        }
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

    view! {
        <div class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto">
            <Show when=move || { is_truncated.get() } fallback=|| view! { <></> }>
                <div class="bg-yellow-600 text-white p-2 mb-2 rounded">
                    "Output truncated. Showing first " {TRUNCATE_THRESHOLD} " and last "
                    {TRUNCATE_THRESHOLD} " lines."
                    <button
                        on:click=open_raw_output
                        class="ml-4 px-3 py-1 bg-blue-700 hover:bg-blue-800 rounded text-sm"
                    >
                        "View Full Raw Output"
                    </button>
                </div>
            </Show>
            <For
                each=display_lines
                key=|line| line.clone()
                children=move |line: String| {
                    let converted = ansi_to_html::convert_escaped(&line)
                        .unwrap_or_else(|_| line.clone());
                    view! { <div class="whitespace-pre font-mono" inner_html=converted></div> }
                }
            />
        </div>
    }
}
