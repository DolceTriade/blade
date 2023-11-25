use leptos::*;
use log;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    log::info!("HELLO");
    view! {
        <button on:click=move |_| {
            log::info!("CLick");
            set_count.update(|c| *c += 1);
        }>

            "Click me: " {move || count.get()}
        </button>
    }
}
