extern crate config;
extern crate log4rs;
extern crate rustc_version;

use log::{info};
use actix::prelude::*;
use crate::protocol::server::GrpcServer;

#[derive(Debug, Clone)]
pub struct Options {
    pub entity_type: std::string::String,
    pub service_name: std::string::String,
    pub persistence_id: std::string::String,
    pub grpc_port: u16,
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
        GrpcServer::builder(_msg.opts)
            .start_server();
        Ok(true)
    }
}

// gRPC
pub mod server {

    use tokio::runtime::Runtime;
    use crate::protocol::Options;
    use super::rustc_version::version;
    use log::{info, debug, trace, warn};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tonic::{transport::Server, Request, Response, Status};
    use prost_types::{FileDescriptorProto, FileDescriptorSet};

    pub mod spec {
        tonic::include_proto!("cloudstate");
    }

    use spec::{
        server::{EntityDiscovery, EntityDiscoveryServer},
        ProxyInfo, EntitySpec, ServiceInfo, Entity,UserFunctionError,
    };

    pub struct Discover {
        pub opts: Options,
    }

    #[tonic::async_trait]
    impl EntityDiscovery for Discover {

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

        async fn discover(
            &self,
            request: Request<ProxyInfo>,
        ) -> Result<Response<EntitySpec>, Status> {

            let metadata = request.metadata();
            let proxy_info: &ProxyInfo = request.get_ref();

            debug!("Received discovery call from sidecar with Metadata [{:?}", metadata );
            info!("Received discovery call from sidecar [{:?} {:?}] supporting CloudState {:?}.{:?}", proxy_info.proxy_name, proxy_info.proxy_version, proxy_info.protocol_major_version, proxy_info.protocol_minor_version);

            let entity = Entity{
                entity_type: self.opts.entity_type.to_string(), 
                service_name: self.opts.service_name.to_string(),
                persistence_id: "".to_string(),
            };

            let vec_entities = vec![entity];

            let lib_version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
            let runtime = "rustc ".to_owned() + &version().unwrap().to_string();

            let info = ServiceInfo{
                service_name: "ShoppingCart".to_string(),
                service_version: "0.1".to_string(),
                service_runtime: runtime,
                support_library_name: "cloudstate-rust-support".to_string(),
                support_library_version: lib_version.unwrap_or("0.0.1").to_string(),
            };

            let reply = spec::EntitySpec {
                proto: vec![],
                entities: vec_entities,
                service_info: Some(info),
            };
            Ok(Response::new(reply))
        }

    }

    pub struct GrpcServer {
        pub options: Options,
    }

    impl GrpcServer {

        pub fn builder(opts: Options) -> Self {
            GrpcServer{
                options: opts
            }
        }

        pub fn start_server(self) -> Result<(), Box<dyn std::error::Error>> {
            // Create the runtime
            let rt = Runtime::new().unwrap();

            let cloneopts = self.options;

            rt.block_on(async {
                debug!("Now running on a worker thread");

                let opts = cloneopts.clone();
                let discover = Discover{ opts: cloneopts };

                let addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    opts.grpc_port
                );

                info!("Start Cloudstate gRPC in {}", "0.0.0.0:8088".to_string());
                Server::builder()
                    .serve(addr, EntityDiscoveryServer::new(discover))
                    .await;
            });

            Ok(())
        }
    }
}









