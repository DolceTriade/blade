use leptos::prelude::*;
use tailwindmerge::tailwind_merge;

#[allow(non_snake_case)]
#[component]
pub fn Card(
    children: Children,
    #[prop(into, default = "".into())] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || tailwind_merge(
            "max-w-fit p-6 bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white",
            &class.get(),
        )>{children()}</div>
    }
}
