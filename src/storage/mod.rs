mod in_memory_storage;

use crate::event::Event;
use crate::event::Timestamp;

pub use in_memory_storage::InMemoryStorage;

/// Error type for storage operations.
#[derive(Debug)]
pub enum StoreError {
    InvalidEventType(String),
}

/// Error type for retrieval operations.
#[derive(Debug)]
pub enum RetrieveError {
    ResultTooLarge(u64),
}
/// Storage trait for event storage.
#[async_trait::async_trait]
pub trait Storage {
    async fn store(&self, event: Event) -> Result<(), StoreError>;

    async fn get_events(
        &self,
        event_type: Option<&str>,
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    ) -> Result<Vec<Event>, RetrieveError>;
}
