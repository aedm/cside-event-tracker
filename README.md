# TODO

- use SmallVec to store event ids
- use libsql for vanity
- add readme
- add error handling
- add tracing
- add observability
- add docker etwas


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


