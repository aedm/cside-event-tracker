mod in_memory_storage;

use crate::event::Event;
use crate::event::Timestamp;

pub use in_memory_storage::InMemoryStorage;

pub enum StoreError {
    InvalidEventType(String),
}

pub enum RetrieveError {
    ResultTooLarge(u64),
}

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