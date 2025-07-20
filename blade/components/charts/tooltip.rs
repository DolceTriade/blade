use leptos::{ev::MouseEvent, html::Div, prelude::*};

#[derive(Clone, Debug)]
pub struct TooltipPosition {
    pub x: f64,
    pub y: f64,
}

#[allow(non_snake_case)]
#[component]
pub fn Tooltip(
    children: Children,
    #[prop(into)] position: Signal<Option<TooltipPosition>>,
    #[prop(default = "top")] placement: &'static str, // "top", "bottom", "left", "right"
) -> impl IntoView {
    let tooltip_ref = NodeRef::<Div>::new();

    let style = move || {
        if let Some(pos) = position.get() {
            let offset = 10.0; // Distance from cursor
            let (x, y) = match placement {
                "bottom" => (pos.x, pos.y + offset),
                "left" => (pos.x - offset, pos.y),
                "right" => (pos.x + offset, pos.y),
                _ => (pos.x, pos.y - offset), // "top" (default)
            };

            format!(
                "position: fixed; \
                 left: {x}px; \
                 top: {y}px; \
                 transform: translate(-50%, -100%); \
                 background-color: rgba(45, 55, 72, 0.95); \
                 color: white; \
                 border: 1px solid #4a5568; \
                 border-radius: 6px; \
                 padding: 8px 12px; \
                 font-size: 12px; \
                 font-weight: 500; \
                 white-space: nowrap; \
                 pointer-events: none; \
                 z-index: 1000; \
                 box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); \
                 backdrop-filter: blur(4px); \
                 opacity: 1; \
                 transition: opacity 0.2s ease-in-out;"
            )
        } else {
            "position: fixed; \
             opacity: 0; \
             pointer-events: none; \
             z-index: -1;"
                .to_string()
        }
    };

    view! {
        <div node_ref=tooltip_ref style=style class="tooltip-container">
            {children()}
        </div>
    }
}

pub fn get_mouse_position_from_event(event: &MouseEvent) -> TooltipPosition {
    TooltipPosition {
        x: event.client_x() as f64,
        y: event.client_y() as f64,
    }
}
