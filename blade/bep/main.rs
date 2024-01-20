use build_event_stream_proto::*;
use clap::*;
use prost::Message;

#[derive(Parser, Debug)]
#[command(name = "BES Output")]
#[command(about = "Bazel Build Event Service outputer")]
struct Args {
    #[arg(short = 'i', long = "input", value_name = "INPUT", default_value = "")]
    input: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let pb = std::fs::read(args.input)?;
    let mut pos = 0;
    let output = "".to_string();
    loop {
        if pos >= pb.len() {
            break;
        }
        match prost::decode_length_delimiter(&pb[pos..]) {
            Ok(size) => {
                if size == 0 {
                    println!("SIZE ZERO");
                    break;
                }
                let size_size = prost::length_delimiter_len(size);
                pos += size_size;
                let end = pos + size;
                let msg = match build_event_stream::BuildEvent::decode(&pb[pos..end]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("err={:#?} len={}", e, size);
                        pos += size;
                        continue;
                    }
                };
                pos += size;

                let Some(build_event_stream::build_event::Payload::Progress(p)) = msg.payload
                else {
                    continue;
                };
                if p.stderr.is_empty() && p.stdout.is_empty() {
                    continue;
                }
                if p.stdout.contains("/home/kubevirt/.cache/bazel/_bazel_kubevirt/36494e8116bcf28a1892091067514203/sandbox/linux-sandbox/1515/execroot/cloudn/nat_bpfel.o") {
                    println!("stdout = {:#?}", p.stdout);
                    println!("stderr = {:#?}", p.stderr);
                }
            }
            Err(e) => {
                println!("ERR: {:#?}", e);
                break;
            }
        }
    }
    println!("{}", output);
    Ok(())
}

#[allow(dead_code)]
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
    lines.insert(orig_num_lines + 1, &clean_stdout);
    lines
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .to_string()
}
