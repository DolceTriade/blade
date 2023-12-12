use build_event_stream_proto::build_event_stream;
use std::{collections::HashMap, option::Option, time::Duration};

pub(crate) struct Handler {}

fn target_label(
    event: &build_event_stream_proto::build_event_stream::BuildEvent,
) -> Option<String> {
    let outer_id = event.id.as_ref()?;
    let id = outer_id.id.as_ref();
    let label = match id {
        Some(build_event_stream::build_event_id::Id::TargetConfigured(t)) => &t.label,
        Some(build_event_stream::build_event_id::Id::TargetCompleted(t)) => &t.label,
        Some(build_event_stream::build_event_id::Id::TestSummary(t)) => &t.label,
        _ => {
            return None;
        }
    };
    Some(label.to_string())
}

fn test_run_info(
    event: &build_event_stream_proto::build_event_stream::BuildEvent,
) -> Option<(String, state::Run)> {
    let outer_id = event.id.as_ref()?;
    let id = outer_id.id.as_ref();
    let label = match id {
        Some(build_event_stream::build_event_id::Id::TestResult(t)) => (
            t.label.to_string(),
            state::Run {
                attempt: t.attempt,
                run: t.run,
                shard: t.shard,
            },
        ),
        _ => {
            return None;
        }
    };
    Some(label)
}

fn to_duration(
    start: Option<&::timestamp_proto::google::protobuf::Timestamp>,
    end: Option<&::timestamp_proto::google::protobuf::Timestamp>,
) -> std::time::Duration {
    let convert = |d: &::timestamp_proto::google::protobuf::Timestamp| {
        if d.seconds < 0 || d.nanos < 0 {
            return Default::default();
        }
        let nanos: u64 = d.nanos as u64;
        Duration::from_secs(d.seconds as u64) + Duration::from_nanos(nanos)
    };
    let s = start.map(convert).unwrap_or_default();
    let e = end.map(convert).unwrap_or_default();
    e - s
}

fn proto_to_rust_duration(
    d: Option<&duration_proto::google::protobuf::Duration>,
) -> std::time::Duration {
    let convert = |d: &duration_proto::google::protobuf::Duration| {
        if d.seconds < 0 || d.nanos < 0 {
            return Default::default();
        }
        let nanos: u64 = d.nanos as u64;
        Duration::from_secs(d.seconds as u64) + Duration::from_nanos(nanos)
    };
    d.map(convert).unwrap_or_default()
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        invocation: &mut state::InvocationResults,
        event: &build_event_stream_proto::build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        match event.payload.as_ref() {
            Some(build_event_stream::build_event::Payload::Configured(target)) => {
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                invocation.targets.insert(
                    label.to_string(),
                    state::Target {
                        name: label.to_string(),
                        status: state::Status::InProgress,
                        kind: target.target_kind.to_string(),
                        start: std::time::SystemTime::now(),
                        end: None,
                    },
                );
            }
            Some(build_event_stream::build_event::Payload::Completed(t)) => {
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                let target = invocation
                    .targets
                    .get_mut(&label)
                    .ok_or(anyhow::anyhow!("failed to find target {}", label))?;
                target.end = Some(std::time::SystemTime::now());
                target.status = if t.success {
                    state::Status::Success
                } else {
                    state::Status::Fail
                };
            }
            Some(build_event_stream::build_event::Payload::Aborted(a)) => {
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                let target = invocation
                    .targets
                    .get_mut(&label)
                    .ok_or(anyhow::anyhow!("failed to find target {}", label))?;
                target.end = Some(std::time::SystemTime::now());
                target.status = match build_event_stream::aborted::AbortReason::try_from(a.reason) {
                    Ok(build_event_stream::aborted::AbortReason::Skipped) => state::Status::Skip,
                    _ => state::Status::Fail,
                };
            }
            Some(build_event_stream::build_event::Payload::TestSummary(summary)) => {
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                invocation.tests.insert(
                    label.to_string(),
                    state::Test {
                        name: label.to_string(),
                        success: summary.overall_status
                            == build_event_stream::TestStatus::Passed as i32,
                        duration: to_duration(
                            summary.first_start_time.as_ref(),
                            summary.last_stop_time.as_ref(),
                        ),
                        runs: Default::default(),
                    },
                );
            }
            Some(build_event_stream::build_event::Payload::TestResult(r)) => {
                let info = test_run_info(event).ok_or(anyhow::anyhow!("failed to find test id"))?;
                let test = invocation
                    .tests
                    .get_mut(&info.0)
                    .ok_or(anyhow::anyhow!("failed to find test {}", &info.0))?;
                let mut files = HashMap::new();
                r.test_action_output.iter().for_each(|f| {
                    if let Some(build_event_stream_proto::build_event_stream::file::File::Uri(
                        uri,
                    )) = &f.file
                    {
                        files.insert(f.name.clone(), uri.clone());
                    }
                });
                test.runs.insert(
                    info.1,
                    state::TestRun {
                        duration: proto_to_rust_duration(r.test_attempt_duration.as_ref()),
                        files,
                    },
                );
            }
            _ => {}
        }
        Ok(())
    }
}
