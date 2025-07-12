#![recursion_limit = "256"]
use cfg_if::cfg_if;

// Needs to be in lib.rs AFAIK because wasm-bindgen needs us to be compiling a
// lib. I may be wrong.
cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;
        use tracing_web::{MakeWebConsoleWriter};
        use tracing_subscriber::prelude::*;
        use routes::app::App;

        #[wasm_bindgen]
        pub fn hydrate() {
            console_error_panic_hook::set_once();
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false) // Only partially supported across browsers
                .without_time()   // std::time is not available in browsers, see note below
                .with_writer(MakeWebConsoleWriter::new()); // write events to the console
            tracing_subscriber::registry()
                .with(fmt_layer)
                .init();
            leptos::mount::hydrate_body(App);
        }
    }
}
