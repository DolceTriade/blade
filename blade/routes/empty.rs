use leptos::*;

#[component]
pub fn Empty() -> impl IntoView {
    view! {
        <div class="grid items-center justify-self-center justify-center h-screen content-start">
            <div class="grid justify-self-center justify-center">
                <svg
                    height="128"
                    style="overflow:visible;enable-background:new 0 0 32 32"
                    viewBox="0 0 32 32"
                    width="32"
                    xml:space="preserve"
                    xmlns="http://www.w3.org/2000/svg"
                    xmlns:xlink="http://www.w3.org/1999/xlink"
                >
                    <g>
                        <g id="Error_1_">
                            <g id="Error">
                                <circle
                                    cx="16"
                                    cy="16"
                                    id="BG"
                                    r="16"
                                    style="fill:#D72828;"
                                ></circle>
                                <path
                                    d="M14.5,25h3v-3h-3V25z M14.5,6v13h3V6H14.5z"
                                    id="Exclamatory_x5F_Sign"
                                    style="fill:#E6E6E6;"
                                ></path>
                            </g>
                        </g>
                    </g>
                </svg>
            </div>
            <p class="font-normal text-3xl text-gray-500">Missing Invocation ID</p>
        </div>
    }
}
