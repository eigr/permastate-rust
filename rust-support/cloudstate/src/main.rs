extern crate log;
extern crate log4rs;
extern crate cloudstate;

use log::{info};
use cloudstate::serveless::{CloudState, EntityService};

fn main() {

    // Cloudstate depends of log4rs to print messages
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    info!("Starting Cloudstate server...");

    let entity_service = EntityService::default()
        .persistence_id("shopping-cart".to_string())
        .protos(vec!["shoppingcart/shoppingcart.proto".to_string(), "shoppingcart/persistence/domain.proto".to_string()])
        .event_sourced();

    CloudState::new()
        .register_event_sourced(
            "com.example.shoppingcart.ShoppingCart".to_string(),
            entity_service)
        .start();
}
