use leptos::{html::*, prelude::*};

const INITIAL_LINES: usize = 500;
const RENDER_CHUNK_SIZE: usize = 500;
const RENDER_INTERVAL_MS: u64 = 10;

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let container_ref = NodeRef::<Div>::new();

    let lines = Memo::new(move |_| text.get().lines().map(String::from).collect::<Vec<_>>());
    let (lines_to_render, set_lines_to_render) = signal(INITIAL_LINES);
    let (rendered_lines, set_rendered_lines) = signal(0);

    // Effect to handle initial render and full text changes
    Effect::new(move |_| {
        // This effect runs when `lines` changes
        lines.with(|_l| {});
        if let Some(container) = container_ref.get() {
            // Clear previous content
            container.set_inner_html("");
            set_rendered_lines.set(0);
            set_lines_to_render.set(INITIAL_LINES);
        }
    });

    // Effect to render new lines incrementally
    Effect::new(move || {
        if let Some(container) = container_ref.get() {
            let lines_vec = lines.read_untracked();
            let start = rendered_lines.get_untracked();
            let end = lines_to_render.get().min(lines_vec.len());
            if start >= end {
                return;
            }
            let document = document();
            for i in start..end {
                if let Some(line_str) = lines_vec.get(i)
                    && let Ok(div) = document.create_element("div")
                {
                    let converted = ansi_to_html::convert_escaped(line_str)
                        .unwrap_or_else(|_| line_str.clone());
                    div.set_inner_html(&converted);
                    div.set_class_name("whitespace-pre font-mono");
                    let _ = container.append_child(&div);
                }
            }
            set_rendered_lines.set(end);

            // Scroll to bottom
            if let Some(c) = container.last_element_child() {
                c.scroll_into_view_with_bool(false);
            }
        }
    });

    // Time-based loading
    Effect::new(move || {
        _ = lines_to_render.get();
        let total_lines = lines.read_untracked().len();
        let rendered_lines = rendered_lines.get_untracked();
        if total_lines <= rendered_lines {
            return;
        }
        set_timeout(
            move || {
                set_lines_to_render.update(|n| *n += RENDER_CHUNK_SIZE);
            },
            std::time::Duration::from_millis(RENDER_INTERVAL_MS),
        );
    });

    view! {
        <div
            node_ref=container_ref
            class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto"
        ></div>
    }
}
