use ansi_to_html;
use leptos::*;

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(text: MaybeSignal<String>) -> impl IntoView {
    view! {
        <div
            class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto"
        >
            {move || match ansi_to_html::convert_escaped(&text.get()) {
                Err(err) => view! { <div>{format!("mistake: {:#?}", err)}</div> },
                Ok(t) => view! { <div class="inline whitespace-pre font-mono" inner_html=t></div> },
            }}

        </div>
    }
}
