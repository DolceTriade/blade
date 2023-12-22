use leptos::*;

#[component]
pub fn AccordionItem<F, IV>(
    header: F,
    children: Children,
    #[prop(optional, into)] header_class: String,
    #[prop(optional, default = false)]
    hide: bool,
) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    let (hide, set_hide) = create_signal(hide);
    view! {
        <button
            type="button"
            on:click=move |_| set_hide.set(!hide.get())
            class="flex w-full grow items-center justify-between p-5 font-medium rtl:text-right text-gray-500 border first:border-t-0 border-gray-200 first:rounded-t-xl last:rounded-b-xl focus:ring-4 focus:ring-gray-200 dark:focus:ring-gray-800 dark:border-gray-700 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 gap-3"
        >
            <span class=header_class>{header()}</span>
            <span>
                <svg
                    data-accordion-icon
                    class=move || {
                        format!(
                            "transition-all w-3 h-3 shrink-0 {}",
                            if hide.get() { "rotate-180" } else { "" },
                        )
                    }

                    aria-hidden="true"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 10 6"
                >
                    <path
                        stroke="currentColor"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M9 5 5 1 1 5"
                    ></path>
                </svg>
            </span>
        </button>
        <div class=move || {
            format!(
                "transition-all ease-in {}",
                if hide.get() { "max-h-0 absolute opacity-0 overflow-hidden" } else { "" },
            )
        }>
            <div class="p-5 border border-b-0 border-gray-200 dark:border-gray-700 dark:bg-gray-900">
                {children()}
            </div>
        </div>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn Accordion(children: Children) -> impl IntoView {
    view! { <div class="m-0 p-0">{children()}</div> }
}
