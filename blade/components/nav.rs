use leptos::{prelude::*, tachys::dom::event_target_checked};
use leptos_router::hooks::use_location;
use darkmode::DarkMode;

fn extract_path(url_str: &str) -> Option<String> {
    if !url_str.starts_with("/invocation/") {
        return None;
    }
    Some(url_str.split("/").take(3).collect::<Vec<&str>>().join("/"))
}

#[allow(non_snake_case)]
#[component]
pub fn Nav(
    #[prop(into)] name: Signal<String>,
    #[prop(into)] logo: Signal<String>,
) -> impl IntoView {
    let location = use_location();
    let dark_mode = expect_context::<(ReadSignal<DarkMode>, WriteSignal<DarkMode>)>();
    let (menu_open, set_menu_open) = signal(false);
    view! {
        <nav class="border-gray-200 bg-gray-50 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white">
            <div class="flex flex-wrap items-center justify-between mx-auto p-4">
                <a
                    href=move || extract_path(&location.pathname.read()).unwrap_or("".to_string())
                    class="flex items-center rtl:space-x-reverse"
                >
                    <img class="hover:animate-spin w-14" src=move || logo.get() alt="Logo" />
                    <span class="self-center text-4xl font-semibold whitespace-nowrap">
                        {move || name.get()}
                    </span>
                </a>
                <div>
                    <button
                        data-collapse-toggle="navbar-hamburger"
                        type="button"
                        class="inline-flex items-center justify-center p-2 w-10 h-10 text-sm text-gray-500 rounded-lg hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200"
                        aria-controls="navbar-hamburger"
                        aria-expanded="false"
                        on:click=move |_| {
                            set_menu_open
                                .update(|d| {
                                    *d = !*d;
                                });
                        }
                    >
                        <span class="sr-only">Open main menu</span>
                        <svg
                            class="w-5 h-5"
                            aria-hidden="true"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 17 14"
                        >
                            <path
                                stroke="currentColor"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M1 1h15M1 7h15M1 13h15"
                            ></path>
                        </svg>
                    </button>
                    <div
                        class="fixed right-1 z-50 hidden my-4 text-base list-none bg-white divide-y divide-gray-100 rounded-lg shadow-sm dark:bg-gray-800 dark:divide-gray-600"
                        id="navbar-hamburger"
                        class:hidden=move || !*menu_open.read()
                    >
                        <ul class="flex flex-col font-medium mt-4 rounded-lg bg-gray-50 dark:bg-gray-800">
                            <li>
                                <label class="relative flex items-center group p-2 text-xl">
                                    Dark Mode
                                    <input
                                        type="checkbox"
                                        class="absolute left-1/2 -translate-x-1/2 w-full h-full peer appearance-none rounded-md"
                                        prop:checked=move || dark_mode.0.read().0
                                        on:change=move |ev| {
                                            let val = event_target_checked(&ev);
                                            dark_mode.1.set(DarkMode(val));
                                            darkmode::set(val).unwrap();
                                        }
                                    />
                                    <span class="w-16 h-10 flex items-center flex-shrink-0 ml-4 p-1 bg-gray-300 dark:bg-gray-800 rounded-full duration-300 ease-in-out peer-checked:bg-green-400 after:w-8 after:h-8 after:bg-white after:rounded-full after:shadow-md after:duration-300 peer-checked:after:translate-x-6 group-hover:after:translate-x-1"></span>
                                </label>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </nav>
    }
}
