use leptos::*;
use leptos_meta::*;
use std::string::ToString;

#[component]
pub fn Card(
    children: Children, 
    #[prop(into, default = "".into())]
    class: MaybeSignal<String>
) -> impl IntoView {
    view! {
        <div class=format!(
            "max-w-max p-6 bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700 {}",
            class.get(),
        )>{children()}</div>
    }
}
