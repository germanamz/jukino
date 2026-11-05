//! The `jukino-ui` binary: a placeholder for Jukino's user interface.
//!
//! It reserves the crate and its place in the architecture. Once a UI framework is
//! chosen, this binary becomes the UI's composition root, and the UI its driving
//! adapter. Until then it only answers `--help` and `--version` through [`cli`].

mod cli;

use clap::Parser as _;

fn main() {
    cli::Cli::parse();
}
