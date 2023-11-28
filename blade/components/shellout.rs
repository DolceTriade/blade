use ansi_to_html;
use leptos::*;
use leptos_meta::*;
use std::string::ToString;

#[component]
pub fn ShellOut(
    text: MaybeSignal<String>,
) -> impl IntoView {
    log::info!("{:#?}", text);
    view! {
        <div class="bg-gray-800 text-white p-4 rounded-lg overflow-x-auto">
            {move || match ansi_to_html::convert_escaped(&text.get()) {
                Err(e) => view! { <div>format!("mistake: {:#?}", e)</div> },
                Ok(t) => view! { <div class="inline whitespace-pre font-mono" inner_html=t></div> },
            }}

        </div>
    }
}
