use anyhow::Context;
use build_event_stream_proto::build_event_stream;
use state::DBManager;
use std::time::Duration;
use timestamp_proto::google::protobuf::Timestamp;

pub(crate) struct Handler {}

fn time(ts: &Option<Timestamp>) -> std::time::SystemTime {
    ts.as_ref()
        .and_then(|ts| {
            if ts.seconds < 0 || ts.nanos < 0 {
                return None;
            }
            let secs = ts.seconds as u64;
            let nanos: u64 = ts.nanos as u64;
            std::time::UNIX_EPOCH
                .checked_add(Duration::from_secs(secs) + Duration::from_nanos(nanos))
        })
        .unwrap_or(std::time::UNIX_EPOCH)
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        db_mgr: &dyn DBManager,
        invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        match &event.payload {
            Some(build_event_stream::build_event::Payload::Started(p)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let invocation = state::InvocationResults {
                    id: p.uuid.clone(),
                    start: time(&p.start_time),
                    command: p.command.clone(),
                    ..Default::default()
                };
                db.upsert_shallow_invocation(&invocation).context("failed to insert invocation")?;
            }
            Some(build_event_stream::build_event::Payload::Expanded(_)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let pattern = event
                .id
                .as_ref()
                .and_then(|id| {
                    if let Some(build_event_stream::build_event_id::Id::Pattern(id)) = &id.id {
                        return Some(id.pattern.clone());
                    }
                    None
                })
                .unwrap_or_default().to_vec();
                db.update_shallow_invocation(invocation_id, Box::new(move|i: &mut state::InvocationResults| {
                    i.pattern = pattern;
                    Ok(())
                }))?;
            }
            _ => {}
        }

        Ok(())
    }
}
