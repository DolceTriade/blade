use cfg_if::cfg_if;
pub mod components;
pub mod routes;

// Needs to be in lib.rs AFAIK because wasm-bindgen needs us to be compiling a lib. I may be wrong.
cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use leptos::*;
        use wasm_bindgen::prelude::wasm_bindgen;
        use tracing_web::{MakeWebConsoleWriter, performance_layer};
        use tracing_subscriber::fmt::format::Pretty;
        use tracing_subscriber::prelude::*;
        use crate::routes::app::App;

        #[wasm_bindgen]
        pub fn hydrate() {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false) // Only partially supported across browsers
                .without_time()   // std::time is not available in browsers, see note below
                .with_writer(MakeWebConsoleWriter::new()); // write events to the console
            let perf_layer = performance_layer()
                .with_details_from_fields(Pretty::default());

            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(perf_layer)
                .init();
            mount_to_body(App);
        }
    }
}
