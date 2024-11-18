use leptos::prelude::*;
use tailwindmerge::tailwind_merge;

#[allow(non_snake_case)]
#[component]
pub fn Card(
    children: Children,
    #[prop(into, default = "".into())] class: MaybeSignal<String>,
) -> impl IntoView {
    view! {
        <div class=move || tailwind_merge(
            "max-w-max p-6 bg-white border border-gray-200 rounded-lg shadow",
            &class.get(),
        )>{children()}</div>
    }
}
