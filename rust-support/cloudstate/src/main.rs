extern crate log;
extern crate log4rs;
extern crate cloudstate;

use log::{info};
use cloudstate::serveless::CloudState;

fn main() {

    // Cloudstate depends of log4rs to print messages
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    info!("Starting Cloudstate server...");
    let entities: Vec<String> = vec!["com.example.shoppingcart.persistence.Domain".to_string(); 1];

    CloudState::new()
        .register_event_sourced(
            "com.example.shoppingcart.ShoppingCart".to_string(),
            Option::from(entities))
        .start();
}
