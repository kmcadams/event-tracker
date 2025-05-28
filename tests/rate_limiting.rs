use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor};
use actix_web::dev::ServiceRequest;
use actix_web::{error::HttpError, http::StatusCode, test, web, App};
use chrono::{TimeZone, Utc};
use event_tracker::api::get_events;
use event_tracker::model::Event;
use event_tracker::storage::{EventStore, InMemoryEventStore};
use std::sync::Arc;
use uuid::Uuid;

//When running tests, need use a test key extractor since no IP address exists
#[derive(Clone)]
struct TestKeyExtractor;

impl KeyExtractor for TestKeyExtractor {
    type Key = String;
    type KeyExtractionError = HttpError;

    fn extract(&self, _req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        Ok("test-client".to_string())
    }
}

#[actix_rt::test]
async fn test_rate_limit_triggers() {
    let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
    let event = Event {
        id: Uuid::new_v4(),
        event_type: "login".into(),
        timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
        payload: serde_json::json!({ "user_id": 1 }),
    };
    store.add_event(event.clone()).unwrap();

    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(2)
        .burst_size(2)
        .key_extractor(TestKeyExtractor)
        .finish()
        .unwrap();

    let app = test::init_service(
        App::new()
            .wrap(Governor::new(&governor_conf))
            .app_data(web::Data::new(store))
            .service(get_events),
    )
    .await;

    let req1 = test::TestRequest::get().uri("/events").to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::OK);

    let req2 = test::TestRequest::get().uri("/events").to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::OK);

    let req3 = test::TestRequest::get().uri("/events").to_request();
    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(resp3.status(), StatusCode::TOO_MANY_REQUESTS);
}
