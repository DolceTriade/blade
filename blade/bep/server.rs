use build_proto::google::devtools::build::v1::*;
use log;
use pretty_env_logger;
use runfiles::Runfiles;
use std::{error::Error, net::ToSocketAddrs, fs};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status, Streaming};
pub struct BuildEventService {}

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
                            log::info!("Got stream {:#?}", v);
                            if let Some(obe) = &v.ordered_build_event.as_ref() {
                                let buildEnded = if let build_event::Event::ComponentStreamFinished(_) = &v.ordered_build_event.as_ref().unwrap().event.as_ref().unwrap().event.as_ref().unwrap() {
                                    true
                                } else {
                                    false
                                };
                                log::error!("Sending ack for {:#?} {}", obe.stream_id, obe.sequence_number);
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
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting!");
    let r = Runfiles::create()?;
    let path = r.rlocation("googleapis/google/devtools/build/v1/build_proto-descriptor-set.proto.bin");
    let desc = fs::read(path)?;
    let server = BuildEventService {};
    let reflect = tonic_reflection::server::Builder::configure().register_encoded_file_descriptor_set(&desc).build()?;

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
