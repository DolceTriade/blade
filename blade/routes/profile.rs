use std::io::Read;

use components::{card::Card, charts::ganttchart::BazelTraceChart, summaryheader::SummaryHeader};
use leptos::{either::Either, prelude::*};
use trace_event_parser::{BazelTrace, TraceEventFile};

#[component]
pub fn BazelProfile() -> impl IntoView {
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();

    // Resource to fetch and parse the profile data
    let profile_data = LocalResource::new(move || async move {
        let profile_uri = invocation.with(|inv| inv.profile_uri.clone());

        match profile_uri {
            Some(uri) => {
                // Fetch the profile artifact
                let bytes = shared::get_artifact(uri)
                    .await
                    .map_err(|e| format!("Failed to fetch profile: {e}"))?;

                // Decompress gzip if needed
                let decompressed_bytes = if bytes.starts_with(&[0x1F, 0x8B]) {
                    // It's gzipped
                    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
                    let mut decompressed = Vec::new();
                    decoder
                        .read_to_end(&mut decompressed)
                        .map_err(|e| format!("Failed to decompress profile: {e}"))?;
                    decompressed
                } else {
                    bytes
                };

                // Parse JSON
                let json_str = String::from_utf8(decompressed_bytes)
                    .map_err(|e| format!("Failed to convert profile to UTF-8: {e}"))?;

                let trace_file = TraceEventFile::from_json(&json_str)
                    .map_err(|e| format!("Failed to parse profile JSON: {e}"))?;

                let bazel_trace = BazelTrace::from_trace_events(trace_file.trace_events);

                Ok(bazel_trace)
            },
            None => Err("No profile data available for this build".to_string()),
        }
    });

    view! {
        <div class="flex flex-col m-1 p-1 dark:bg-gray-800">
            <Card class="p-3 m-2">
                <SummaryHeader />
            </Card>

            <Suspense fallback=move || {
                view! { <div class="text-center py-8">"Loading profile data..."</div> }
            }>
                {move || Suspend::new(async move {
                    match profile_data.await {
                        Ok(bazel_trace) => {
                            Either::Left(
                                view! {
                                    <div class="h-[73dvh] overflow-auto">
                                        <h2 class="text-lg font-semibold mb-4">
                                            "Profile Timeline"
                                        </h2>
                                        <BazelTraceChart bazel_trace=bazel_trace />
                                    </div>
                                },
                            )
                        }
                        Err(error) => {
                            Either::Right(
                                view! {
                                    <div class="text-center py-8 text-red-500">
                                        <p class="font-semibold">"Error loading profile:"</p>
                                        <p class="text-sm mt-2">{error}</p>
                                    </div>
                                },
                            )
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
