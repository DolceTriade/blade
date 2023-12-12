use build_event_stream_proto::build_event_stream;
pub(crate) struct Handler {}

fn cleanup(orig: &str, stdout: &str, stderr: &str) -> String {
    let orig_lines = orig.split('\n').collect::<Vec<_>>();
    let orig_num_lines = if !orig.is_empty() {
        orig_lines.len()
    } else {
        0
    };
    let clean_stdout = stdout.replace('\r', "");
    let clean_stderr = stderr.replace('\r', "");
    let out_lines = clean_stdout.split('\n');
    let err_lines = clean_stderr.split('\n');
    let mut lines = orig_lines
        .into_iter()
        .chain(out_lines)
        .chain(err_lines)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mut to_remove = vec![];
    for (i, l) in lines.iter().enumerate().skip(orig_num_lines) {
        if i == 0 {
            continue;
        }
        let c = l.matches("\x1b[1A\x1b[K").count();
        if c > 0 {
            for j in 0..c {
                to_remove.push(i - 1 - j);
            }
        }
    }

    for i in to_remove {
        if i >= lines.len() {
            log::warn!("Tried to delete a line out of range: {} >= {}", i, lines.len());
            continue;
        }
        lines[i] = "";
    }
    lines
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .to_string()
}

impl crate::EventHandler for Handler {
    fn handle_event(
        &self,
        invocation: &mut state::InvocationResults,
        event: &build_event_stream_proto::build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        if let Some(build_event_stream::build_event::Payload::Progress(p)) = event.payload.as_ref()
        {
            if p.stderr.is_empty() && p.stdout.is_empty() {
                return Ok(());
            }
            invocation.output = cleanup(&invocation.output, &p.stdout, &p.stderr);
        }
        Ok(())
    }
}
