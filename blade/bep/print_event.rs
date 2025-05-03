use std::sync::{Arc, Mutex};

use build_event_stream_proto::build_event_stream;
use prost_reflect::ReflectMessage;
use regex::Regex;
use state::DBManager;
pub(crate) struct Handler {
    pub message_re: Arc<Mutex<Regex>>,
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        _db_mgr: &dyn DBManager,
        _invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        let re = self.message_re.lock().unwrap();
        if re.as_str().is_empty() {
            return Ok(());
        }
        let desc = event.descriptor();
        let dm = event.transcode_to_dynamic();
        let oneof = match desc.oneofs().next() {
            None => {
                return Ok(());
            },
            Some(o) => o,
        };
        let _ = oneof.fields().try_for_each(|f| {
            if dm.has_field(&f) && re.is_match(f.field_descriptor_proto().type_name()) {
                let type_name = f.field_descriptor_proto().name();
                let j = serde_json::ser::to_string(&dm).map_err(|_| ())?;
                tracing::info!(type_name, "{}", j);
                return Err(());
            }
            Ok(())
        });
        Ok(())
    }
}
