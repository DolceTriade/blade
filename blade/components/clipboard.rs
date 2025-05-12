use leptos::prelude::*;
use tailwindmerge::tailwind_merge;

#[component]
pub fn CopyToClipboard(
    #[prop(into)] text: Signal<String>,
    #[prop(into, default = "".into())] class: Signal<String>,
) -> impl IntoView {
    view! {
        <span class=move || tailwind_merge("h-4 w-4 rounded-lg inline-block", &class.read())>
            <button
                title="Copy to clipboard"
                on:click=move |_| {
                    let clipboard = window().navigator().clipboard();
                    _ = clipboard.write_text(&text.read());
                }
            >

                <img class="dark:invert" src="/assets/copy.svg" />
            </button>
        </span>
    }
}
