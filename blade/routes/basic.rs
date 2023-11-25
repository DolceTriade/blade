use leptos::*;
use log;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    log::info!("HELLO");
    view! {cx,
        <button
            on:click=move |_| {
                log::info!("CLick");
                set_count.update(|c| *c += 1);
            }
        >
            "Click me: "
            {move || count.get()}
        </button>
    }
}
