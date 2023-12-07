use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use log;
use crate::components::nav::Nav;
use crate::routes::empty::Empty;
use crate::routes::invocation::Invocation;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/static/style.css"/>
        <div id="root" class="h-full w-full max-h-full max-w-full">
            <Router>
                <Nav name="Blade" logo="/pkg/static/logo.svg"/>
                <main>
                    <Routes>
                        <Route path="invocation/:id" view=Invocation/>
                        <Route path="*" view=Empty/>
                    </Routes>
                </main>
            </Router>
        </div>
    }
}
