use leptos::*;
use leptos_meta::*;
use log;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    log::info!("HELLO");
    provide_meta_context();
    view! {
            <Stylesheet id="leptos" href="/pkg/static/style.css"/>
            <div class="p-6 max-w-sm mx-auto bg-white rounded-xl shadow-lg flex items-center space-x-4">
                <div>
                    <div class="text-xl font-medium text-black">ChitChat</div>
                    <p class="text-slate-500">You have a new message!</p>
                    <button class="p-6 max-w-sm mx-auto bg-white rounded-xl shadow-lg flex items-center space-x-4" on:click=move |_| {
                        log::info!("CLick");
                        set_count.update(|c| *c += 1);
                    }>
        
                        "Click me: " {move || count.get()}
                    </button>
                </div>
            </div>
        }
}
