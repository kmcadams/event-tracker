use actix_web::http::StatusCode;
use actix_web::{test, web, App};
use chrono::{DateTime, TimeZone, Utc};
use event_tracker::api::{get_event_by_id, get_events};
use event_tracker::model::Event;
use event_tracker::storage::{EventStore, InMemoryEventStore};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub fn insert_test_events(store: Arc<dyn EventStore>, types_and_times: &[(&str, &str)]) {
    for (event_type, timestamp_str) in types_and_times {
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .unwrap()
            .to_utc();

        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            timestamp,
            payload: json!({ "test": true }),
        };

        store.add_event(event).expect("Failed to insert test event");
    }
}

#[actix_rt::test]
async fn test_get_events_returns_inserted_event() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let event = Event {
        id: Uuid::new_v4(),
        event_type: "login".into(),
        timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
        payload: serde_json::json!({ "user_id": 1 }),
    };
    store.add_event(event.clone()).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?event_type=login")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 1);
    assert_eq!(returned[0].id, event.id);
}

#[actix_rt::test]
async fn test_get_events_returns_only_filtered_events() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    insert_test_events(
        store.clone(),
        &[
            ("login", "2025-01-01T12:00:00Z"),
            ("logout", "2025-01-01T13:00:00Z"),
            ("login", "2025-01-02T12:00:00Z"),
        ],
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?event_type=login")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 2);
}

#[actix_rt::test]
async fn test_get_events_returns_200_if_none_found() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?event_type=login")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 0);
}

#[actix_rt::test]
async fn test_get_events_by_time_range() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

    insert_test_events(
        store.clone(),
        &[
            ("login", "2025-01-01T12:00:00Z"),
            ("login", "2025-01-02T12:00:00Z"),
            ("logout", "2025-01-02T12:00:00Z"),
            ("login", "2025-01-03T12:00:00Z"),
        ],
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?start=2025-01-02T00:00:00Z&end=2025-01-02T23:59:59Z")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 2);
}

#[actix_rt::test]
async fn test_get_events_by_time_range_and_type() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

    insert_test_events(
        store.clone(),
        &[
            ("login", "2025-01-01T12:00:00Z"),
            ("login", "2025-01-02T12:00:00Z"),
            ("logout", "2025-01-02T12:00:00Z"),
            ("login", "2025-01-03T12:00:00Z"),
        ],
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?start=2025-01-02T00:00:00Z&end=2025-01-02T23:59:59Z&event_type=login")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 1);
}

#[actix_rt::test]
async fn test_get_events_invalid_datetimes() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

    insert_test_events(
        store.clone(),
        &[
            ("login", "2025-01-01T12:00:00Z"),
            ("login", "2025-01-02T12:00:00Z"),
            ("logout", "2025-01-02T12:00:00Z"),
            ("login", "2025-01-03T12:00:00Z"),
        ],
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?start=about1pmontuesday")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let req = test::TestRequest::get()
        .uri("/events?end=about1thefollowingwednesday")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_get_events_invalid_query_parameter() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

    insert_test_events(
        store.clone(),
        &[
            ("login", "2025-01-01T12:00:00Z"),
            ("login", "2025-01-02T12:00:00Z"),
            ("logout", "2025-01-02T12:00:00Z"),
            ("login", "2025-01-03T12:00:00Z"),
        ],
    );

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/events?strat=2025-01-02T00:00:00Z")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned: Vec<Event> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned.len(), 4);
}

#[actix_rt::test]
async fn test_get_event_by_id_success() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let event = Event {
        id: Uuid::new_v4(),
        event_type: "test".into(),
        timestamp: Utc::now(),
        payload: serde_json::json!({ "val": 123 }),
    };

    store.add_event(event.clone()).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(get_event_by_id),
    )
    .await;

    let uri = format!("/events/{}", event.id);
    let req = test::TestRequest::get().uri(&uri).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = test::read_body(resp).await;
    let returned_event: Event = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(returned_event.id, event.id);
    assert_eq!(returned_event.event_type, event.event_type);
}
