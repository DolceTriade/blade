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
use prost::Message;
use regex::Regex;
use scopeguard::defer;
use state::DBManager;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status, transport::Server};
use tracing::{Instrument, Level, instrument, span};

mod buildinfo;
mod options;
mod print_event;
mod progress;
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
    static ref MESSAGE_HANDLER_ERRORS: Counter::<u64> = metrics::register_metric(
        "blade_bep_message_handler_errors",
        "Total number of errors returned by the message handlers",
        Counter::default()
    );
    static ref ACTIVE_STREAMS: Gauge::<u32, AtomicU32> = metrics::register_metric(
        "blade_bep_active_streams",
        "Total number of active streams",
        Gauge::default()
    );
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

fn session_result(
    db_mgr: &dyn DBManager,
    invocation_id: &str,
    success: bool,
) -> anyhow::Result<()> {
    let mut db = db_mgr.get()?;
    db.update_shallow_invocation(
        invocation_id,
        Box::new(move |i: &mut state::InvocationResults| {
            match success {
                true => i.status = state::Status::Success,
                false => i.status = state::Status::Fail,
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
            let mut session_uuid = "".to_string();
            let span = tracing::span::Span::current();
            TOTAL_STREAMS.inc();
            ACTIVE_STREAMS.inc();
            defer! {
                ACTIVE_STREAMS.dec();
            }
            loop {
                let maybe_message = in_stream
                    .message()
                    .await
                    .and_then(|msg| {
                        msg.ok_or(Status::invalid_argument(
                            "empty PublishBuildToolEventStreamRequest",
                        ))
                    })
                    .and_then(|msg| {
                        msg.ordered_build_event
                            .ok_or(Status::invalid_argument("empty OrderedBuildEvent"))
                    });
                match maybe_message {
                    Ok(obe) => {
                        let mut build_ended = false;
                        let stream_id_or = obe.stream_id.as_ref();
                        if stream_id_or.is_none() {
                            tracing::warn!("missing stream id");
                            TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::InvalidArgument.into() }).inc();
                            return;
                        }
                        let Some(uuid) = stream_id_or.map(|id| id.invocation_id.clone()) else {
                            continue;
                        };
                        if session_uuid.is_empty() {
                            session_uuid = uuid.clone();
                            span.record("session_uuid", &session_uuid);
                            tracing::info!("Stream started");
                            let mut already_over = false;
                            if let Ok(mut db) = global.db_manager.as_ref().get() {
                                if let Ok(inv) = db.get_shallow_invocation(&session_uuid) {
                                    if let Some(end) = inv.end {
                                        if std::time::SystemTime::now()
                                            .duration_since(end)
                                            .unwrap_or(std::time::Duration::from_secs(0))
                                            > global.session_lock_time
                                        {
                                            already_over = true;
                                        }
                                    }
                                }
                            }
                            if already_over {
                                tracing::warn!("session already ended");

                                let _ = tx
                                    .send(Err(Status::new(
                                        tonic::Code::FailedPrecondition,
                                        "session already ended",
                                    )))
                                    .await
                                    .ok();
                                TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::FailedPrecondition.into() }).inc();
                                return;
                            }

                            if let Ok(mut db) = global.db_manager.as_ref().get() {
                                let _ = db
                                    .update_shallow_invocation(
                                        &session_uuid,
                                        Box::new(|i: &mut state::InvocationResults| {
                                            i.status = state::Status::InProgress;
                                            Ok(())
                                        }),
                                    )
                                    .ok();
                            }
                        }

                        let Some(event) = obe.event.as_ref().and_then(|event| event.event.as_ref())
                        else {
                            continue;
                        };
                        match event {
                            build_event::Event::BazelEvent(any) => {
                                let be_or = build_event_stream::BuildEvent::decode(&any.value[..]);
                                if be_or.is_err() {
                                    let err_str =  format!("invalid event: {:#?}", be_or.unwrap_err());
                                    {
                                        let _ = unexpected_cleanup_session(
                                            global.db_manager.as_ref(),
                                            &uuid,
                                        )
                                        .map_err(|e| {
                                            tracing::error!(
                                                "{session_uuid}: error closing stream: {e:#?}"
                                            )
                                        });
                                    }
                                    tracing::error!("{}", err_str);
                                    let _ = tx
                                    .send(Err(Status::new(tonic::Code::InvalidArgument, err_str)))
                                    .await
                                    .ok();
                                    drop(tx);
                                    TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::InvalidArgument.into() }).inc();
                                    return;
                                }
                                let Ok(be) = be_or else {
                                    continue;
                                };
                                match be.payload.as_ref() {
                                    Some(build_event_stream::build_event::Payload::Finished(f)) => {
                                        let success = f
                                            .exit_code
                                            .as_ref()
                                            .unwrap_or(
                                                &build_event_stream::build_finished::ExitCode {
                                                    name: "idk".into(),
                                                    code: 1,
                                                },
                                            )
                                            .code
                                            == 0;
                                        let _ = session_result(
                                            global.db_manager.as_ref(),
                                            &uuid,
                                            success,
                                        )
                                        .map_err(|e| {
                                            tracing::error!(
                                                "{session_uuid}: error closing stream: {e:#?}"
                                            )
                                        });
                                        if !success {
                                            TOTAL_STREAMS_ERRORS.get_or_create(&ErrorLabels { code: tonic::Code::Unknown.into() }).inc();
                                        }
                                    }
                                    Some(_) => {
                                        for v in &*handlers {
                                            if let Err(e) = v.handle_event(
                                                global.db_manager.as_ref(),
                                                &uuid,
                                                &be,
                                            ) {
                                                tracing::warn!("{:#?}", e);
                                                MESSAGE_HANDLER_ERRORS.inc();
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            build_event::Event::ComponentStreamFinished(_) => {
                                build_ended = true;
                            }
                            _ => {}
                        }
                        let send_fail = tx
                            .send(Ok(PublishBuildToolEventStreamResponse {
                                sequence_number: obe.sequence_number,
                                stream_id: obe.stream_id.clone(),
                            }))
                            .await
                            .inspect_err(|e| {
                                tracing::warn!("failed to send message: {:#?}", e)
                            })
                            .map(|_| false)
                            .unwrap_or(true);
                        if build_ended || send_fail {
                            tracing::info!("Build over");
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
                        if !session_uuid.is_empty() {
                            let _ = unexpected_cleanup_session(
                                global.db_manager.as_ref(),
                                &session_uuid,
                            )
                            .map_err(|e| {
                                tracing::error!("error closing stream: {e:#?}")
                            });
                        }

                        drop(tx);
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
