use leptos::{html, prelude::*};

#[allow(non_snake_case)]
#[component]
pub fn Tooltip<F, IV>(children: Children, tooltip: F) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    let tel = NodeRef::<html::Span>::new();
    let hover = move |_| {
        if let Some(el) = tel.get() {
            el.parent_element()
                .map(|s| {
                    let body = document().body().unwrap().get_bounding_client_rect();
                    s.get_bounding_client_rect().y() - body.y()
                })
                .map(|t| el.set_attribute("style", &format!("top: {}px", t)).ok());
        }
    };
    view! {
        <div on:mouseenter=hover class="group">
            <span
                node_ref=tel
                class="pointer-events-none absolute top-0 left-auto w-max bg-black text-white rounded-lg opacity-0 transition-opacity group-hover:pointer-events-auto group-hover:opacity-100 group-hover:z-50"
            >
                {tooltip()}
            </span>
            {children()}
        </div>
    }
}
