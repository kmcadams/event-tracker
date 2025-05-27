use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Deserialize, Default)]
pub struct EventQuery {
    pub event_type: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct NewEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
}

impl NewEvent {
    pub fn into_event(self) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: self.event_type,
            timestamp: self.timestamp,
            payload: self.payload,
        }
    }
}
