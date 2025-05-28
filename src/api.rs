use actix_web::{get, post, web, Responder};
use log::{debug, info, warn};

use std::sync::Arc;

use crate::error::AppError;
use crate::model::{Event, EventQuery, NewEvent};
use crate::storage::EventStore;
use uuid::Uuid;

#[post("/events")]
async fn post_event(
    store: web::Data<Arc<dyn EventStore>>,
    payload: web::Json<NewEvent>,
) -> Result<impl Responder, AppError> {
    debug!("Received event: {:#?}", payload);
    let new_event = Event {
        id: Uuid::new_v4(),
        event_type: payload.event_type.clone(),
        timestamp: payload.timestamp,
        payload: payload.payload.clone(),
    };

    store.add_event(new_event.clone())?;

    info!("Stored event: {:#?}", new_event);

    Ok(web::Json(new_event))
}

#[get("/events")]
async fn get_events(
    store: web::Data<Arc<dyn EventStore>>,
    query: web::Query<EventQuery>,
) -> Result<impl Responder, AppError> {
    debug!("Received query: {:#?}", query);
    let results = store.query_events(query.into_inner())?;
    info!("Query results: {:#?}", results);
    Ok(web::Json(results))
}

#[get("/events/{id}")]
async fn get_event_by_id(
    store: web::Data<Arc<dyn EventStore>>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    debug!("Received id: {:#?}", path);
    let id = path.into_inner();
    match store.get_by_id(id)? {
        Some(event) => {
            info!("Found event: {:#?}", event);
            Ok(web::Json(event))
        }
        None => {
            warn!("Event {} not found", id);
            Err(AppError::NotFound(format!("Event {} not found", id)))
        }
    }
}
