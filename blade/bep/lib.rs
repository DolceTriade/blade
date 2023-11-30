use anyhow::Context;
use anyhow::Result;
use build_event_stream_proto::{build_event_stream::File, *};
use build_proto::build;
use build_proto::google::devtools::build::v1::*;
use futures;
use futures::lock::Mutex;
use lazy_static::lazy_static;
use log;
use pretty_env_logger;
use prost::Message;
use prost_reflect::ReflectMessage;
use prost_reflect::{DescriptorPool, DynamicMessage};
use prost_types::FileDescriptorSet;
use proto_registry;
use runfiles::Runfiles;
use scopeguard;
use scopeguard::defer;
use serde_json;
use state;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{
    default,
    error::Error,
    fs,
    net::ToSocketAddrs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status, Streaming};

mod progress;
mod target;

lazy_static! {
    static ref HANDLERS: Vec<Box<dyn EventHandler + Sync + Send>> = vec![Box::new(progress::Handler {}), Box::new(target::Handler {})];
}

trait EventHandler {
    fn handle_event(
        &self,
        invocation: &mut state::InvocationResults,
        event: &build_event_stream::BuildEvent,
    ) -> anyhow::Result<()>;
}

pub struct BuildEventService {
    state: Arc<state::Global>,
    
}

fn proto_name(s: &str) -> &str {
    if let Some(idx) = s.find("/") {
        return &s[idx + 1..];
    }
    s
}

fn build_over() -> build_event_stream::BuildEvent {
    let mut be: build_event_stream::BuildEvent = build_event_stream::BuildEvent::default();
    be.payload = Some(build_event_stream::build_event::Payload::Finished(
        build_event_stream::BuildFinished {
            ..Default::default()
        },
    ));
    be
}

fn build_aborted() -> build_event_stream::BuildEvent {
    let mut be: build_event_stream::BuildEvent = build_event_stream::BuildEvent::default();
    be.payload = Some(build_event_stream::build_event::Payload::Aborted(
        build_event_stream::Aborted {
            reason: 0,
            description: "idk".into(),
        },
    ));
    be
}

fn unexpected_cleanup_session(invocation: &mut state::Invocation) {
    if invocation.results.success.is_none() {
        invocation.results.success = Some(false);
    }
    drop(&invocation.tx);
}

async fn get_session(global: Arc<state::Global>, uuid: String) -> Arc<Mutex<state::Invocation>> {
    let mut map = global.sessions.lock().await;
    if let Some(invocation) = map.get(&uuid) {
        return invocation.clone();
    }
    let invocation = Arc::new(Mutex::new(state::Invocation::default()));
    map.insert(uuid, invocation.clone());
    invocation
}

#[tonic::async_trait]
impl publish_build_event_server::PublishBuildEvent for BuildEventService {
    async fn publish_lifecycle_event(
        &self,
        request: tonic::Request<
            build_proto::google::devtools::build::v1::PublishLifecycleEventRequest,
        >,
    ) -> std::result::Result<tonic::Response<empty_proto::google::protobuf::Empty>, tonic::Status>
    {
        return Ok(Response::new(empty_proto::google::protobuf::Empty {}));
    }

    type PublishBuildToolEventStreamStream =
        ReceiverStream<Result<PublishBuildToolEventStreamResponse, Status>>;

    async fn publish_build_tool_event_stream(
        &self,
        request: tonic::Request<tonic::Streaming<PublishBuildToolEventStreamRequest>>,
    ) -> std::result::Result<tonic::Response<Self::PublishBuildToolEventStreamStream>, tonic::Status>
    {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let global = self.state.clone();
        tokio::spawn(async move {
            while let maybe_message = in_stream.message().await {
                match maybe_message {
                    Ok(res) => {
                        if let Some(v) = res {
                            if let Some(obe) = &v.ordered_build_event {
                                let mut buildEnded = false;
                                let stream_id_or = obe.stream_id.as_ref();
                                if stream_id_or.is_none() {
                                    log::warn!("missing stream id");
                                    return;
                                }
                                let uuid = stream_id_or.unwrap().invocation_id.clone();
                                let session = get_session(global.clone(), uuid.clone()).await;
                                let event = obe.event.as_ref().unwrap().event.as_ref().unwrap();
                                match event {
                                    build_event::Event::BazelEvent(any) => {
                                        let be_or =
                                            build_event_stream::BuildEvent::decode(&any.value[..]);
                                        if be_or.is_err() {
                                            log::error!(
                                                "{} invalid event: {:#?}",
                                                uuid,
                                                be_or.unwrap_err()
                                            );
                                            {
                                                let mut s = session.lock().await;
                                                unexpected_cleanup_session(&mut s);
                                            }
                                            return;
                                        }
                                        let be = be_or.unwrap();
                                        match be.payload.as_ref().unwrap() {
                                            build_event_stream::build_event::Payload::Finished(
                                                f,
                                            ) => {
                                                let mut s = session.lock().await;
                                                let success = f.exit_code.as_ref().unwrap_or(&build_event_stream::build_finished::ExitCode { name: "idk".into(), code: 1 }).code == 0;
                                                s.results.success = Some(success);
                                                s.tx.send(()).await;
                                            }
                                            _ => {
                                                for v in &*HANDLERS {
                                                    let mut s = session.lock().await;
                                                    match v.handle_event(&mut s.results, &be) {
                                                        Err(e) => log::warn!("{:#?}", e),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    build_event::Event::ComponentStreamFinished(end) => {
                                        buildEnded = true;
                                    }
                                    _ => {
                                        //log::info!("Got other event: {:#?}", event)
                                    }
                                }
                                tx.send(Ok(PublishBuildToolEventStreamResponse {
                                    sequence_number: obe.sequence_number,
                                    stream_id: obe.stream_id.clone(),
                                }))
                                .await
                                .or_else(|_| {
                                    buildEnded = true;
                                    Err(())
                                });
                                if buildEnded {
                                    log::error!("BUILD OVER");
                                    drop(tx);
                                    return;
                                }
                            } else {
                                log::info!("Party over");
                                return;
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Error: {}", err);
                        return;
                    }
                }
            }
        });
        let out_stream = ReceiverStream::new(rx);
        return Ok(Response::new(out_stream));
    }
}

pub async fn run_bes_grpc(host: SocketAddr, state: Arc<state::Global>) -> Result<()> {
    let reflect = tonic_reflection::server::Builder::configure()
        .register_file_descriptor_set(*proto_registry::DESCRIPTORS.clone())
        .build()?;

    proto_registry::init_global_descriptor_pool()?;
    let server = BuildEventService {
        state,
    };
    Server::builder()
        .add_service(publish_build_event_server::PublishBuildEventServer::new(
            server,
        ))
        .add_service(reflect)
        .serve(host)
        .await
        .context("error starting grpc server")
}
