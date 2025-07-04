use leptos::{html::*, prelude::*};
use leptos::either::Either;

pub fn scroll_bottom(el: web_sys::HtmlElement) {
    if let Some(c) = el.last_element_child() {
        c.scroll_into_view_with_bool(false)
    }
}

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(#[prop(into)] text: Signal<String>) -> impl IntoView {
    let nr = NodeRef::<Div>::new();
    Effect::new(move |_| {
        // Empty callback so this is called on update.
        text.with(|_| {});
        if let Some(nr) = nr.get() {
            scroll_bottom(nr.into());
        }
    });
    view! {
        <div
            node_ref=nr
            class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto"
        >
            {move || match ansi_to_html::convert_escaped(&text.read()) {
                Err(err) => Either::Left(view! { <div>{format!("mistake: {err:#?}")}</div> }),
                Ok(t) => {
                    Either::Right(view! { <div class="inline whitespace-pre font-mono" inner_html=t></div> })
                }
            }}

        </div>
    }
}
