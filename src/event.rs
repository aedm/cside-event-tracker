use serde::{Deserialize, Serialize};

pub type Timestamp = u64;

/// The event type we need to store.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Event {
    pub event_type: String,
    pub timestamp: Timestamp,
    pub payload: serde_json::value::Value,
}
