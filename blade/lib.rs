use cfg_if::cfg_if;
#[allow(clippy::empty_docs)]
pub mod components;
#[allow(clippy::empty_docs)]
pub mod routes;

// Needs to be in lib.rs AFAIK because wasm-bindgen needs us to be compiling a lib. I may be wrong.
cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use leptos::*;
        use leptos::mount::mount_to_body
        use wasm_bindgen::prelude::wasm_bindgen;
        use tracing_web::{MakeWebConsoleWriter};
        use tracing_subscriber::prelude::*;
        use crate::routes::app::App;

        #[wasm_bindgen]
        pub fn hydrate() {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false) // Only partially supported across browsers
                .without_time()   // std::time is not available in browsers, see note below
                .with_writer(MakeWebConsoleWriter::new()); // write events to the console

            tracing_subscriber::registry()
                .with(fmt_layer)
                .init();
            mount_to_body(App);
        }
    }
}
