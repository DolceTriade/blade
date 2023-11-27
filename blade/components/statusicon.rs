use leptos::*;

#[component]
pub fn StatusIcon(
    success: bool,
    #[prop(into, default = "".into())] 
    class: MaybeSignal<String>,
) -> impl IntoView {
    view! {
        {move || match success {
            true => {
                view! { <img class=class.get() src="/pkg/static/success.svg"/> }
            }
            false => {
                view! { <img class=class.get() src="/pkg/static/fail.svg"/> }
            }
        }}
    }
}
