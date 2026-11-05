//! Command-line parsing for the placeholder UI binary.

/// The Jukino user interface (not built yet).
#[derive(clap::Parser)]
#[command(version, arg_required_else_help = true)]
pub(crate) struct Cli {}
