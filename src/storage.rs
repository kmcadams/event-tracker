use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::{Event, EventQuery};

pub trait EventStore: Send + Sync {
    fn add_event(&self, event: Event) -> Result<(), AppError>;
    fn query_events(&self, query: EventQuery) -> Result<Vec<Event>, AppError>;
    fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, AppError>;
}

pub struct InMemoryEventStore {
    events: RwLock<HashMap<Uuid, Event>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
        }
    }
}

impl EventStore for InMemoryEventStore {
    fn add_event(&self, event: Event) -> Result<(), AppError> {
        let mut events = self
            .events
            .write()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        events.insert(event.id, event);
        Ok(())
    }

    fn query_events(&self, query: EventQuery) -> Result<Vec<Event>, AppError> {
        let events = self
            .events
            .read()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let result = events
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
        Ok(result)
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, AppError> {
        let events = self
            .events
            .read()
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Ok(events.get(&id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use serde_json::json;

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
        };

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.events.write().unwrap();
            panic!("simulate panic while holding write lock");
        }));

        let event = sample_event(None, "test", "2025-01-01T12:00:00Z");
        let result = store.add_event(event);
        assert!(matches!(result, Err(AppError::InternalError(_))));
    }
}
