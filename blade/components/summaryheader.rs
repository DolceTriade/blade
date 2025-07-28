use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};
use time::macros::format_description;

use crate::{clipboard::CopyToClipboard, statusicon::StatusIcon, tooltip::Tooltip};

#[allow(non_snake_case)]
#[component]
fn SummaryItem<S>(num: Signal<usize>, suffix: S) -> impl IntoView
where
    S: AsRef<str> + std::fmt::Display + std::marker::Send + 'static,
{
    view! {
        <div class="pl-4 pr-4">
            <span class="text-m">{move || num.get().to_string()}</span>
            <span class="pl-1 text-xs">
                {move || format!("{}{}", suffix, if num.get() != 1 { "s" } else { "" })}
            </span>
        </div>
    }
}

#[derive(PartialEq)]
struct Counts {
    num_targets: usize,
    passing_targets: usize,
    failing_targets: usize,
    num_tests: usize,
    passing_tests: usize,
    failing_tests: usize,
    status: state::Status,
}

pub fn format_time(t: &std::time::SystemTime) -> String {
    let ts: time::OffsetDateTime = (*t).into();
    ts.format(&format_description!(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]"
    ))
    .unwrap_or(format!("{ts:#?}"))
}

fn ucfirst(s: &str) -> String {
    let mut new = s.to_owned();
    if let Some(start) = new.get_mut(0..1) {
        start.make_ascii_lowercase();
    }
    new
}

fn toggle_page_url(current_path: &str, page: &str) -> String {
    let page_suffix = format!("/{page}");

    if current_path.ends_with(&page_suffix) {
        // If currently on this page, go back to base invocation
        // Split by '/' and take the first 3 parts (empty, "invocation", id)
        let parts: Vec<&str> = current_path.split('/').collect();
        if parts.len() >= 3 {
            format!("/{}/{}", parts[1], parts[2])
        } else {
            current_path.to_string()
        }
    } else {
        // If not on this page, go to this page
        // Split by '/' and take the first 3 parts, then append the desired page
        let parts: Vec<&str> = current_path.split('/').collect();
        if parts.len() >= 3 {
            format!("/{}/{}{}", parts[1], parts[2], page_suffix)
        } else {
            format!("{}{}", current_path.trim_end_matches('/'), page_suffix)
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn SummaryHeader() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();
    let counts = Memo::new(move |_| {
        invocation.with(|invocation| {
            let num_targets = invocation.targets.len();
            let mut passing_targets: usize = 0;
            let mut failing_targets: usize = 0;
            invocation.targets.values().for_each(|t| match t.status {
                state::Status::Success => passing_targets += 1,
                state::Status::Fail => failing_targets += 1,
                _ => {},
            });
            let num_tests = invocation.tests.len();
            let mut passing_tests: usize = 0;
            let mut failing_tests: usize = 0;
            invocation.tests.values().for_each(|t| match t.status {
                state::Status::Success => passing_tests += 1,
                state::Status::InProgress => {},
                _ => failing_tests += 1,
            });

            Counts {
                num_targets,
                passing_targets,
                failing_targets,
                num_tests,
                passing_tests,
                failing_tests,
                status: invocation.status,
            }
        })
    });
    move || {
        let num_targets = Signal::derive(move || counts.read().num_targets);
        let passing_targets = Signal::derive(move || counts.read().passing_targets);
        let failing_targets = Signal::derive(move || counts.read().failing_targets);
        let num_tests = Signal::derive(move || counts.read().num_tests);
        let passing_tests = Signal::derive(move || counts.read().passing_tests);
        let failing_tests = Signal::derive(move || counts.read().failing_tests);
        let status = Signal::derive(move || counts.read().status);
        let cmd = ucfirst(&invocation.read().command);
        let patterns = invocation.read().pattern.join(" ");
        let start = format_time(&invocation.read().start);
        let location = use_location();
        let duration = invocation
            .read()
            .end
            .map(|end| {
                let duration = end
                    .duration_since(invocation.read().start)
                    .unwrap_or_default();
                format!("Took {}", humantime::format_duration(duration))
            })
            .unwrap_or_default();
        let is_disconnected = Signal::derive(move || {
            let inv = invocation.read();
            let is_incomplete = matches!(
                inv.status,
                state::Status::InProgress | state::Status::Unknown
            );
            is_incomplete && !inv.is_live
        });
        view! {
            <div class="w-screen h-fit grid grid-rows-1 grid-flow-col content-start divide-x overflow-hidden">
                <div class="grid grid-rows-1 grid-flow-col place-content-start">
                    <div class="p-4 place-content-center self-center relative overflow-hidden">
                        <StatusIcon class="h-8 w-8" status=status />
                        {move || {
                            is_disconnected
                                .get()
                                .then(|| {
                                    view! {
                                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-10">
                                            <Tooltip tooltip=|| "Disconnected">
                                                <img
                                                    class="h-6 w-6 dark:invert cursor-pointer"
                                                    src="/assets/disconnect.svg"
                                                />
                                            </Tooltip>
                                        </div>
                                    }
                                })
                        }}
                    </div>
                    <div class="grid grid-rows-3 items-start self-center place-content-center">
                        <div class="flex gap-3 content-start place-items-center">
                            <div class="place-content-center">
                                <img class="h-6 w-6" src="/assets/bazel.svg" />
                            </div>
                            <span class="text-lg">
                                <b>{cmd}</b>
                            </span>
                            <span class="inline-flex overflow-auto whitespace-nowrap">
                                {patterns.clone()}
                            </span>
                            <span>
                                <CopyToClipboard text=patterns />
                            </span>
                        </div>
                        <div class="text-gray-400 text-sm self-center">{start}</div>
                        <div class="flex gap-2 items-center">
                            {duration}
                            <A href=move || {
                                let current_path = location.pathname.read();
                                toggle_page_url(&current_path, "details")
                            }>
                                <span class="text-blue-500 underline">(details)</span>
                            </A>
                            {move || {
                                invocation
                                    .read()
                                    .profile_uri
                                    .as_ref()
                                    .map(|_| {
                                        view! {
                                            <A href=move || {
                                                let current_path = location.pathname.read();
                                                toggle_page_url(&current_path, "profile")
                                            }>
                                                <span class="text-blue-500 underline">(profile)</span>
                                            </A>
                                        }
                                    })
                            }}
                        </div>
                    </div>
                </div>
                <div class="content-center place-self-end self-center grid grid-rows-1 grid-flow-col">
                    <div class="p-4 place-content-center">
                        <img class="h-10 w-10 dark:invert" src="/assets/code.svg" />
                    </div>
                    <div class="p-4 place-content-center">
                        <SummaryItem num=num_targets suffix="Total Target" />
                        <SummaryItem num=passing_targets suffix="Passing Target" />
                        {(failing_targets.get() > 0)
                            .then(|| {
                                view! {
                                    <SummaryItem num=failing_targets suffix="Failing Target" />
                                }
                            })}
                    </div>
                </div>
                <div class="content-center place-self-center self-center grid grid-rows-1 grid-flow-col">
                    {{
                        (num_tests.get() > 0)
                            .then(|| {
                                view! {
                                    <div class="p-4 place-content-center">
                                        <img class="h-10 w-10 dark:invert" src="/assets/test.svg" />
                                    </div>
                                }
                            })
                    }}
                    <div class="p-4 place-content-center">
                        {(num_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=num_tests suffix="Total Test" /> }
                            })}
                        {(passing_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=passing_tests suffix="Passing Test" /> }
                            })}
                        {(failing_tests.get() > 0)
                            .then(|| {
                                view! { <SummaryItem num=failing_tests suffix="Failing Test" /> }
                            })}
                    </div>
                </div>
            </div>
        }
    }
}
