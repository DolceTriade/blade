use build_event_stream_proto::build_event_stream;
use std::option::Option;

pub(crate) struct Handler {}

fn target_label(event: &build_event_stream_proto::build_event_stream::BuildEvent) -> Option<String> {
    let outer_id= event.id.as_ref()?;
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
            Some(build_event_stream::build_event::Payload::TestSummary(summary)) => {
                let label = target_label(event).ok_or(anyhow::anyhow!("target not found"))?;
                invocation.tests.insert(
                    label.to_string(),
                    state::Test {
                        name: label.to_string(),
                        success: summary.overall_status
                            == build_event_stream::TestStatus::Passed as i32,
                    },
                );
            }
            _ => {}
        }
        Ok(())
    }
}