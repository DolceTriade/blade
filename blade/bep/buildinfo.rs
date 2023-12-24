use build_event_stream_proto::build_event_stream;
use timestamp_proto::google::protobuf::Timestamp;
use std::time::Duration;

pub(crate) struct Handler {}


fn time(ts: &Option<Timestamp>) -> std::time::SystemTime {
    ts.as_ref().and_then(|ts| {
        if ts.seconds < 0|| ts.nanos < 0 {
            return None;
        }
        let secs = ts.seconds as u64;
        let nanos: u64 = ts.nanos as u64;
        std::time::UNIX_EPOCH.checked_add(Duration::from_secs(secs) + Duration::from_nanos(nanos))
    }).unwrap_or(std::time::UNIX_EPOCH)
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        invocation: &mut state::InvocationResults,
        event: &build_event_stream_proto::build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        match &event.payload {
            Some(build_event_stream::build_event::Payload::Started(p)) => {
                invocation.id = p.uuid.clone();
                invocation.start = time(&p.start_time);
                invocation.command = p.command.clone();
            }
            Some(build_event_stream::build_event::Payload::Expanded(_)) => {
                invocation.pattern = event.id.as_ref().and_then(|id| {
                    if let Some(build_event_stream::build_event_id::Id::Pattern(id)) = &id.id {
                        return Some(id.pattern.clone());
                    }
                    None
                }).unwrap_or_default()
            }
            _=>{}
        }
        Ok(())
    }
}
