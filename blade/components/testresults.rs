use leptos::prelude::*;

use crate::components::{accordion::*, shellout::ShellOut, statusicon::StatusIcon};

fn junit_status_to_status(s: junit_parser::TestStatus) -> state::Status {
    match s {
        junit_parser::TestStatus::Success => state::Status::Success,
        junit_parser::TestStatus::Error(_) => state::Status::Fail,
        junit_parser::TestStatus::Failure(_) => state::Status::Fail,
        junit_parser::TestStatus::Skipped(_) => state::Status::Skip,
    }
}

fn status_weight(s: &junit_parser::TestStatus) -> u8 {
    match s {
        junit_parser::TestStatus::Error(_) => 1,
        junit_parser::TestStatus::Failure(_) => 1,
        junit_parser::TestStatus::Skipped(_) => 2,
        junit_parser::TestStatus::Success => 3,
    }
}

fn sort_tests(cases: &[junit_parser::TestCase]) -> Vec<&junit_parser::TestCase> {
    let mut cases = cases.iter().collect::<Vec<_>>();
    cases.sort_unstable_by(|a, b| {
        let a_s = status_weight(&a.status);
        let b_s = status_weight(&b.status);
        if a_s != b_s {
            return a_s.cmp(&b_s);
        }
        a.name.cmp(&b.name)
    });
    cases
}

fn merge_error(e: &junit_parser::TestError) -> String {
    let mut parts = Vec::new();
    if !e.error_type.is_empty() {
        parts.push(e.error_type.as_str());
    }
    if !e.message.is_empty() {
        parts.push(e.message.as_str());
    }
    if !e.text.is_empty() {
        parts.push(e.text.as_str());
    }
    parts.join("\n")
}

fn merge_fail(e: &junit_parser::TestFailure) -> String {
    let mut parts = Vec::new();
    if !e.failure_type.is_empty() {
        parts.push(e.failure_type.as_str());
    }
    if !e.message.is_empty() {
        parts.push(e.message.as_str());
    }
    if !e.text.is_empty() {
        parts.push(e.text.as_str());
    }
    parts.join("\n")
}

fn merge_skip(e: &junit_parser::TestSkipped) -> String {
    let mut parts = Vec::new();
    if !e.skipped_type.is_empty() {
        parts.push(e.skipped_type.as_str());
    }
    if !e.message.is_empty() {
        parts.push(e.message.as_str());
    }
    if !e.text.is_empty() {
        parts.push(e.text.as_str());
    }
    parts.join("\n")
}

#[allow(non_snake_case)]
#[component]
pub fn TestResults() -> impl IntoView {
    let xml = expect_context::<LocalResource<Option<junit_parser::TestSuites>>>();
    let sorted_tests = move || {
        xml.read()
            .as_ref()
            .and_then(|sw| sw.clone().and_then(|ts| ts.suites.first().cloned()))
            .map(|c| {
                sort_tests(&c.cases)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    view! {
        <Suspense fallback=move || {
            view! { <div>Loading...</div> }
        }>
            {move || match xml.read().as_ref().and_then(|sw| sw.as_ref().map(|_| true)) {
                Some(_) => {
                    view! {
                        <Accordion>
                            <For
                                each=sorted_tests
                                key=move |c| c.name.clone()
                                children=move |c| {
                                    let status = junit_status_to_status(c.status.clone());
                                    let header = c.name.clone();
                                    let duration = c.time;
                                    let id = c.name.clone();
                                    let mut message = match c.status {
                                        junit_parser::TestStatus::Error(e) => merge_error(&e),
                                        junit_parser::TestStatus::Failure(e) => merge_fail(&e),
                                        junit_parser::TestStatus::Skipped(e) => merge_skip(&e),
                                        junit_parser::TestStatus::Success => "SUCCESS".into(),
                                    };
                                    let mut parts = vec![message];
                                    if let Some(out) = &c.system_out && !out.is_empty() {
                                        parts.push(out.clone());
                                    }
                                    if let Some(err) = &c.system_err && !err.is_empty() {
                                        parts.push(err.clone());
                                    }
                                    let message = parts.join("\n");
                                    view! {
                                        <AccordionItem
                                            header_class="w-full"
                                            hide=true
                                            header=move || {
                                                view! {
                                                    <div
                                                        id=id.clone()
                                                        class="flex justify-between items-center"
                                                    >
                                                        <span class="flex items-center">
                                                            <StatusIcon class="h-4 w-4" status=status.into() />
                                                            <h3 class="p-2">{header.clone()}</h3>
                                                        </span>
                                                        <div class="text-gray-400 text-xs pl-2 float-right">
                                                            {format!("{duration:.2}s")}
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        >

                                            <div>
                                                <ShellOut text=message />
                                            </div>
                                        </AccordionItem>
                                    }
                                        .into_any()
                                }
                            />

                        </Accordion>
                    }
                        .into_any()
                }
                _ => view! { <div></div> }.into_any(),
            }}

        </Suspense>
    }
}
