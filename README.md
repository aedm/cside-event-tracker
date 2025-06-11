A simple event tracking service. Might be a homework.


## Quick start

Install Rust, then run

```bash
cargo run --release
```

The server runs on `http://localhost:3000`.


## Usage

There are two endpoints:

- `POST /events`
    - Stores an event.
    - Accepts a JSON object with the following fields:
        - `event_type`: the type of the event
        - `timestamp`: the timestamp of the event
        - `payload`: the payload of the event
- `GET /events`
    - Returns a list of events.
    - Accepts the following query parameters:
        - `event_type`: the type of the event
        - `start`: the start timestamp
        - `end`: the end timestamp


## Notes about the implementation

- The in-memory storage maintains double indexing for efficient queries. It opts to use a single `RwLock` for all indexes to avoid data race issues of updating indexes separately. Faster alternatives exist (eg. fences or eventual consistency) at the cost of complexity or consistency.

- The `Event` type is deserialized into an owned value using `serde`. A faster approach would be to use a zero-copy deserialization library like sonic_rs. However, in this case, the event object needs to be stored as-is in the index, so a zero-copy deserialization approach would not make a difference. But it could improve the performance if there was more filtering.

- Usually I like returning error values in a unified format, hence the `app_error` module.


## TODO

Things I didn't find time to do:

- Use SmallVec to store event ids for better performance.
- Add a libsql-based storage.
- Add tracing.
- Add observability.
- Add docker containerization.
- Make API tests run in CI.
- Move API tests to `/tests` for better organization.