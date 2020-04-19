extern crate config;
extern crate log4rs;
extern crate rustc_version;

use log::{info};
use actix::prelude::*;
use crate::protocol::server::GrpcServer;
use crate::serveless::EntityService;

pub mod spec {
    tonic::include_proto!("cloudstate");
}

#[derive(Debug, Clone)]
pub struct Options {
    pub entity_service: EntityService,
    pub service_name: String,
    pub service_version: String,
    pub server_port: u16,
}

pub struct StartMessage {
    pub opts: Options,
}

impl Message for StartMessage {
    type Result = Result<bool, std::io::Error>;
}

pub struct ProtocolHandlerActor;

impl Actor for ProtocolHandlerActor {
    type Context = Context<Self>;
}

// Actor handler
impl Handler<StartMessage> for ProtocolHandlerActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, _msg: StartMessage, _ctx: &mut Context<Self>) -> Self::Result {
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
    use std::fs::File;
    use std::io::Read;

    use tonic::{transport::Server, Request, Response, Status};
    //use prost_types::{FileDescriptorProto, FileDescriptorSet};

    use crate::protocol::spec::{
        server::{EntityDiscovery, EntityDiscoveryServer},
        ProxyInfo, EntitySpec, ServiceInfo, Entity,UserFunctionError,
    };

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
                entity_type: self.opts.entity_service.entity_type.to_string(),
                service_name: self.opts.service_name.to_string(),
                persistence_id: self.opts.entity_service.persistence_id.to_string(),
            };

            let vec_entities = vec![entity];

            let lib_name: String = String::from("cloudstate-rust-support");
            let lib_version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
            let runtime = "rustc ".to_owned() + &version().unwrap().to_string();

            let info = ServiceInfo {
                service_name: self.opts.service_name.to_string(),
                service_version: self.opts.service_version.to_string(),
                service_runtime: runtime,
                support_library_name: lib_name,
                support_library_version: lib_version.unwrap_or("0.5.0").to_string(),
            };

            // protoc --include_imports \
            // --proto_path=<proto file directory> \
            // --descriptor_set_out=user-function.desc \
            // <path to .proto files>
            //let tmp = tempfile::Builder::new().prefix("prost-build").tempdir()?;
            //let descriptor_set = tmp.path().join("prost-descriptor-set");


            let mut file = File::open("user-function.desc").unwrap();

            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();

            let reply = EntitySpec {
                proto: data,
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

            let clone_opts = self.options;

            rt.block_on(async {
                debug!("Now running on a worker thread");

                let opts = clone_opts.clone();
                let discover = Discover{ opts: clone_opts };

                let addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    opts.server_port
                );

                info!("Start CloudState gRPC in 0.0.0.0:{}", opts.server_port);
                Server::builder()
                    .add_service(EntityDiscoveryServer::new(discover))
                    .serve(addr)
                    .await
                    .map_err(|err| error!("Error during start server phase: {:?}", err))
                    .ok();
            });

            Ok(())
        }
    }

}



