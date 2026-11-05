//! The `jukino` command-line interface.
//!
//! This binary is a composition root: the only place that builds adapters and hands
//! them to the core. The [`cli`] module is the driving adapter that turns
//! command-line arguments into calls on the core.

mod cli;

use clap::Parser as _;

fn main() {
    cli::Cli::parse();
}
