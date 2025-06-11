use std::{
    collections::BTreeMap,
    ops::Bound,
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
};
use ahash::AHashMap;

use crate::{event::{Event, Timestamp}, storage::Storage};

type EventId = u64;

struct IndexedEvents {
    event_by_id: AHashMap<EventId, Event>,
    events_by_timestamp: BTreeMap<Timestamp, Vec<EventId>>,
    events_by_type_by_timestamp: AHashMap<String, BTreeMap<Timestamp, Vec<EventId>>>,
}

pub struct InMemoryStorage {
    // Using a single lock for all indexes is not the most performant but it's good enough
    // and avoids data race issues of updating indexes separately. Faster alternatives
    // exist (eg. fences or eventual consistency) at the cost of complexity or consistency.
    events: RwLock<IndexedEvents>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(IndexedEvents {
                event_by_id: AHashMap::new(),
                events_by_type_by_timestamp: AHashMap::new(),
                events_by_timestamp: BTreeMap::new(),
            }),
        }
    }
}

static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

impl Storage for InMemoryStorage {
    fn store(&self, event: Event) {
        let event_id = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);

        let mut events = self.events.write().unwrap();
        events
            .events_by_type_by_timestamp
            .entry(event.event_type.to_string())
            .or_default()
            .entry(event.timestamp)
            .or_default()
            .push(event_id);
        events
            .events_by_timestamp
            .entry(event.timestamp)
            .or_default()
            .push(event_id);
        events.event_by_id.insert(event_id, event);
    }

    fn get_events(
        &self,
        event_type: Option<&str>,
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    ) -> Vec<Event> {
        let events_lock = self.events.read().unwrap();

        // Filter by event type, if specified
        let events = if let Some(event_type) = event_type {
            match events_lock.events_by_type_by_timestamp.get(event_type) {
                Some(events) => events,
                None => return vec![],
            }
        } else {
            &events_lock.events_by_timestamp
        };

        // Filter by timestamp range, if specified
        let start = match start {
            Some(start) => Bound::Included(start),
            None => Bound::Unbounded,
        };
        let end = match end {
            Some(end) => Bound::Included(end),
            None => Bound::Unbounded,
        };
        events
            .range((start, end))
            .flat_map(|(_, event_ids)| {
                event_ids.iter().map(|event_id| {
                    let k = events_lock.event_by_id.get(event_id).unwrap().clone();
                    k
                })
            })
            .collect()
    }
}


mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_in_memory_storage() {
        let event_1 = Event {
            event_type: "login".to_string(),
            timestamp: 4,
            payload: json!({ "user_id": 123, "ip": "127.0.0.4" }),
        };
        let event_2 = Event {
            event_type: "login".to_string(),
            timestamp: 5,
            payload: json!({ "user_id": 123, "ip": "127.0.0.5" }),
        };
        let event_3 = Event {
            event_type: "foo".to_string(),
            timestamp: 6,
            payload: json!({ "user_id": 123, "ip": "127.0.0.6" }),
        };
        let store = InMemoryStorage::new();

        store.store(event_1.clone());
        store.store(event_2.clone());
        store.store(event_3.clone());

        assert_eq!(
            store.get_events(None, None, None),
            vec![event_1, event_2, event_3]
        );
        assert_eq!(store.get_events(None, Some(5), None), vec![event_2]);
        assert_eq!(store.get_events(None, None, Some(5)), vec![event_2]);
        assert_eq!(
            store.get_events(Some("login"), None, None),
            vec![event_1, event_2]
        );
        assert_eq!(
            store.get_events(Some("login"), Some(5), None),
            vec![event_2]
        );
        assert_eq!(
            store.get_events(Some("login"), None, Some(5)),
            vec![event_2]
        );
        assert_eq!(
            store.get_events(Some("login"), Some(5), Some(5)),
            vec![event_2]
        );
    }
}
