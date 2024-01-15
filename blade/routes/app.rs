use crate::components::nav::Nav;
use crate::routes::artifact::Artifact;
use crate::routes::empty::Empty;
use crate::routes::invocation::Invocation;
use crate::routes::summary::Summary;
use crate::routes::test::Test;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
        <div id="root" class="h-screen w-screen max-h-screen max-w-screen overflow-hidden">
            <Router>
                <Nav name="Blade" logo="/pkg/static/logo.svg"/>
                <main>
                    <Routes>
                        <Route path="invocation/:id" view=Invocation>
                            <Route path="*any" view=Summary/>
                            <Route path="test" view=Test/>
                            <Route path="artifact" view=Artifact/>
                        </Route>
                        <Route path="*" view=Empty/>
                    </Routes>
                </main>
            </Router>
        </div>
    }
}
