use build_event_stream_proto::build_event_stream;
use std::{option::Option, time::Duration};

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
) -> Option<(String, state::TestRun)> {
    let outer_id = event.id.as_ref()?;
    let id = outer_id.id.as_ref();
    let label = match id {
        Some(build_event_stream::build_event_id::Id::TestResult(t)) => (
            t.label.to_string(),
            state::TestRun {
                attempt: t.attempt,
                run: t.run,
                shard: t.shard,
                duration: Default::default(),
                files: Default::default(),
                details: Default::default(),
                status: state::Status::Unknown,
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
                let test = invocation
                    .tests
                    .entry(label.clone())
                    .or_insert_with(|| state::Test {
                        name: label.clone(),
                        status: state::Status::InProgress,
                        duration: Duration::default(),
                        runs: Default::default(),
                        num_runs: 0,
                    });
                test.status =
                    match build_event_stream::TestStatus::try_from(summary.overall_status)? {
                        build_event_stream::TestStatus::Passed => state::Status::Success,
                        _ => state::Status::Fail,
                    };
                test.duration = to_duration(
                    summary.first_start_time.as_ref(),
                    summary.last_stop_time.as_ref(),
                );
                test.num_runs = summary.run_count as usize;
            }
            Some(build_event_stream::build_event::Payload::TestResult(r)) => {
                let mut info =
                    test_run_info(event).ok_or(anyhow::anyhow!("failed to find test id"))?;
                let test = invocation
                    .tests
                    .entry(info.0.clone())
                    .or_insert_with(|| state::Test {
                        name: info.0.clone(),
                        status: state::Status::InProgress,
                        duration: Duration::default(),
                        runs: Default::default(),
                        num_runs: 0,
                    });
                r.test_action_output.iter().for_each(|f| {
                    if let Some(build_event_stream_proto::build_event_stream::file::File::Uri(
                        uri,
                    )) = &f.file
                    {
                        info.1.files.insert(
                            f.name.clone(),
                            state::Artifact {
                                size: f.length as usize,
                                uri: uri.clone(),
                            },
                        );
                    }
                });
                info.1.duration = proto_to_rust_duration(r.test_attempt_duration.as_ref());
                info.1.status = match build_event_stream::TestStatus::try_from(r.status)? {
                    build_event_stream::TestStatus::Passed => state::Status::Success,
                    _ => state::Status::Fail,
                };
                info.1.details = r.status_details.clone();
                test.num_runs = std::cmp::max(test.num_runs, info.1.run as usize);
                test.runs.push(info.1);
            }
            _ => {}
        }
        Ok(())
    }
}
