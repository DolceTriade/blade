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
            Some(build_event_stream::build_event::Payload::UnstructuredCommandLine(opts)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let o = state::BuildOptions {
                    unstructured: opts.args.clone(),
                    ..Default::default()
                };
                db.insert_options(invocation_id, &o)
                    .context("failed to insert unstructured command line")?;
            }
            Some(build_event_stream::build_event::Payload::OptionsParsed(opts)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let o = state::BuildOptions {
                    startup: opts.startup_options.clone(),
                    explicit_startup: opts.explicit_startup_options.clone(),
                    cmd_line: opts.cmd_line.clone(),
                    explicit_cmd_line: opts.explicit_cmd_line.clone(),
                    ..Default::default()
                };
                db.insert_options(invocation_id, &o)
                    .context("failed to insert parsed options")?;
            }
            Some(build_event_stream::build_event::Payload::BuildMetadata(md)) => {
                let mut db = db_mgr.get().context("failed to get db handle")?;
                let o = state::BuildOptions {
                    build_metadata: md.metadata.clone(),
                    ..Default::default()
                };
                db.insert_options(invocation_id, &o)
                    .context("failed to insert parsed options")?;
            }
            _ => {}
        }
        Ok(())
    }
}
