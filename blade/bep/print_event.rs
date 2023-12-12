use prost_reflect::ReflectMessage;
use regex::Regex;
pub(crate) struct Handler {
    pub message_re: Regex,
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        _invocation: &mut state::InvocationResults,
        event: &build_event_stream_proto::build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        let desc = event.descriptor();
        let dm = event.transcode_to_dynamic();
        let oneof = match desc.oneofs().next() {
            None => {
                return Ok(());
            }
            Some(o) => o,
        };
        let _ = oneof.fields().try_for_each(|f| {
            if dm.has_field(&f)
                && self
                    .message_re
                    .is_match(f.field_descriptor_proto().type_name())
            {
                let j = serde_json::ser::to_string(&dm).map_err(|_| ())?;
                log::info!("{}", j);
                return Err(());
            }
            Ok(())
        });
        Ok(())
    }
}
