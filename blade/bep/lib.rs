#![allow(clippy::result_large_err)]

use std::{
    fmt::Write,
    net::SocketAddr,
    sync::{Arc, Mutex, atomic::AtomicU32},
};

use anyhow::{Context, Result};
use build_event_stream_proto::*;
use build_proto::google::devtools::build::v1::*;
use lazy_static::lazy_static;
use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family, gauge::Gauge},
};
use regex::Regex;
use scopeguard::defer;
use state::DBManager;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status, transport::Server};
use tracing::{Instrument, Level, instrument, span};

mod buildinfo;
mod buildtoollogs;
mod options;
mod print_event;
mod progress;
mod session;
mod target;

lazy_static! {
    static ref TOTAL_STREAMS: Counter::<u64> = metrics::register_metric(
        "blade_bep_streams",
        "Total number of streams",
        Counter::default()
    );
    static ref TOTAL_STREAMS_ERRORS: Family::<ErrorLabels, Counter> = metrics::register_metric(
        "blade_bep_stream_errors",
        "Total number of stream errors",
        Family::default()
    );
    static ref ACTIVE_STREAMS: Gauge::<u32, AtomicU32> = metrics::register_metric(
        "blade_bep_active_streams",
        "Total number of active streams",
        Gauge::default()
    );
}

enum BuildState {
    BuildInProgress,
    BuildFinished,
}

struct ProccessedEvent {
    obe: OrderedBuildEvent,
}

trait EventHandler {
    fn handle_event(
        &self,
        db_mgr: &dyn DBManager,
        invocation_id: &str,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()>;
}

pub struct BuildEventService {
    state: Arc<state::Global>,
    handlers: Arc<Vec<Box<dyn EventHandler + Sync + Send>>>,
}

fn unexpected_cleanup_session(db_mgr: &dyn DBManager, invocation_id: &str) -> anyhow::Result<()> {
    let mut db = db_mgr.get()?;
    db.update_shallow_invocation(
        invocation_id,
        Box::new(move |i: &mut state::InvocationResults| {
            match i.status {
                state::Status::InProgress | state::Status::Unknown => {
                    i.status = state::Status::Fail
                },
                _ => {},
            }
            i.end = Some(std::time::SystemTime::now());
            Ok(())
        }),
    )?;
    Ok(())
}

#[tonic::async_trait]
impl publish_build_event_server::PublishBuildEvent for BuildEventService {
    type PublishBuildToolEventStreamStream =
        ReceiverStream<Result<PublishBuildToolEventStreamResponse, Status>>;

    #[instrument(skip_all)]
    async fn publish_lifecycle_event(
        &self,
        _request: tonic::Request<
            build_proto::google::devtools::build::v1::PublishLifecycleEventRequest,
        >,
    ) -> std::result::Result<tonic::Response<empty_proto::google::protobuf::Empty>, tonic::Status>
    {
        return Ok(Response::new(empty_proto::google::protobuf::Empty {}));
    }

    #[instrument(skip_all)]
    async fn publish_build_tool_event_stream(
        &self,
        request: tonic::Request<tonic::Streaming<PublishBuildToolEventStreamRequest>>,
    ) -> std::result::Result<tonic::Response<Self::PublishBuildToolEventStreamStream>, tonic::Status>
    {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let global = self.state.clone();
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            TOTAL_STREAMS.inc();
            ACTIVE_STREAMS.inc();
            defer! {
                ACTIVE_STREAMS.dec();
                tracing::info!("Stream ended.");
            }
            let mut session = session::BESSession::new(handlers, global.clone());
            loop {
                let Ok(msg) = tokio::time::timeout(std::time::Duration::from_secs(60), in_stream.message()).await else {
                    tracing::warn!("Timeout waiting for message for {}, skipping.", session.invocation_id());
                    let _ = tx.send(Err(tonic::Status::deadline_exceeded("failed to wait for timeout"))).await;
                    return;
                };

                match msg.and_then(|msg| session.process_message(msg)) {
                    Ok(out) => {
                        if out.obe.event.is_none() {
                            if !session.is_build_over() {
                                tracing::warn!("Received empty event for {}, skipping.", session.invocation_id());
                                let _ = tx.send(Err(tonic::Status::invalid_argument("empty event"))).await;
                            }
                            return;
                        }
                        if let Err(e) = tx.send(Ok(PublishBuildToolEventStreamResponse { stream_id: out.obe.stream_id.clone(), sequence_number: out.obe.sequence_number })).await {
                            tracing::error!("Error sending response, aborting: {:#?}", e);
                                    TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::Aborted.into() }).inc();
                            return;
                        }
                    }
                    Err(err) => {
                        // Tonic gives us this scary message for disconnects. It's really just a disconnect.
                        if err.message().contains("error reading a body from connection: stream closed because of a broken pipe") {
                            tracing::warn!("Client closed stream. Closing session.");
                            TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::Aborted.into() }).inc();
                        } else {
                            tracing::error!("Error: {}", err);
                            TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: err.code().into() }).inc();
                        }
                        if !session.invocation_id().is_empty() {
                            let _ = unexpected_cleanup_session(
                                global.db_manager.as_ref(),
                                session.invocation_id(),
                            )
                            .map_err(|e| {
                                tracing::error!("error closing stream: {e:#?}")
                            });
                        }
                        return;
                    }
                }
            }
        }.instrument(span!(Level::INFO, "bep_grpc_stream", "session_uuid" = tracing::field::Empty)));
        let out_stream = ReceiverStream::new(rx);
        return Ok(Response::new(out_stream));
    }
}

#[instrument(skip_all)]
pub async fn run_bes_grpc(
    host: SocketAddr,
    state: Arc<state::Global>,
    print_message_re: Arc<Mutex<Regex>>,
) -> Result<()> {
    let reflect = tonic_reflection::server::Builder::configure()
        .register_file_descriptor_set(*proto_registry::DESCRIPTORS.clone())
        .build()?;
    let handlers: Vec<Box<dyn EventHandler + Sync + Send>> = vec![
        Box::new(progress::Handler {}),
        Box::new(target::Handler {}),
        Box::new(buildinfo::Handler {}),
        Box::new(buildtoollogs::Handler {}),
        Box::new(options::Handler {}),
        Box::new(print_event::Handler {
            message_re: print_message_re,
        }),
    ];
    proto_registry::init_global_descriptor_pool()?;
    let server = BuildEventService {
        state,
        handlers: Arc::new(handlers),
    };
    Server::builder()
        .tcp_keepalive(Some(std::time::Duration::from_secs(20)))
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(20)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(30)))
        .add_service(
            publish_build_event_server::PublishBuildEventServer::new(server)
                .max_decoding_message_size(10 * 1024 * 1024),
        )
        .add_service(reflect)
        .serve(host)
        .await
        .context("error starting grpc server")
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ErrorLabels {
    code: StatusCode,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct StatusCode(tonic::Code);

impl EncodeLabelValue for StatusCode {
    fn encode(
        &self,
        encoder: &mut prometheus_client::encoding::LabelValueEncoder,
    ) -> std::prelude::v1::Result<(), std::fmt::Error> {
        encoder.write_str(&format!("{:#?}", self.0))
    }
}

impl From<tonic::Code> for StatusCode {
    fn from(value: tonic::Code) -> Self { Self(value) }
}
