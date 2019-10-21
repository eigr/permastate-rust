extern crate config;
extern crate log4rs;

use log::info;
use actix::prelude::*;
use crate::protocol::{Options, ProtocolHandlerActor, RegisterMessage};

#[derive(Debug, Clone)]
pub struct EntityService {
    entity_type: String,
    protos: Vec<String>,
    persistence_id: String,
    snapshot_every: u16,
}

impl Default for EntityService {

    fn default() -> Self {
        EntityService {
            entity_type: String::from(""),
            protos: vec![],
            persistence_id: String::from(""),
            snapshot_every: 0
        }
    }
}

impl EntityService {

    pub fn persistence_id(&mut self, persistence_id: String) -> &mut EntityService {
        self.persistence_id = persistence_id;
        self
    }

    pub fn protos(&mut self, protos: Vec<String>) -> &mut EntityService {
        self.protos = protos;
        self
    }

    pub fn snapshot(&mut self, every: u16) -> &mut EntityService {
        self.snapshot_every = every;
        self
    }

    pub fn event_sourced(&mut self) -> EntityService {
        self.entity_type = "cloudstate.eventsourced.EventSourced".to_string();
        self.clone()
    }

    pub fn crdt(&mut self) -> EntityService {
        self.entity_type = "cloudstate.crdt.Crdt".to_string();
        self.clone()
    }

}

#[derive(Debug)]
pub struct CloudState {
    entity: EntityService,
    desciptor: String,
    additional_desciptors: Option<Vec<String>>,
    service_name: String,
    service_version: String,
    actor_system_name: String,
    server_port: u16,
}

impl Default for CloudState {

    fn default() -> CloudState {
        CloudState {
            entity: EntityService::default(),
            desciptor: String::from(""),
            additional_desciptors: Option::None,
            service_name: String::from(""),
            service_version: String::from("0.0.1"),
            actor_system_name: String::from("cloudstate-rust-system"),
            server_port: 8088
        }
    }
}

impl CloudState {
    
    pub fn new() -> Self {
        Default::default()
    }

    pub fn service_name(&mut self, service_name: String) -> &mut CloudState {
        self.service_name = service_name;
        self
    }

    pub fn service_version(&mut self, service_version: String) -> &mut CloudState {
        self.service_version = service_version;
        self
    }

    pub fn actor_system_name(&mut self, system_name: String) -> &mut CloudState {
        self.actor_system_name = system_name;
        self
    }

    pub fn port(&mut self, server_port: u16) -> &mut CloudState {
        self.server_port = server_port;
        self
    }

    pub fn register_entity_service(&mut self, service_name: String, entity_service: EntityService) -> &mut CloudState {
        self.service_name = service_name;
        self.entity = entity_service;
        self
    }

    pub fn start(&mut self) -> &mut CloudState {
        debug!("Create ActorSystem {:?}", self.actor_system_name);
        let actor_system = System::new("cloudstate-rust-system".to_string());

        // start new actor
        let addr = ProtocolHandlerActor{}.start();

        let options = Options {
            entity_type: self.entity.entity_type.clone(),
            service_name: self.service_name.clone(),
            desciptor: self.desciptor.clone(),
            additional_desciptors: self.additional_desciptors.clone(),
            persistence_id: self.entity.persistence_id.clone(),
            service_version: self.service_version.clone(),
            server_port: self.server_port
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

        actor_system.run()
            .map_err(|err| error!("Error on start ActorSystem. Error: {:?}", err))
            .ok();

        self
    }

}