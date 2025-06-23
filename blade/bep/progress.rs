use anyhow::Context;
use build_event_stream_proto::build_event_stream;
use state::DBManager;

const DELETE_LINE_SEQ: &str = "\x1b[1A\x1b[K";

pub(crate) struct Handler {}

fn cleanup(stdout: &str, stderr: &str) -> (u32, Vec<String>) {
    let clean_stdout = stdout.replace('\r', "");
    let clean_stderr = stderr.replace('\r', "");
    let out_lines = clean_stdout.split('\n');
    let mut lines = clean_stderr
        .split('\n')
        .chain(out_lines)
        .map(String::from)
        .collect::<Vec<_>>();
    let mut to_remove = vec![];
    let mut database_remove = 0;
    for (i, l) in lines.iter_mut().enumerate() {
        // This logic assumes that this delete lines sequence is always on a line of its
        // own. This happens to be the case in Bazel.
        let c = l.matches(DELETE_LINE_SEQ).count();
        for j in 0..c {
            if i == 0 || i - 1 < j {
                database_remove = std::cmp::max(j + 1 - i, database_remove);
                continue;
            }
            to_remove.push(i - 1 - j);
        }
        if c > 0 {
            *l = l.replace(DELETE_LINE_SEQ, "");
        }
    }
    for i in to_remove {
        if i >= lines.len() {
            tracing::warn!(
                "Tried to delete a line out of range: {} >= {}",
                i,
                lines.len()
            );
            continue;
        }
        lines[i] = "".to_string();
    }
    let out = lines
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    (database_remove.try_into().unwrap(), out)
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
            let progress = cleanup(&p.stdout, &p.stderr);
            if progress.0 > 0 {
                // Try our best. If it doesn't work out, just log and continue.
                _ = db
                    .delete_last_output_lines(invocation_id, progress.0)
                    .inspect_err(|e| {
                        tracing::warn!(
                            "error deleting {} lines for invocation {}: {:#?}",
                            progress.0,
                            invocation_id,
                            e
                        );
                    });
            }
            db.insert_output_lines(invocation_id, progress.1)
                .context("failed to update progress")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::progress::{DELETE_LINE_SEQ, cleanup};

    fn make<S, T>(del: T, lines: &[S]) -> (u32, Vec<String>)
    where
        T: Into<u32>,
        S: ToString,
    {
        (del.into(), lines.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn test_cleanup() {
        assert_eq!(cleanup("a", "b"), make(0_u32, &["b", "a"]));
        assert_eq!(cleanup("", "b"), make(0_u32, &["b"]));
        assert_eq!(cleanup("a", ""), make(0_u32, &["a"]));
        assert_eq!(cleanup("", ""), (0_u32, Vec::new()));
        assert_eq!(
            cleanup(
                &("a\r\nb\r\n".to_owned() + DELETE_LINE_SEQ + DELETE_LINE_SEQ),
                "ab"
            ),
            make(0_u32, &["ab"])
        );
        assert_eq!(
            cleanup(
                &("a\r\nb\r\n".to_owned() + DELETE_LINE_SEQ + DELETE_LINE_SEQ),
                "ab"
            ),
            make(0_u32, &["ab"])
        );
        assert_eq!(
            cleanup(
                &("a\r\nb\r\n".to_owned() + DELETE_LINE_SEQ + DELETE_LINE_SEQ + DELETE_LINE_SEQ),
                "ab"
            ),
            (0, vec![])
        );
    }
}
