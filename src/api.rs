use actix_web::{get, post, web, Responder};
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
    let new_event = Event {
        id: Uuid::new_v4(),
        event_type: payload.event_type.clone(),
        timestamp: payload.timestamp,
        payload: payload.payload.clone(),
    };

    store.add_event(new_event.clone())?;

    Ok(web::Json(new_event))
}

#[get("/events")]
async fn get_events(
    store: web::Data<Arc<dyn EventStore>>,
    query: web::Query<EventQuery>,
) -> Result<impl Responder, AppError> {
    let results = store.query_events(query.into_inner())?;
    Ok(web::Json(results))
}
