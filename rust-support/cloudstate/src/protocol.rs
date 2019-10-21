extern crate config;
extern crate log4rs;
extern crate rustc_version;

use log::{info};
use actix::prelude::*;
use crate::protocol::server::GrpcServer;

pub mod spec {
    tonic::include_proto!("cloudstate");
}

#[derive(Debug, Clone)]
pub struct Options {
    pub entity_type: String,
    pub service_name: String,
    pub desciptor: String,
    pub additional_desciptors: Option<Vec<String>>,
    pub persistence_id: String,
    pub service_version: String,
    pub server_port: u16,
}

pub struct RegisterMessage {
    pub opts: Options,
}

impl Message for RegisterMessage {
    type Result = Result<bool, std::io::Error>;
}

pub struct ProtocolHandlerActor;

impl Actor for ProtocolHandlerActor {
    type Context = Context<Self>;
}

// Actor handler
impl Handler<RegisterMessage> for ProtocolHandlerActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, _msg: RegisterMessage, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Starting server and register messages");
        Ok(
            GrpcServer::new(_msg.opts)
            .start().is_ok()
        )
    }
}

// gRPC
pub mod server {

    use tokio::runtime::Runtime;
    use crate::protocol::Options;
    use super::rustc_version::version;
    use log::{info, debug};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tonic::{transport::Server, Request, Response, Status};
    use prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::protocol::spec::{
        server::{EntityDiscovery, EntityDiscoveryServer},
        ProxyInfo, EntitySpec, ServiceInfo, Entity,UserFunctionError,
    };

    use crate::protocol::router::Router;

    #[derive(Debug, Clone)]
    pub struct Discover {
        pub opts: Options,
    }

    #[tonic::async_trait]
    impl EntityDiscovery for Discover {

        async fn discover(
            &self,
            request: Request<ProxyInfo>,
        ) -> Result<Response<EntitySpec>, Status> {

            let metadata = request.metadata();
            let proxy_info: &ProxyInfo = request.get_ref();

            debug!("Received discovery call from sidecar with Metadata [{:?}", metadata );
            info!("Received discovery call from sidecar [{:?} {:?}] supporting CloudState {:?}.{:?}", proxy_info.proxy_name, proxy_info.proxy_version, proxy_info.protocol_major_version, proxy_info.protocol_minor_version);
            info!("Supported sidecar entity types: {:?}", proxy_info.supported_entity_types);

            let entity = Entity {
                entity_type: self.opts.entity_type.to_string(), 
                service_name: self.opts.service_name.to_string(),
                persistence_id: self.opts.persistence_id.to_string(),
            };

            let vec_entities = vec![entity];

            let lib_name: String = "cloudstate-rust-support".to_string();
            let lib_version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
            let runtime = "rustc ".to_owned() + &version().unwrap().to_string();

            let info = ServiceInfo {
                service_name: self.opts.service_name.to_string(),
                service_version: self.opts.service_version.to_string(),
                service_runtime: runtime,
                support_library_name: lib_name,
                support_library_version: lib_version.unwrap_or("0.0.1").to_string(),
            };

            /*FileDescriptorSet{
                file: vec![]
            }*/

            let reply = EntitySpec {
                proto: vec![],
                entities: vec_entities,
                service_info: Some(info),
            };

            Ok(Response::new(reply))
        }

        // Handle Sidecar Errors Messages
        async fn report_error(
            &self,
            request: Request<UserFunctionError>,
        ) -> Result<Response<()>, Status> {

            let metadata = request.metadata();
            let user_func_error: &UserFunctionError = request.get_ref();

            debug!("Receive request for report_error. Metadata: {:?}", metadata );
            error!("Received report_error from sidecar. Error: {:?}", user_func_error.message);
            Ok(Response::new(()))
        }

    }

    pub struct GrpcServer {
        pub options: Options,
    }

    impl GrpcServer {

        pub fn new(opts: Options) -> Self {
            GrpcServer{
                options: opts
            }
        }

        pub fn start(self) -> Result<(), Box<dyn std::error::Error>> {
            // Create the runtime
            let rt = Runtime::new().unwrap();

            let cloneopts = self.options;

            rt.block_on(async {
                debug!("Now running on a worker thread");

                let opts = cloneopts.clone();
                let discover = Discover{ opts: cloneopts };

                let addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    opts.server_port
                );

                info!("Start CloudState gRPC in 0.0.0.0:{}", opts.server_port);
                Server::builder()
                    .serve(
                        addr,
                        Router {
                            entity_discovery: EntityDiscoveryServer::new(discover)
                        })
                    .await
                    .map_err(|err| error!("Error during start server phase: {:?}", err))
                    .ok();
            });

            Ok(())
        }
    }

}

pub mod router {
    use futures_util::future;
    use http::{Request, Response};

    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    use tonic::{body::BoxBody, transport::Body};

    use crate::protocol::spec::{
        server::{EntityDiscoveryServer},
    };

    use tower::Service;

    #[derive(Clone)]
    pub struct Router {
        pub entity_discovery: EntityDiscoveryServer<crate::protocol::server::Discover>,
        //pub entity_discovery: EntityDiscovery,
    }

    impl Service<()> for Router {
        type Response = Router;
        type Error = Never;
        type Future = future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Ok(()).into()
        }

        fn call(&mut self, _req: ()) -> Self::Future {
            future::ok(self.clone())
        }
    }

    impl Service<Request<Body>> for Router {
        type Response = Response<BoxBody>;
        type Error = Never;
        type Future =
        Pin<Box<dyn Future<Output = Result<Response<BoxBody>, Never>> + Send + 'static>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Ok(()).into()
        }

        fn call(&mut self, req: Request<Body>) -> Self::Future {
            let mut segments = req.uri().path().split("/");
            segments.next();
            let service = segments.next().unwrap();

            match service {
                "cloudstate.EntityDiscovery" => {
                    let me = self.clone();
                    Box::pin(async move {
                        let mut svc = me.entity_discovery;
                        let mut svc = svc.call(()).await.unwrap();

                        let res = svc.call(req).await.unwrap();
                        Ok(res)
                    })
                }

                /*"cloudstate.eventsourced.EventSourced" => {
                    let me = self.clone();
                    Box::pin(async move {
                        let mut svc =
                            UnimplementedServiceServer::from_shared(me.unimplemented_service);
                        let mut svc = svc.call(()).await.unwrap();

                        let res = svc.call(req).await.unwrap();
                        Ok(res)
                    })
                }*/

                /*"cloudstate.crdt.Crdt" => {
                    let me = self.clone();
                    Box::pin(async move {
                        let mut svc =
                            UnimplementedServiceServer::from_shared(me.unimplemented_service);
                        let mut svc = svc.call(()).await.unwrap();

                        let res = svc.call(req).await.unwrap();
                        Ok(res)
                    })
                }*/

                /*"cloudstate.function.StatelessFunction" => {
                    let me = self.clone();
                    Box::pin(async move {
                        let mut svc =
                            UnimplementedServiceServer::from_shared(me.unimplemented_service);
                        let mut svc = svc.call(()).await.unwrap();

                        let res = svc.call(req).await.unwrap();
                        Ok(res)
                    })
                }*/

                _ => unimplemented!(),
            }
        }
    }

    #[derive(Debug)]
    pub enum Never {}

    impl std::fmt::Display for Never {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match *self {}
        }
    }

    impl std::error::Error for Never {}
}









