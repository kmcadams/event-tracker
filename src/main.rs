use std::sync::Arc;

use actix_web::{web, App, HttpServer};

use event_tracker::api::{get_event_by_id, get_events, post_event};
use event_tracker::storage::{EventStore, InMemoryEventStore};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());

    println!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(store_data.clone()))
            .service(post_event)
            .service(get_events)
            .service(get_event_by_id)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
