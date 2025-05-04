use leptos::prelude::*;

#[component]
pub fn CopyToClipboard(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let clipboard = window().navigator().clipboard();
    view! {
        <div on:click=move |_| {
            // Hope for the best... Dont't check any errors.
            _ = clipboard.write_text(&text.read());
        }>
            <img src="/assets/copy.svg" />
        </div>
    }
}
