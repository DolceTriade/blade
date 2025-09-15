use std::time::Duration;

use anyhow::{Context, Result};
use build_event_stream_proto::*;
use build_proto::google::devtools::build::v1 as bes;
use clap::Parser;
use futures::{StreamExt, stream};
use prost::Message as _;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    Request,
    transport::{Channel, Endpoint},
};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Parser, Debug, Clone)]
#[command(name = "streamstress", version, about = "BES gRPC stream stress tool")]
struct Args {
    /// Server address, e.g. http://127.0.0.1:8980 or https://host:port
    #[arg(short = 'a', long = "addr", default_value = "http://127.0.0.1:8980")]
    addr: String,

    /// Number of concurrent streams to start
    #[arg(short = 'n', long = "streams", default_value_t = 100u32)]
    streams: u32,

    /// Number of messages per stream (including start and finish)
    #[arg(short = 'm', long = "messages", default_value_t = 100u32)]
    messages_per_stream: u32,

    /// Bytes of random stderr/stdout per progress event
    #[arg(short = 's', long = "size", default_value_t = 256u32)]
    payload_size: u32,

    /// Delay between messages per stream in milliseconds
    #[arg(short = 'd', long = "delay_ms", default_value_t = 0u64)]
    delay_ms: u64,

    /// Max concurrent requests in the stream send loop
    #[arg(long = "concurrency", default_value_t = 100u32)]
    concurrency: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let args = Args::parse();
    info!(?args, "starting streamstress");

    let payload = vec![b'X'; args.payload_size as usize];

    // Launch streams with bounded concurrency to avoid overwhelming the local
    // process
    let stream_count = args.streams;
    let concurrency = args.concurrency as usize;
    let results = stream::iter(0..stream_count)
        .map(|i| {
            let args = args.clone();
            let payload = payload.clone();
            async move {
                let channel = build_channel(&args).await?;
                let client = bes::publish_build_event_client::PublishBuildEventClient::new(channel);
                if let Err(e) = run_single_stream(client, i, &args, &payload).await {
                    warn!(stream = i, error = ?e, "stream failed");
                    return Err(e);
                }
                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    let failures = results.iter().filter(|r| r.is_err()).count();
    info!(streams = stream_count, failures, "completed");
    Ok(())
}

async fn build_channel(args: &Args) -> Result<Channel> {
    let endpoint = Endpoint::from_shared(args.addr.clone())?
        .tcp_keepalive(Some(Duration::from_secs(20)))
        .http2_keep_alive_interval(Duration::from_secs(20))
        .keep_alive_timeout(Duration::from_secs(30));

    Ok(endpoint.connect().await?)
}

async fn run_single_stream(
    mut client: bes::publish_build_event_client::PublishBuildEventClient<Channel>,
    stream_index: u32,
    args: &Args,
    payload: &[u8],
) -> Result<()> {
    let invocation_id = Uuid::new_v4().to_string();

    // Build a streaming request channel
    let (tx, rx) = tokio::sync::mpsc::channel::<bes::PublishBuildToolEventStreamRequest>(128);
    let in_stream = ReceiverStream::new(rx);

    let response_stream = client
        .publish_build_tool_event_stream(Request::new(in_stream))
        .await
        .context("open stream")?
        .into_inner();

    // Spawn a task to drain responses to avoid backpressure deadlock
    tokio::spawn(async move {
        tokio::pin!(response_stream);
        while let Some(res) = response_stream.next().await {
            // Ignore responses; server acks sequence_numbers
            if res.is_err() {
                break;
            }
        }
    });

    // Send messages
    let mut seq: i64 = 1;
    let stream_id = bes::StreamId {
        invocation_id: invocation_id.clone(),
        build_id: String::new(),
        component: 0,
    };

    // 1) Send a BuildStarted event
    send_bazel_event(&tx, &stream_id, seq, build_started(), false).await?;
    seq += 1;

    // 2) Progress messages
    let progress_messages = args.messages_per_stream.saturating_sub(2); // start + finish
    for i in 0..progress_messages {
        let be = progress_event(i as i32, payload);
        send_bazel_event(&tx, &stream_id, seq, be, false).await?;
        seq += 1;
        if args.delay_ms > 0 {
            sleep(Duration::from_millis(args.delay_ms)).await;
        }
    }

    // 3) Send a BuildFinished event with last_message
    send_bazel_event(&tx, &stream_id, seq, build_finished(true), true).await?;

    // Close the sender
    drop(tx);
    info!(stream = stream_index, %invocation_id, "stream completed");
    Ok(())
}

async fn send_bazel_event(
    tx: &tokio::sync::mpsc::Sender<bes::PublishBuildToolEventStreamRequest>,
    stream_id: &bes::StreamId,
    sequence_number: i64,
    mut be: build_event_stream::BuildEvent,
    last_message: bool,
) -> Result<()> {
    be.last_message = last_message;
    let any = any_proto::google::protobuf::Any {
        type_url: "type.googleapis.com/build_event_stream.BuildEvent".to_string(),
        value: be.encode_to_vec(),
    };
    let request = bes::PublishBuildToolEventStreamRequest {
        ordered_build_event: Some(bes::OrderedBuildEvent {
            stream_id: Some(stream_id.clone()),
            sequence_number,
            event: Some(bes::BuildEvent {
                event_time: Some(timestamp_proto::google::protobuf::Timestamp {
                    seconds: 0,
                    nanos: 0,
                }),
                event: Some(bes::build_event::Event::BazelEvent(any)),
            }),
        }),
        ..Default::default()
    };
    tx.send(request).await.context("send event")?;
    Ok(())
}

fn build_started() -> build_event_stream::BuildEvent {
    build_event_stream::BuildEvent {
        id: Some(build_event_stream::BuildEventId {
            id: Some(build_event_stream::build_event_id::Id::Started(
                build_event_stream::build_event_id::BuildStartedId {},
            )),
        }),
        children: vec![],
        last_message: false,
        payload: Some(build_event_stream::build_event::Payload::Started(
            build_event_stream::BuildStarted {
                build_tool_version: String::from("streamstress"),
                command: String::from("test"),
                ..Default::default()
            },
        )),
    }
}

fn progress_event(count: i32, payload: &[u8]) -> build_event_stream::BuildEvent {
    build_event_stream::BuildEvent {
        id: Some(build_event_stream::BuildEventId {
            id: Some(build_event_stream::build_event_id::Id::Progress(
                build_event_stream::build_event_id::ProgressId {
                    opaque_count: count,
                },
            )),
        }),
        children: vec![],
        last_message: false,
        payload: Some(build_event_stream::build_event::Payload::Progress(
            build_event_stream::Progress {
                stdout: String::from_utf8_lossy(payload).to_string(),
                stderr: String::new(),
            },
        )),
    }
}

fn build_finished(success: bool) -> build_event_stream::BuildEvent {
    build_event_stream::BuildEvent {
        id: Some(build_event_stream::BuildEventId {
            id: Some(build_event_stream::build_event_id::Id::BuildFinished(
                build_event_stream::build_event_id::BuildFinishedId {},
            )),
        }),
        children: vec![],
        last_message: false,
        #[allow(deprecated)]
        payload: Some(build_event_stream::build_event::Payload::Finished(
            build_event_stream::BuildFinished {
                exit_code: Some(build_event_stream::build_finished::ExitCode {
                    name: String::from("OK"),
                    code: if success { 0 } else { 1 },
                }),
                finish_time_millis: 0,
                finish_time: None,
                anomaly_report: None,
                failure_detail: None,
                overall_success: success,
            },
        )),
    }
}
