use ahash::AHashMap;
use std::{
    collections::BTreeMap,
    ops::Bound,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{
    event::{Event, Timestamp},
    storage::{RetrieveError, Storage, StoreError},
};

// An internal identifier for events.
type EventId = u64;
static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

// Made-up restriction to demonstrate error handling.
const MAX_QUERIED_EVENTS: usize = 4;
/// Stores events in an indexed manner for efficient queries.
struct IndexedEvents {
    /// Stores events by their internal identifier.
    event_by_id: AHashMap<EventId, Event>,

    /// Stores events by their timestamp. This allows for efficient range queries.
    events_by_timestamp: BTreeMap<Timestamp, Vec<EventId>>,

    /// Stores events by their type and timestamp. This allows for efficient range queries by type.
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

#[async_trait::async_trait]
impl Storage for InMemoryStorage {
    #[instrument(skip_all)]
    async fn store(&self, event: Event) -> Result<(), StoreError> {
        debug!("Storing event");
        if event.event_type == "winter wrap up" {
            // In-memory storage doesn't support this event type.
            // It's a made-up restriction to demonstrate error handling.
            return Err(StoreError::InvalidEventType(event.event_type));
        }

        let event_id = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);
        let event_type = event.event_type.clone();

        let mut events_guard = self.events.write().await;
        events_guard
            .events_by_type_by_timestamp
            .entry(event_type)
            .or_default()
            .entry(event.timestamp)
            .or_default()
            .push(event_id);
        events_guard
            .events_by_timestamp
            .entry(event.timestamp)
            .or_default()
            .push(event_id);
        events_guard.event_by_id.insert(event_id, event);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn get_events(
        &self,
        event_type: Option<&str>,
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    ) -> Result<Vec<Event>, RetrieveError> {
        debug!("Getting events");
        let events_guard = self.events.read().await;

        // Filter by event type, if specified
        let events = if let Some(event_type) = event_type {
            match events_guard.events_by_type_by_timestamp.get(event_type) {
                Some(events) => events,
                None => return Ok(vec![]),
            }
        } else {
            &events_guard.events_by_timestamp
        };

        // Filter by timestamp range, if specified
        let start = match start {
            Some(start) => Bound::Included(start),
            _ => Bound::Unbounded,
        };
        let end = match end {
            Some(end) => Bound::Included(end),
            _ => Bound::Unbounded,
        };

        // Get events in the specified range. Make sure not to return more than MAX_QUERIED_EVENTS.
        let result: Vec<_> = events
            .range((start, end))
            .flat_map(|(_, event_ids)| {
                event_ids
                    .iter()
                    // All ids should exist so a flat_map is appropriate.
                    .flat_map(|event_id| events_guard.event_by_id.get(event_id).cloned())
            })
            .take(MAX_QUERIED_EVENTS + 1)
            .collect();

        if result.len() > MAX_QUERIED_EVENTS {
            return Err(RetrieveError::ResultTooLarge(MAX_QUERIED_EVENTS as u64));
        }

        debug!("Found {} events", result.len());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    #[tokio::test]
    async fn test_filtering() {
        let event_1 = Event {
            event_type: "login".to_string(),
            timestamp: 4,
            payload: serde_json::json!({ "user_id": 123, "ip": "127.0.0.4" }),
        };
        let event_2 = Event {
            event_type: "login".to_string(),
            timestamp: 5,
            payload: serde_json::json!({ "user_id": 123, "ip": "127.0.0.5" }),
        };
        let event_3 = Event {
            event_type: "foo".to_string(),
            timestamp: 6,
            payload: serde_json::json!({ "user_id": 123, "ip": "127.0.0.6" }),
        };
        let store = InMemoryStorage::new();

        store.store(event_1.clone()).await.unwrap();
        store.store(event_2.clone()).await.unwrap();
        store.store(event_3.clone()).await.unwrap();

        assert_eq!(
            store.get_events(None, None, None).await.unwrap(),
            vec![event_1.clone(), event_2.clone(), event_3.clone()]
        );
        assert_eq!(
            store.get_events(None, Some(5), None).await.unwrap(),
            vec![event_2.clone(), event_3.clone()]
        );
        assert_eq!(
            store.get_events(None, None, Some(5)).await.unwrap(),
            vec![event_1.clone(), event_2.clone()]
        );
        assert_eq!(
            store.get_events(Some("login"), None, None).await.unwrap(),
            vec![event_1.clone(), event_2.clone()]
        );
        assert_eq!(
            store
                .get_events(Some("login"), Some(5), None)
                .await
                .unwrap(),
            vec![event_2.clone()]
        );
        assert_eq!(
            store
                .get_events(Some("login"), None, Some(5))
                .await
                .unwrap(),
            vec![event_1.clone(), event_2.clone()]
        );
        assert_eq!(
            store
                .get_events(Some("login"), Some(5), Some(5))
                .await
                .unwrap(),
            vec![event_2.clone()]
        );
    }
}
