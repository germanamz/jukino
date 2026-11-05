//! Repository checks for Jukino, run as `cargo xtask <check>`.
//!
//! Every check is a pure function over source text or data, so it can be tested
//! with fixtures; `main.rs` wires the checks to the command line.

pub mod arch;
pub mod attribution;
pub mod comments;
pub mod coverage;
pub mod finding;
pub mod git;
pub mod guard;
pub mod pipeline;
pub mod runner;
