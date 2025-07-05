#[allow(unused_macros)]
#[macro_export]
macro_rules! measure_time {
    ($label:expr, $($code:tt)*) => {{
        let start = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        let result = { $($code)* };

        let end = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        let duration = end - start;
        web_sys::console::log_1(&format!("{}: {:.3}ms", $label, duration).into());

        result
    }};
}
