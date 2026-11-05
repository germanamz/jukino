//! Jukino's core: the domain model and the ports through which it talks to the
//! outside world.
//!
//! The core holds every business rule and no technology. It performs no I/O and
//! never reads the clock, the environment or a random source. Requests, folders,
//! collections, graph-middleware and chain runs are domain concepts here; HTTP
//! clients, storage and JavaScript engines live in adapters that implement the
//! core's ports.

mod request_path;

pub use request_path::{RequestPath, RequestPathError};
