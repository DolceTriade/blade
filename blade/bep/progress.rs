use anyhow::Context;
use build_event_stream_proto::build_event_stream;
use state::DBManager;
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
    let err_lines = clean_stderr.split('\n');
    let mut lines = orig_lines.into_iter().chain(err_lines).collect::<Vec<_>>();
    let mut to_remove = vec![];
    for (i, l) in lines.iter().enumerate().skip(orig_num_lines) {
        let c = l.matches("\x1b[1A\x1b[K").count();
        for j in 0..c {
            to_remove.push(i - 1 - j);
        }
    }

    for i in to_remove {
        if i >= lines.len() {
            log::warn!(
                "Tried to delete a line out of range: {} >= {}",
                i,
                lines.len()
            );
            continue;
        }
        lines[i] = "";
    }
    lines.insert(orig_num_lines, &clean_stdout);
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
        db_mgr: &dyn DBManager,
        invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()> {
        if let Some(build_event_stream::build_event::Payload::Progress(p)) = event.payload.as_ref()
        {
            if p.stderr.is_empty() && p.stdout.is_empty() {
                return Ok(());
            }
            let mut db = db_mgr.get().context("failed to get db handle")?;
            let output = db.get_progress(invocation_id)?;
            let progress = cleanup(&output, &p.stdout, &p.stderr);
            db.update_shallow_invocation(
                invocation_id,
                Box::new(move |i: &mut state::InvocationResults| {
                    i.output = progress;
                    Ok(())
                }),
            )
            .context("failed to update progress")?;
        }
        Ok(())
    }
}
