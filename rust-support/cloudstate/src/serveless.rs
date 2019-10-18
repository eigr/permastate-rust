extern crate config;
extern crate log4rs;

use log::info;
use actix::prelude::*;
use crate::protocol::{Options, ProtocolHandlerActor, RegisterMessage};

#[derive(Debug)]
pub struct CloudState {
    entity_type: String,
    additional_descriptors: Option<Vec<String>>,
    service_name: String,
    persistence_id: String,
    actor_system_name: String,
    server_port: u16,
}

impl Default for CloudState {

    fn default() -> CloudState {
        CloudState {
            entity_type: "".to_string(),
            additional_descriptors: Option::None,
            service_name: "".to_string(),
            persistence_id: "".to_string(),
            actor_system_name: "cloudstate-rust-system".to_string(),
            server_port: 8088
        }
    }
}

impl CloudState {
    
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_event_sourced(&mut self, entity: String, additional_descriptors: Option<Vec<String>>) -> &mut CloudState {
        self.entity_type = entity;
        self.additional_descriptors = additional_descriptors;
        self
    }

    pub fn register_event_crdt(&mut self, entity: String, additional_descriptors:  Option<Vec<String>>) -> &mut CloudState {
        self.entity_type = entity;
        self.additional_descriptors = additional_descriptors;
        self
    }

    pub fn start(&mut self) -> &mut CloudState {
        debug!("Create ActorSystem {:?}", self.actor_system_name);
        let actor_system = System::new("cloudstate-rust-system".to_string());

        // start new actor
        let addr = ProtocolHandlerActor{}.start();

        let options = Options {
            entity_type: self.entity_type.clone(),
            service_name: "ShoppingCart".to_string(),
            persistence_id: self.persistence_id.clone(),
            grpc_port: self.server_port
        };

        let msg = RegisterMessage {
            opts: options,
        };
        
        // send message and get future for result
        let res = addr.send(msg);

        Arbiter::spawn(
            res.map(|_res| {
                info!("System started!");
            })
            .map_err(|_| ())
        );

        actor_system.run();
        self
    }

}