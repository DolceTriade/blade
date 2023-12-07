use leptos::*;

#[allow(non_snake_case)]
#[component]
pub fn ListItem(
    children: Children
) -> impl IntoView {
    view! { <li class="py-3 sm:py-4">{children()}</li> }
}


#[component]
pub fn List(
    children: Children
) -> impl IntoView
{
    view! {
        <ul role="list" class="divide-y divide-gray-200 dark:divide-gray-700">
            {children()}
        </ul>
    }
}
