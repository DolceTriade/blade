use build_event_stream_proto::build_event_stream;
pub(crate) struct Handler {}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        invocation: &mut state::InvocationResults,
        event: &build_event_stream_proto::build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        match event.payload.as_ref() {
            Some(build_event_stream::build_event::Payload::Progress(p)) => {
                invocation
                    .output
                    .push_str(&p.stdout.replace("\n\r", "\n"));
                invocation
                    .output
                    .push_str(&p.stderr.replace("\n\r", "\n"));
            }
            _ => {}
        }
        Ok(())
    }
}
