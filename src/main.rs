use log::{error, info};
use std::sync::Arc;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};

use event_tracker::api::{get_event_by_id, get_events, post_event};
use event_tracker::storage::{EventStore, InMemoryEventStore};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yml", log4rs::config::Deserializers::default()).unwrap_or_else(|e| {
        error!("Failed to initialize log4rs: {}", e);
        std::process::exit(1)
    });
    info!("Starting server...");

    let host = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());

    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(5)
        .burst_size(10)
        .finish()
        .unwrap_or_else(|| {
            error!("Failed to create governor config");
            std::process::exit(2)
        });

    info!("Listening on http://{}", host);
    HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&governor_conf))
            .app_data(web::Data::from(store_data.clone()))
            .service(post_event)
            .service(get_events)
            .service(get_event_by_id)
    })
    .bind(host)?
    .run()
    .await
}
