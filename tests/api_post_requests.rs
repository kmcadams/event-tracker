use event_tracker::{
    api::post_event,
    storage::{EventStore, InMemoryEventStore},
};

use actix_web::{http::StatusCode, test, web, App};
use std::sync::Arc;

#[actix_rt::test]
async fn test_post_event_valid() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());

    let app = test::init_service(App::new().app_data(store_data.clone()).service(post_event)).await;

    let req = test::TestRequest::post()
        .uri("/events")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
            "event_type": "login",
            "timestamp": "2025-01-01T12:00:00Z",
            "payload": { "user_id": 1 }
        }"#,
        )
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_post_event_invalid_timestamp() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());
    let app = test::init_service(App::new().app_data(store_data.clone()).service(post_event)).await;

    let req = test::TestRequest::post()
        .uri("/events")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "event_type": "login",
                "timestamp": "not-a-timestamp",
                "payload": { "user_id": 1 }
            }"#,
        )
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_post_event_missing_field() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());
    let app = test::init_service(App::new().app_data(store_data.clone()).service(post_event)).await;

    let req = test::TestRequest::post()
        .uri("/events")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "timestamp": "2025-01-01T12:00:00Z",
                "payload": { "user_id": 1 }
            }"#,
        )
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_post_event_empty_body() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());
    let app = test::init_service(App::new().app_data(store_data.clone()).service(post_event)).await;

    let req = test::TestRequest::post()
        .uri("/events")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_post_event_malformed_json() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let store_data: web::Data<Arc<dyn EventStore>> = web::Data::new(store.clone());
    let app = test::init_service(App::new().app_data(store_data.clone()).service(post_event)).await;

    let req = test::TestRequest::post()
        .uri("/events")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{ event_type: 'login' }")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
