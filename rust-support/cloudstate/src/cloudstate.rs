use log::{info, debug, trace, warn};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct Cloudstate {
    
}

impl Cloudstate {
    pub fn builder() -> Self {
        Default::default()
    }
}