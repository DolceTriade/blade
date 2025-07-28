use components::charts::ganttchart::BazelTraceChart;
use components::charts::ganttchart_canvas_hybrid::BazelTraceChartCanvasHybrid;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use leptos::either::Either;
use std::io::Read;
use trace_event_parser::{BazelTrace, TraceEventFile};

#[derive(PartialEq, Params)]
struct ProfileParams {
    id: Option<String>,
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params::<ProfileParams>();
    let id = move || params.with(|p| p.as_ref().ok().and_then(|p| p.id.clone()).unwrap_or_default());
    let invocation = expect_context::<RwSignal<state::InvocationResults>>();

    // Resource to fetch and parse the profile data
    let profile_data = LocalResource::new(move || async move {
        let profile_uri = invocation.with(|inv| inv.profile_uri.clone());

        match profile_uri {
            Some(uri) => {
                // Fetch the profile artifact
                let bytes = shared::get_artifact(uri).await.map_err(|e| format!("Failed to fetch profile: {e}"))?;

                // Decompress gzip if needed
                let decompressed_bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
                    // It's gzipped
                    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
                    let mut decompressed = Vec::new();
                    decoder.read_to_end(&mut decompressed)
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
            }
            None => Err("No profile data available for this build".to_string())
        }
    });

    view! {
        <div class="p-4">
            <div class="mb-4">
                <A href=move || format!("/invocation/{}", id())>
                    "← Back to Summary"
                </A>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="text-center py-8">
                        "Loading profile data..."
                    </div>
                }
            }>
                {move || Suspend::new(async move {
                    match profile_data.await {
                        Ok(bazel_trace) => {
                            Either::Left(view! {
                                <div>
                                    <h2 class="text-lg font-semibold mb-4">"Profile Timeline"</h2>
                                    <BazelTraceChartCanvasHybrid bazel_trace=bazel_trace />
                                </div>
                            })
                        }
                        Err(error) => {
                            Either::Right(view! {
                                <div class="text-center py-8 text-red-500">
                                    <p class="font-semibold">"Error loading profile:"</p>
                                    <p class="text-sm mt-2">{error}</p>
                                </div>
                            })
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
