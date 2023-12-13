use leptos::*;

#[component]
pub fn Nav(
    #[prop(into)] name: MaybeSignal<String>,
    #[prop(into)] logo: MaybeSignal<String>,
) -> impl IntoView {
    view! {
        <nav class="border-gray-200 bg-gray-50 dark:bg-gray-800 dark:border-gray-700">
            <div class="flex flex-wrap items-center justify-between mx-auto p-4">
                <a href="#" class="flex items-center rtl:space-x-reverse">
                    <img class="hover:animate-spin" src=move || logo.get() class="w-14" alt="Logo"/>
                    <span class="self-center text-4xl font-semibold whitespace-nowrap dark:text-white">
                        {move || name.get()}
                    </span>
                </a>
                <button
                    data-collapse-toggle="navbar-hamburger"
                    type="button"
                    class="inline-flex items-center justify-center p-2 w-10 h-10 text-sm text-gray-500 rounded-lg hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600"
                    aria-controls="navbar-hamburger"
                    aria-expanded="false"
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
                <div class="hidden w-full" id="navbar-hamburger">
                    <ul class="flex flex-col font-medium mt-4 rounded-lg bg-gray-50 dark:bg-gray-800 dark:border-gray-700">
                        <li>
                            <a
                                href="#"
                                class="block py-2 px-3 text-white bg-blue-700 rounded dark:bg-blue-600"
                                aria-current="page"
                            >
                                Home
                            </a>
                        </li>
                    </ul>
                </div>
            </div>
        </nav>
    }
}
