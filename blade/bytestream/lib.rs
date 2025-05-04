use std::collections::HashMap;

use bytestream_proto::google::bytestream::*;
use futures::StreamExt;

fn stringify<E: std::fmt::Debug>(e: E) -> String { format!("{e:#?}") }
pub struct Client {
    overrides: HashMap<String, String>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            overrides: Default::default(),
        }
    }

    pub fn add_override(&mut self, host: &str, o: &str) {
        self.overrides.insert(host.to_string(), o.to_string());
    }

    pub async fn download_file(&self, f: &str) -> Result<Vec<u8>, String> {
        let mut uri = url::Url::parse(f).map_err(stringify)?;
        let path = uri.path().to_string();
        uri.set_path("");
        if let Some(o) = self.overrides.get(&Into::<String>::into(uri.clone())) {
            uri = url::Url::parse(o).map_err(stringify)?;
        }
        let uri_str: String = uri.clone().into();
        let mut client = byte_stream_client::ByteStreamClient::connect(uri_str)
            .await
            .map_err(stringify)?;
        let req = ReadRequest {
            read_limit: 0,
            read_offset: 0,
            resource_name: path,
        };
        let mut stream = client.read(req).await.map_err(stringify)?.into_inner();
        let mut v = vec![];
        loop {
            match stream.next().await.as_mut() {
                Some(Ok(r)) => {
                    v.append(&mut r.data);
                },
                Some(Err(e)) => return Err(stringify(e)),
                _ => break,
            }
        }
        Ok(v)
    }
}

impl Default for Client {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod test {
    use std::{future::Future, net::SocketAddr};

    use bytestream_proto::google::bytestream::*;
    use tokio::{net::TcpListener, sync::mpsc};
    use tokio_stream::{
        StreamExt,
        wrappers::{ReceiverStream, TcpListenerStream},
    };
    use tonic::{Request, Response, Status, async_trait, transport::Server};

    struct ServerStub {}

    #[async_trait]
    impl byte_stream_server::ByteStream for ServerStub {
        type ReadStream = ReceiverStream<Result<ReadResponse, Status>>;

        async fn read(
            &self,
            request: Request<ReadRequest>,
        ) -> Result<Response<Self::ReadStream>, Status> {
            let req = request.into_inner();
            if &req.resource_name != "/path/to/real_resource" {
                return Err(tonic::Status::internal("badness"));
            }
            // creating infinite stream with requested message
            let repeat = std::iter::repeat(ReadResponse { data: vec![0x0] });
            let mut stream = Box::pin(tokio_stream::iter(repeat.take(100)));

            // spawn and channel are required if you want handle "disconnect" functionality
            // the `out_stream` will not be polled after client disconnect
            let (tx, rx) = mpsc::channel(128);
            tokio::spawn(async move {
                while let Some(item) = stream.next().await {
                    match tx.send(Result::<_, Status>::Ok(item)).await {
                        Ok(_) => {
                            // item (server response) was queued to be send to
                            // client
                        },
                        Err(_item) => {
                            // output_stream was build from rx and both are dropped
                            break;
                        },
                    }
                }
            });

            let output_stream = ReceiverStream::new(rx);
            Ok(Response::new(output_stream))
        }

        async fn write(
            &self,
            _request: tonic::Request<tonic::Streaming<super::WriteRequest>>,
        ) -> std::result::Result<tonic::Response<super::WriteResponse>, tonic::Status> {
            todo!()
        }

        async fn query_write_status(
            &self,
            _request: tonic::Request<super::QueryWriteStatusRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryWriteStatusResponse>, tonic::Status>
        {
            todo!()
        }
    }

    async fn server_and_addr() -> (impl Future<Output = ()>, SocketAddr) {
        let lis = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = lis.local_addr().unwrap();
        let stream = TcpListenerStream::new(lis);

        let serve_future = async {
            let result = Server::builder()
                .add_service(byte_stream_server::ByteStreamServer::new(ServerStub {}))
                .serve_with_incoming(stream)
                .await;
            // Server must be running fine...
            assert!(result.is_ok());
        };

        (serve_future, addr)
    }

    #[tokio::test]
    async fn download_file_test() {
        let (serve_future, addr) = server_and_addr().await;

        let request_future = async {
            let c = crate::Client::new();
            let uri = format!("bytestream://{addr}/path/to/real_resource");
            let bytes = c.download_file(&uri).await.unwrap();
            // Validate server response with assertions
            assert_eq!(bytes.len(), 100);
        };

        // Wait for completion, when the client request future completes
        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }

    #[tokio::test]
    async fn download_file_with_override_test() {
        let (serve_future, addr) = server_and_addr().await;

        let request_future = async {
            let mut c = crate::Client::new();
            let uri = "bytestream://whatever.com/path/to/real_resource";
            c.add_override(
                "bytestream://whatever.com",
                &format!("bytestream://{}", &addr.to_string()),
            );
            let bytes = c.download_file(uri).await.unwrap();
            // Validate server response with assertions
            assert_eq!(bytes.len(), 100);
        };

        // Wait for completion, when the client request future completes
        tokio::select! {
            _ = serve_future => panic!("server returned first"),
            _ = request_future => (),
        }
    }
}
