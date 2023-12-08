use leptos::*;
use state;
use tailwindmerge::tailwind_merge;

#[allow(non_snake_case)]
#[component]
pub fn StatusIcon(
    status: MaybeSignal<state::Status>,
    #[prop(into, default = "".into())] class: MaybeSignal<String>,
) -> impl IntoView {
    view! {
        {move || match status.get() {
            state::Status::Success => {
                view! { <img class=class.get() src="/pkg/static/success.svg"/> }.into_view()
            }
            state::Status::Fail => {
                view! { <img class=class.get() src="/pkg/static/fail.svg"/> }.into_view()
            }
            _ => {
                view! {
                    <span class=tailwind_merge("relative flex", &class.get())>
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-yellow-200 opacity-75"></span>
                        <span class=tailwind_merge(
                            "relative inline-flex rounded-full bg-yellow-300",
                            &class.get(),
                        )></span>
                    </span>
                }
                    .into_view()
            }
        }}
    }
}
