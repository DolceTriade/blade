use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::{
    components::nav::Nav,
    routes::{
        artifact::Artifact,
        details::Details,
        empty::Empty,
        invocation::Invocation,
        summary::Summary,
        test::Test,
    },
};

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
        <Title formatter />
        <Stylesheet id="leptos" href="/assets/style.css" />
        <Router>
            <div id="root" class="h-screen w-screen">
                <Nav name="Blade" logo="/assets/logo.svg" />
                <main>
                    <Routes fallback=|| "Not Found.">
                        <ParentRoute path=path!("invocation/:id") view=Invocation>
                            <Route path=path!("test") view=Test />
                            <Route path=path!("details") view=Details />
                            <Route path=path!("artifact") view=Artifact />
                            <Route path=path!("*any") view=Summary />
                        </ParentRoute>
                        <Route path=path!("*any") view=Empty />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
