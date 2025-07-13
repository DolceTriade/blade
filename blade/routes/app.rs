use components::nav::Nav;
use darkmode::DarkMode;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::{
    artifact::Artifact,
    dashboard::Dashboard,
    details::Details,
    empty::Empty,
    invocation::Invocation,
    summary::Summary,
    test::Test,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (dark_mode, set_dark_mode) = signal(DarkMode(false));
    provide_context((dark_mode, set_dark_mode));
    Effect::new(move || {
        set_dark_mode.set(DarkMode(darkmode::get()));
    });
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
        <Html class:dark=move || dark_mode.read().0 />
        <Router>
            <div
                id="root"
                class="h-screen w-screen max-w-screen max-h-screen dark:bg-gray-800 dark:placeholder-gray-400 dark:text-white overflow-clip"
                class:dark=move || dark_mode.read().0
            >
                <Nav name="Blade" logo="/assets/logo.svg" />
                <main>
                    <Routes fallback=|| "Not Found.">
                        <ParentRoute path=path!("invocation/:id") view=Invocation>
                            <Route path=path!("test") view=Test />
                            <Route path=path!("details") view=Details />
                            <Route path=path!("artifact") view=Artifact />
                            <Route path=path!("*any") view=Summary />
                        </ParentRoute>
                        <Route path=path!("dashboard") view=Dashboard />
                        <Route path=path!("/") view=Dashboard />
                        <Route path=path!("*any") view=Empty />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
