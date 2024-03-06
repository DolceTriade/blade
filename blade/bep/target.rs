use anyhow::Context;
use build_event_stream_proto::build_event_stream;
use state::DBManager;
use std::option::Option;

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
        Some(build_event_stream::build_event_id::Id::TestResult(t)) => &t.label,
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
        prototime::timestamp::from_proto(d).ok()
    };
    let s = start
        .and_then(convert)
        .unwrap_or_else(std::time::SystemTime::now);
    let e = end
        .and_then(convert)
        .unwrap_or_else(std::time::SystemTime::now);
    e.duration_since(s)
        .unwrap_or(std::time::Duration::from_secs(0))
}

fn proto_to_rust_duration(
    d: Option<&duration_proto::google::protobuf::Duration>,
) -> std::time::Duration {
    let convert =
        |d: &duration_proto::google::protobuf::Duration| prototime::duration::from_proto(d).ok();
    d.and_then(convert).unwrap_or_default()
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        db_mgr: &dyn DBManager,
        invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        match event.payload.as_ref() {
            Some(build_event_stream::build_event::Payload::Configured(target)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let label =
                    target_label(event).ok_or(anyhow::anyhow!("target not found: {event:#?}"))?;
                db.upsert_target(
                    invocation_id,
                    &state::Target {
                        name: label.to_string(),
                        status: state::Status::InProgress,
                        kind: target.target_kind.to_string(),
                        start: std::time::SystemTime::now(),
                        end: None,
                    },
                )
                .context("failed to insert target")?;
            }
            Some(build_event_stream::build_event::Payload::Completed(t)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                db.update_target_result(
                    invocation_id,
                    &label,
                    if t.success {
                        state::Status::Success
                    } else {
                        state::Status::Fail
                    },
                    std::time::SystemTime::now(),
                )
                .context("failed to update target result")?;
            }
            Some(build_event_stream::build_event::Payload::Aborted(a)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let label =
                    target_label(event).ok_or(anyhow::anyhow!("target not found: {event:#?}"))?;
                db.update_target_result(
                    invocation_id,
                    &label,
                    match build_event_stream::aborted::AbortReason::try_from(a.reason) {
                        Ok(build_event_stream::aborted::AbortReason::Skipped) => {
                            state::Status::Skip
                        }
                        _ => state::Status::Fail,
                    },
                    std::time::SystemTime::now(),
                )
                .context("failed to update target result")?;
            }
            Some(build_event_stream::build_event::Payload::TestSummary(summary)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let label =
                    target_label(event).ok_or(anyhow::anyhow!("target not found: {event:#?}"))?;
                let test = state::Test {
                    name: label.clone(),
                    status: match build_event_stream::TestStatus::try_from(summary.overall_status)?
                    {
                        build_event_stream::TestStatus::Passed => state::Status::Success,
                        _ => state::Status::Fail,
                    },
                    duration: to_duration(
                        summary.first_start_time.as_ref(),
                        summary.last_stop_time.as_ref(),
                    ),
                    end: std::time::SystemTime::now(),
                    runs: Default::default(),
                    num_runs: summary.run_count as usize,
                };
                db.upsert_test(invocation_id, &test)
                    .context("failed to insert test")?;
            }
            Some(build_event_stream::build_event::Payload::TestResult(r)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let mut info =
                    test_run_info(event).ok_or(anyhow::anyhow!("failed to find test id"))?;
                let mut test =
                    db.get_test(invocation_id, &info.0)
                        .unwrap_or_else(|_| state::Test {
                            name: info.0.clone(),
                            duration: Default::default(),
                            num_runs: 0,
                            runs: vec![],
                            end: std::time::SystemTime::now(),
                            status: state::Status::InProgress,
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
                let test_id = db
                    .upsert_test(invocation_id, &test)
                    .context("failed to update test")?;
                db.upsert_test_run(invocation_id, &test_id, &info.1)
                    .context("error inserting test run")?;
            }
            _ => {}
        }
        Ok(())
    }
}
