use build_event_stream_proto::build_event_stream;
use crate::EventHandler;

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        db_mgr: &dyn state::DBManager,
        invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        if let Some(build_event_stream::build_event::Payload::BuildToolLogs(logs)) = &event.payload {
            // Look for command.profile.gz in the log files
            for log in &logs.log {
                if log.name == "command.profile.gz"
                    && let Some(build_event_stream::file::File::Uri(uri)) = &log.file
                {
                    // Update the invocation with the profile URI
                    let mut db = db_mgr.get()?;
                    let uri2 = uri.clone();
                    db.update_shallow_invocation(
                        invocation_id,
                        Box::new(move |i: &mut state::InvocationResults| {
                            i.profile_uri = Some(uri2);
                            Ok(())
                        }),
                    )?;
                    break;
                }
            }
        }
        Ok(())
    }
}
