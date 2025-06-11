# Section 1: Technical Assessment Questions

## 1. Concurrency and Asynchronous Programming:
_How would you design a task scheduler in Rust using **tokio** or **async-std**? What are the key considerations for error handling and state management in an async context?_

A basic task scheduler defers the execution of a task to single or multiple later points in time.

A simple implementation could just wrap the task callback with a `sleep` and spawn it in a [`JoinSet`](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) to make sure tasks don't outlive the scheduler.  [Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=74c4568184173c1c880b04ba5c5c6bba) 

**Error handling in schedulers:** since the execution of tasks is decoupled from the caller, the scheduler itself has to collect and handle errors. A simple implementation uses an error handler trait and adds it to the task wrapper. [Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=f60ebc381a11c2d77fb62d2727509012)

**Error types:**  Here, I simply used `anyhow` but in a real-life setting, a use case specific error trait might be more appropriate to communicate additional details like the task id and the reason of failure. The crate [thiserror](https://docs.rs/thiserror/latest/thiserror/) is usually helpful. 

**State sharing in async context:** This is not specific to task scheduling, general Rust rules apply. In case of Tokio, shared state is governed by `Sync` and `Send` traits since [`spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html) itself requires callbacks to implement `Send`. In general, immutable state is usually shared using `Arc` or `&'static` references. Mutable state requires synchronization primitives (like `Mutex`, `RwLock` etc.), atomics, channels, thread-safe collections (like the excellent `DashMap`) and so on.

Shared references to the stack are not allowed due to Rust's ["scoped task trilemma"](https://without.boats/blog/the-scoped-task-trilemma/): you can only pick two out of concurrency, parallelizability and borrowing, and Tokio's `spawn` sacrifices borrowing. This is a very "Rusty" problem: most other languages either don't allow references to stack at all (Java, Go etc), and those that allow it also allow UB (C++, Zig).

Single thread state sharing (`Rc`, `RefCell` etc.) can be used in async contexts with "thread-per-task" async runtimes like [glommio](https://github.com/DataDog/glommio) or [monoio](https://github.com/bytedance/monoio). Their `spawn` methods do not require `Sync`.

The topic of shared state is kinda huge, and I'm happy to discuss it further on a live interview.

**Additional possible task scheduler features:** 
- Repeating tasks: use a loop inside the wrapper.
- Waiting for tasks to finish: `JoinSet` has `join_all()`.
- Aborting tasks: use the `AbortHandler` returned by `spawn`.
- Querying tasks state: the `AbortHandler` can do that, too.
- Task ID, run count, retry count, priority queue... I'm happy to discuss any of them live.

## 2. Error Handling:
_Rust’s error handling uses **Result** and **Option**. Provide a brief example of a nested error handling scenario and how you would propagate and handle errors effectively._

Example code: load a text file where each line is an integer, return the largest one of them.

This code demonstrates the following concepts:
- Defining a custom error type
- Wrapping error types into each other
- Converting between different error types
- Converting `Option` to `Result`
- Formatting with `thiserror`.
- Use the `?` operator to quickly return a [residual](https://doc.rust-lang.org/std/ops/trait.Residual.html) .

Note: the code would normally be smaller and faster, but I wanted to highlight the nested error handling aspect here and split the functionality into multiple functions. 

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error opening file '{1}': {0}")]
    IoError(std::io::Error, String),

    #[error("Can't parse line: {0}")]
    ParseError(String),

    #[error("File is empty")]
    FileIsEmpty,
}

fn load_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// Not optimal to collect numbers into a Vec, just a demonstration of error handling.
fn parse_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Vec<i64>, MyError> {
    lines
        .map(|line| {
            line.parse::<i64>()
                .map_err(|_| MyError::ParseError(line.to_string()))
        })
        .collect()
}

pub fn get_largest_number_from_file(path: &str) -> Result<i64, MyError> {
    let content = load_file(path).map_err(|err| MyError::IoError(err, path.to_string()))?;
    let numbers = parse_lines(content.lines())?;
    let largest = numbers.into_iter().max().ok_or(MyError::FileIsEmpty)?;
    Ok(largest)
}

// Example usage:
pub fn main() {
    let result = get_largest_number_from_file("example.txt");
    match result {
        Ok(largest) => println!("Largest number: {}", largest),
        Err(err) => println!("Error: {}", err),
    }
}
```

## 3. Memory Management and Safety:
_Explain how Rust ensures memory safety without a garbage collector. How would you handle a scenario where multiple threads need to access shared data?_

Rust's memory safety has multiple pillars.
- **Ownership model:** the borrow checker makes it sure that at any point in time, there is at most one reference with mutability rights to any given object.
- **Runtime checks:** where mutability can't be evaluated at build time, we can use primitives that provide "interior mutability" at run time (`RefCell`, `Mutex`, `AtomicU32` etc.). There are further checks like array range checks to avoid undefined behavior.
- **Lifecycle management:** Rust borrows (haha) the concept of [RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization) from C++ which ties allocated resources to the lifecycle of their owners. As soon as an object doesn't have an owner anymore, all resources are freed immediately, no need to wait for a garbage collector. The concept of RAII not only applies to memory, but many other resources like file handlers, database connections etc.

For example, "use-after-free" is avoided here:
```rust
let mut x = vec![42];
x[0] = 37; // totally random number
drop(x);
x[0] = 73; // COMPILE ERROR
```
The `drop` function takes ownership of `x` and drops allocated resources. After that, the function scope no longer has ownership.

## 4. Data Structures and Algorithms:
_Which Rust data structures would you use to implement a cache with TTL (Time-to-Live) functionality? Why?_

A TTL cache is a data type with two main query types:
- Access item by key
- Get all items who's last access happened before a given threshold.
This requires maintaining two indexes. A simple key based lookup is usually performed by a `HashMap` (or an alternative like `AHashMap`). 

Range-based queries are more effectively handled by self-balancing trees, like a B-Tree: they allow storing items in order, where inserting and retrieving are both O(logN) operations.

I'd use `BTreeSet` to store items ordered by expiration date, and provide an ordering function derived from the last access time.
- When an item is accessed, it gets removed from the set, its access time gets updated, and reinserted into the set again.
- When the cleanup function runs, I'd get all outdated items with something like [`BTreeSet::split_off`](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html#method.split_off)and drop outdated keys from the hashmap as well.

## 5. System Design:
_You are designing a high-throughput logging service in Rust that needs to handle millions of log entries per second. How would you architect this service? Which Rust libraries/crates would you consider?_

(I'll assume this is not about an in-process logger.)

Let's break it down into a pipeline of stages so each part is as effective as can be: Load Balancing, Ingestion, Processing and Output.

**Load balancing:** Depending on the task, processing requirements, available hardware and other factors, a single machine may or may not be enough to process millions of messages per second. Preparing the system to scale vertically is advised. Distributing the load between nodes is essential: tools to achieve this include load balancers, Kafka partitions, and many others. This part is largely independent of Rust.

**Ingestion:** Such a high load almost certainly requires some sort of async runtime. Tokio is the most widely used one, but there are a few others that might be more performant, albeit less mature.
Providing a REST API endpoints is a widely used approach. In that case, high-performance, mature crates include Actix, Axum or Warp used in conjunction with Serde to deserialize JSON. But if applicable, more efficient alternatives exist, like gRPC provided by Tonic. Other data source alternatives include MQTT (mostly embedded devices) Kafka or Amazon SNS. 

Consider zero-copy deserialization: Serde can do that for strings without JSON escape sequences, but there are zero-copy formats like Cap'n Proto.

Sharded containers

Parking_lot

**Processing:** This stage might include filtering, labeling, or collecting statistics. Use case dependent.

**Batching / Aggregation:** It's usually not a good idea to output all pieces of data individually. Instead, we should but to batch them, or alternatively, only output aggregate information. Between processing and batching, one could consider `crossbeam-channel`, `flume` or using a ring buffer. Also, one might consider compression here, Zstd is one of the neat crates to achieve that.

**Output:** Again, highly dependent on the use case. `Elasticsearch` is a widely used database to store logs, but event stream services (like Kafka), cloud storage (S3), local file systems etc. could all be the targets.

Other crates to consider:
- "tracing" provides structured, thread-safe traces
- "tracing-opentelemetry" sends those traces to an OpenTelemetry-compatible receiver
- "parking_lot" and "rayon" might be helpful for data processing, depending on the use case
- "opentelemetry" or "prometheus" for observability

