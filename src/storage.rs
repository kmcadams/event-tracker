use log::{debug, info};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::{Event, EventQuery};

//Trait implementation that all other storage implementations use
//Web api accepts any Struct/Object that implements this trait
//can expand as needed
pub trait EventStore: Send + Sync {
    fn add_event(&self, event: Event) -> Result<(), AppError>;
    fn query_events(&self, query: EventQuery) -> Result<Vec<Event>, AppError>;
    fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, AppError>;
}

//Initial Struct and implementation for in-memory storage of events.  Also can continue to be used for testing
//Guarded with a RwLock--Reads could be many, writes should be few
#[derive(Default)]
pub struct InMemoryEventStore {
    events: RwLock<HashMap<Uuid, Event>>,
    count: AtomicUsize,
}

impl InMemoryEventStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            count: AtomicUsize::new(0),
        }
    }

    pub fn metrics(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

impl EventStore for InMemoryEventStore {
    fn add_event(&self, event: Event) -> Result<(), AppError> {
        let mut events = self
            .events
            .write()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        debug!("Inserting event with ID: {}", event.id);

        events.insert(event.id, event);
        self.count.fetch_add(1, Ordering::Relaxed);
        let current_count = events.len();
        let estimated_event_size = std::mem::size_of::<Event>();
        let estimated_total_bytes = current_count * estimated_event_size;

        info!(
            "Current event count: {}, Estimated memory usage: {} bytes",
            current_count, estimated_total_bytes
        );
        Ok(())
    }

    fn query_events(&self, query: EventQuery) -> Result<Vec<Event>, AppError> {
        let events = self
            .events
            .read()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let result: Vec<Event> = events
            .values()
            .filter(|event| {
                query
                    .event_type
                    .as_ref()
                    .map_or(true, |t| &event.event_type == t)
                    && query.start.map_or(true, |start| event.timestamp >= start)
                    && query.end.map_or(true, |end| event.timestamp <= end)
            })
            .cloned()
            .collect();

        debug!(
            "Query: type={:?}, start={:?}, end={:?} -> {} result(s)",
            query.event_type,
            query.start,
            query.end,
            result.len()
        );
        Ok(result)
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, AppError> {
        debug!("Retrieving event with ID: {}", id);
        let events = self
            .events
            .read()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok(events.get(&id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use tokio::task;

    fn sample_event(id: Option<Uuid>, event_type: &str, ts: &str) -> Event {
        Event {
            id: id.unwrap_or_else(Uuid::new_v4),
            event_type: event_type.to_string(),
            timestamp: DateTime::parse_from_rfc3339(ts).unwrap().to_utc(),
            payload: json!({ "example": true }),
        }
    }

    #[test]
    fn test_add_and_get_event() {
        let store = InMemoryEventStore::new();
        let event = sample_event(None, "test", "2025-01-01T12:00:00Z");

        let id = event.id;
        store.add_event(event.clone()).unwrap();

        let retrieved = store.get_by_id(id).unwrap();
        assert_eq!(retrieved, Some(event));
        assert_eq!(store.metrics(), 1);
    }

    #[test]
    fn test_query_by_type() {
        let store = InMemoryEventStore::new();
        let e1 = sample_event(None, "login", "2025-01-01T12:00:00Z");
        let e2 = sample_event(None, "logout", "2025-01-01T13:00:00Z");
        store.add_event(e1.clone()).unwrap();
        store.add_event(e2.clone()).unwrap();

        let results = store
            .query_events(EventQuery {
                event_type: Some("login".to_string()),
                start: None,
                end: None,
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, "login");
        assert_eq!(store.metrics(), 2);
    }

    #[test]
    fn test_query_by_time_range() {
        let store = InMemoryEventStore::new();
        let e1 = sample_event(None, "test", "2025-01-01T10:00:00Z");
        let e2 = sample_event(None, "test", "2025-01-01T11:00:00Z");
        let e3 = sample_event(None, "test", "2025-01-01T12:00:00Z");

        store.add_event(e1.clone()).unwrap();
        store.add_event(e2.clone()).unwrap();
        store.add_event(e3.clone()).unwrap();

        let start = DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
            .unwrap()
            .to_utc();
        let end = DateTime::parse_from_rfc3339("2025-01-01T11:30:00Z")
            .unwrap()
            .to_utc();

        let results = store
            .query_events(EventQuery {
                event_type: None,
                start: Some(start),
                end: Some(end),
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timestamp, e2.timestamp);
        assert_eq!(store.metrics(), 3);
    }

    #[test]
    fn test_get_by_id_not_found() {
        let store = InMemoryEventStore::new();
        let random_id = Uuid::new_v4();
        let result = store.get_by_id(random_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_query_events_no_match() {
        let store = InMemoryEventStore::new();
        let result = store
            .query_events(EventQuery {
                event_type: Some("nonexistent".into()),
                start: None,
                end: None,
            })
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_poisoned_lock_add_event() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let store = InMemoryEventStore {
            events: RwLock::new(HashMap::new()),
            count: AtomicUsize::new(0),
        };

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.events.write().unwrap();
            panic!("simulate panic while holding write lock");
        }));

        let event = sample_event(None, "test", "2025-01-01T12:00:00Z");
        let result = store.add_event(event);
        assert!(matches!(result, Err(AppError::InternalError(_))));
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

        store
            .add_event(Event {
                id: Uuid::new_v4(),
                event_type: "test".into(),
                timestamp: Utc::now(),
                payload: serde_json::json!({"user_id": 1}),
            })
            .unwrap();

        let store1 = Arc::clone(&store);
        let store2 = Arc::clone(&store);

        let t1 = task::spawn_blocking(move || {
            let res = store1.query_events(EventQuery::default()).unwrap();
            assert!(!res.is_empty());
        });

        let t2 = task::spawn_blocking(move || {
            let res = store2.query_events(EventQuery::default()).unwrap();
            assert!(!res.is_empty());
        });

        t1.await.unwrap();
        t2.await.unwrap();
    }

    #[tokio::test]
    async fn test_read_write_concurrency() {
        let store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());

        let store_reader = Arc::clone(&store);
        let store_writer = Arc::clone(&store);

        let reader = task::spawn_blocking(move || {
            let _ = store_reader.query_events(EventQuery::default());
        });

        let writer = task::spawn_blocking(move || {
            store_writer
                .add_event(Event {
                    id: Uuid::new_v4(),
                    event_type: "write".into(),
                    timestamp: Utc::now(),
                    payload: serde_json::json!({"val": 42}),
                })
                .unwrap();
        });

        reader.await.unwrap();
        writer.await.unwrap();
    }
}
