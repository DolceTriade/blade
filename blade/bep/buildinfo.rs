use anyhow::Context;
use build_event_stream_proto::build_event_stream;
use state::DBManager;

pub(crate) struct Handler {}

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
                    start: p
                        .start_time
                        .as_ref()
                        .and_then(|s| prototime::timestamp::from_proto(s).ok())
                        .unwrap_or_else(std::time::SystemTime::now),
                    command: p.command.clone(),
                    ..Default::default()
                };
                db.upsert_shallow_invocation(&invocation)
                    .context("failed to insert invocation")?;
            },
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
                    .unwrap_or_default()
                    .to_vec();
                db.update_shallow_invocation(
                    invocation_id,
                    Box::new(move |i: &mut state::InvocationResults| {
                        i.pattern = pattern;
                        Ok(())
                    }),
                )?;
            },
            _ => {},
        }

        Ok(())
    }
}
