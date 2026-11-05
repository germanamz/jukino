//! Command-line parsing: the driving adapter between the terminal and the core.

/// API interaction for humans and agents.
#[derive(clap::Parser)]
#[command(version, arg_required_else_help = true)]
pub(crate) struct Cli {}
