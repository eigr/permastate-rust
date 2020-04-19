extern crate config;
extern crate log4rs;

use log::info;
use actix::prelude::*;
use crate::protocol::{Options, ProtocolHandlerActor, StartMessage};

#[derive(Debug, Clone)]
pub struct EntityService {
    pub entity_type: String,
    pub persistence_id: String,
    pub snapshot_every: u16,
}

impl Default for EntityService {

    fn default() -> EntityService {
        EntityService {
            entity_type: String::from(""),
            persistence_id: String::from(""),
            snapshot_every: 0
        }
    }
}

impl EntityService {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn persistence_id(&mut self, persistence_id: String) -> &mut EntityService {
        self.persistence_id = persistence_id;
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
    service_name: String,
    service_version: String,
    actor_system_name: String,
    server_port: u16,
}

impl Default for CloudState {

    fn default() -> CloudState {
        CloudState {
            entity: EntityService::default(),
            service_name: String::from(""),
            service_version: String::from("0.5.0"),
            actor_system_name: String::from("cloudstate-rust-system"),
            server_port: 8080
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
        let system = self.actor_system_name.clone();
        debug!("Create ActorSystem {:?}", system);
        let actor_system = System::new(system);

        // start new actor
        let addr = ProtocolHandlerActor{}.start();

        let options = Options {
            entity_service: self.entity.clone(),
            service_name: self.service_name.clone(),
            service_version: self.service_version.clone(),
            server_port: self.server_port
        };

        let msg = StartMessage {
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