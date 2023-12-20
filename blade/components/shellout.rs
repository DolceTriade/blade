use ansi_to_html;
use leptos::*;
use leptos_dom::html::*;

pub fn scroll_bottom(el: HtmlElement<Div>) {
    if let Some(c) = el.last_element_child() {
        c.scroll_into_view_with_bool(false)
    }
}

#[allow(non_snake_case)]
#[component]
pub fn ShellOut(
    #[prop(into)]
    text: MaybeSignal<String>
) -> impl IntoView {
    let nr = create_node_ref::<Div>();
    let t = text.clone();
    create_effect(move|_| {
        // Empty callback so this is called on update.
        t.with(|_|{});
        if let Some(nr) = nr.get() {
            scroll_bottom(nr);
        }
    });
    view! {
        <div _ref=nr class="bg-gray-800 text-white p-4 rounded-lg overflow-auto overflow-x-auto"
        >
            {move || match ansi_to_html::convert_escaped(&text.get()) {
                Err(err) => view! { <div>{format!("mistake: {:#?}", err)}</div> },
                Ok(t) => view! { <div class="inline whitespace-pre font-mono" inner_html=t></div> },
            }}

        </div>
    }
}
