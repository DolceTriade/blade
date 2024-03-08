use anyhow::Context;
use anyhow::Result;
use build_event_stream_proto::*;
use build_proto::google::devtools::build::v1::*;
use prost::Message;
use regex::Regex;
use state::DBManager;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Response, Status};
use tracing::instrument;
use tracing::span;
use tracing::Instrument;
use tracing::Level;

mod buildinfo;
mod options;
mod print_event;
mod progress;
mod target;

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
                }
                _ => {}
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
                                    tracing::error!(
                                        "invalid event: {:#?}",
                                        be_or.unwrap_err()
                                    );
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
                                    }
                                    Some(_) => {
                                        for v in &*handlers {
                                            if let Err(e) = v.handle_event(
                                                global.db_manager.as_ref(),
                                                &uuid,
                                                &be,
                                            ) {
                                                tracing::warn!("{:#?}", e);
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
                            tracing::warn!("Client closed stream. Closing session.")
                        } else {
                            tracing::error!("Error: {}", err);
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
        .add_service(
            publish_build_event_server::PublishBuildEventServer::new(server)
                .max_decoding_message_size(10 * 1024 * 1024),
        )
        .add_service(reflect)
        .serve(host)
        .await
        .context("error starting grpc server")
}

pub struct RegexHandle {
    enabled: AtomicBool,
    regex: Mutex<regex::Regex>,
}

impl RegexHandle {
    pub fn new(re: &str) -> anyhow::Result<Self> {
        Ok(Self {
            enabled: AtomicBool::new(!re.is_empty()),
            regex: Mutex::new(regex::Regex::new(re)?),
        })
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn update_regex(&mut self, re: &str) -> anyhow::Result<()> {
        if re.is_empty() {
            self.enabled
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
        let mut r = self.regex.lock().unwrap();
        *r = regex::Regex::new(re)?;
        self.enabled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn is_match(&self, text: &str) -> bool {
        if !self.enabled.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }
        let r = self.regex.lock().unwrap();
        r.is_match(text)
    }
}
