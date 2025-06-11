mod in_memory_storage;

use crate::event::Event;
use crate::event::Timestamp;

pub use in_memory_storage::InMemoryStorage;

pub trait Storage {
    fn store(&self, event: Event);

    fn get_events(
        &self,
        event_type: Option<&str>,
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    ) -> Vec<Event>;
}