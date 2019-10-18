extern crate config;
extern crate log4rs;

use log::{info, debug, trace, warn};
use tonic::{transport::Server, Request, Response, Status};

use actix::prelude::*;
use crate::protocol::{Options, ProtocolHandlerActor, RegisterMessage};

#[derive(Debug, Default, Clone)]
pub struct Cloudstate {
    
}

impl Cloudstate {
    
    pub fn builder() -> Self {
        Default::default()
    }

    pub fn start(self) -> Self {
        let system = System::new("cloudstate-rust");

        // start new actor
        let addr = ProtocolHandlerActor{}.start();

        let options = Options{};

        let msg = RegisterMessage{
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

        system.run();
        self
    }
}