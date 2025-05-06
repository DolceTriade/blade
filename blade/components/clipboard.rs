use leptos::prelude::*;

#[component]
pub fn CopyToClipboard(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let clipboard = window().navigator().clipboard();
    view! {
        <button on:click=move |_| {
            _ = clipboard.write_text(&text.read());
        }>
            <img class="dark:invert" src="/assets/copy.svg" />
        </button>
    }
}
