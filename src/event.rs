use serde::{Deserialize, Serialize};

pub type Timestamp = u64;

// Use zero-copy deserialization where possible. Unfortunately serde_json::value::Value
// doesn't support borrowing and RawValue is only a &str. This means we have to copy
// the payload into a new Value. Use sonic_rs instead if zero-copy deserialization is
// a priority.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Event {
    pub event_type: String,
    pub timestamp: Timestamp,
    pub payload: serde_json::value::Value,
}
