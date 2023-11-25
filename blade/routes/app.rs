use leptos::*;
use leptos_meta::*;
use log;
use crate::components::nav::Nav;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/static/style.css"/>
        <Nav name="Blade".into() logo="/pkg/static/logo.svg".into()/>
    }
}
