use crate::components::nav::Nav;
use crate::routes::artifact::Artifact;
use crate::routes::details::Details;
use crate::routes::invocation::Invocation;
use crate::routes::summary::Summary;
use crate::routes::test::Test;
use crate::routes::empty::Empty;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let formatter = |text: String| {
        format!(
            "Blade{}{}",
            if text.is_empty() { "" } else { " - " },
            if text.is_empty() { "" } else { &text },
        )
    };
    view! {
        <Title formatter/>
        <Stylesheet id="leptos" href="/pkg/static/style.css"/>
        <Router>
            <div id="root" class="h-screen w-screen max-h-screen max-w-screen overflow-hidden">
                <Nav name="Blade" logo="/pkg/static/logo.svg"/>
                <main>
                <Routes fallback=|| "Not Found.">
                    <ParentRoute path=path!("invocation/:id") view=Invocation>
                        <Route path=path!("*any") view=Summary/>
                        <Route path=path!("test") view=Test/>
                        <Route path=path!("details") view=Details/>
                        <Route path=path!("artifact") view=Artifact/>
                    </ParentRoute>
                    <Route path=path!("*any") view=Empty/>
                </Routes>
                </main>
            </div>
        </Router>
    }
}
