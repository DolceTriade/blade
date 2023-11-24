use anyhow::{Result};
use build_event_stream_proto::{build_event_stream::File, *};
use build_proto::google::devtools::build::v1::*;
use log;
use pretty_env_logger;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage};
use prost_types::FileDescriptorSet;
use proto_registry;
use runfiles::Runfiles;
use serde_json;
use std::{default, error::Error, fs, net::ToSocketAddrs, path::{Path, PathBuf}};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use walkdir::WalkDir;

pub struct BuildEventService {}

fn proto_name(s: &str) -> &str {
    if let Some(idx)=s.find("/") {
        return &s[idx+1..];
    }
    s
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
        log::error!("Got message: {:#?}", request);
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

        tokio::spawn(async move {
            while let maybe_message = in_stream.message().await {
                match maybe_message {
                    Ok(res) => {
                        if let Some(v) = res {
                            if let Some(obe) = &v.ordered_build_event.as_ref() {
                                let mut buildEnded = false;
                                let event = &v
                                    .ordered_build_event
                                    .as_ref()
                                    .unwrap()
                                    .event
                                    .as_ref()
                                    .unwrap()
                                    .event
                                    .as_ref()
                                    .unwrap();
                                match event {
                                    build_event::Event::BazelEvent(any) => {
                                        let be: build_event_stream::BuildEvent =
                                            build_event_stream::BuildEvent::decode(&any.value[..])
                                                .unwrap();
                                        match be.payload.as_ref().unwrap() {
                                            build_event_stream::build_event::Payload::Progress(
                                                p,
                                            ) => {
                                                print!("{0}", p.stdout);
                                                print!("{0}", p.stderr);
                                            }
                                            build_event_stream::build_event::Payload::Aborted(
                                                p,
                                            ) => println!("Abort {:#?}", p),
                                            build_event_stream::build_event::Payload::Started(
                                                s,
                                            ) => println!("Started {:#?}", s),
                                            _ => {
                                                log::info!("got be {}", proto_name(&any.type_url));
                                                let md_or = DescriptorPool::global().get_message_by_name(proto_name(&any.type_url));
                                                if md_or.is_none() {
                                                    log::warn!("MD not found for: {}", proto_name(&any.type_url));
                                                    continue;
                                                }
                                                let md = md_or.unwrap();
                                                let dm_or = DynamicMessage::decode(md, &any.value[..]);
                                                if dm_or.is_err() {
                                                    log::error!("can't parse build event into dm: {:#?}", dm_or.err().unwrap());
                                                    continue;
                                                }
                                                let sr = serde_json::ser::to_string(&dm_or.unwrap()).unwrap();
                                                log::info!("JSON {}", &sr);
                                            }
                                        }
                                    }
                                    build_event::Event::ComponentStreamFinished(end) => {
                                        buildEnded = true;
                                    }
                                    _ => {
                                        log::info!("Got other event: {:#?}", event)
                                    }
                                }
                                log::error!(
                                    "Sending ack for {:#?} {}",
                                    obe.stream_id,
                                    obe.sequence_number
                                );
                                tx.send(Ok(PublishBuildToolEventStreamResponse {
                                    sequence_number: obe.sequence_number,
                                    stream_id: obe.stream_id.clone(),
                                }))
                                .await
                                .expect("working tx");
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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("Starting!");
    let reflect = tonic_reflection::server::Builder::configure()
        .register_file_descriptor_set(*proto_registry::DESCRIPTORS.clone())
        .build()?;

    proto_registry::init_global_descriptor_pool()?;

    log::info!("Loaded messages: {:#?}", DescriptorPool::global().all_messages().map(|x| x.full_name().to_string()).collect::<Vec<String>>());

    let server = BuildEventService {};
    Server::builder()
        .add_service(publish_build_event_server::PublishBuildEventServer::new(
            server,
        ))
        .add_service(reflect)
        .serve("[::]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
