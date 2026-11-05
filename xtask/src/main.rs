//! Command-line entry point for `cargo xtask`.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "xtask reports findings on the terminal"
)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser as _;
use xtask::finding::Finding;
use xtask::guard::{self, Mode, Outcome};
use xtask::{arch, attribution, comments, git, runner};

/// Jukino's repository checks.
#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Reject comment blocks, block comments and commented-out code.
    Comments {
        /// Files to check; every Rust file in the repository when empty.
        paths: Vec<PathBuf>,
    },
    /// Check crate dependencies and the core's imports against `arch.toml`.
    Arch,
    /// Report changes to existing tests and to gates since the merge base.
    Guard {
        /// The ref to compare against, through its merge base with `HEAD`.
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Fail unless the pull request has the `guard-approved` label.
        #[arg(long)]
        ci: bool,
        /// The pull request's labels, comma separated.
        #[arg(long, value_delimiter = ',')]
        labels: Vec<String>,
    },
    /// Run the tests under coverage and enforce the thresholds.
    Coverage,
    /// Mutation-test the code changed since the merge base.
    Mutants {
        /// The ref to compare against, through its merge base with `HEAD`.
        #[arg(long, default_value = "origin/main")]
        base: String,
    },
    /// Run every check in order, as CI does.
    Ci,
    /// Reject AI attribution in a commit message, a range of commits, or a pull
    /// request's title or description.
    #[command(group(clap::ArgGroup::new("source").required(true)))]
    Attribution {
        /// A commit message file, as passed to the commit-msg hook.
        #[arg(long, group = "source")]
        file: Option<PathBuf>,
        /// A revision range such as `origin/main..HEAD`.
        #[arg(long, group = "source")]
        range: Option<String>,
        /// A pull-request title.
        #[arg(long, group = "source")]
        title: Option<String>,
        /// A pull-request description.
        #[arg(long, group = "source")]
        body: Option<String>,
    },
}

/// Where the messages checked for attribution come from.
enum MessageSource {
    File(PathBuf),
    Range(String),
    Text { label: &'static str, text: String },
}

impl MessageSource {
    fn from_args(
        file: Option<PathBuf>,
        range: Option<String>,
        text: (Option<String>, Option<String>),
    ) -> Self {
        match (file, range, text) {
            (Some(file), _, _) => Self::File(file),
            (None, Some(range), _) => Self::Range(range),
            (None, None, (Some(title), _)) => Self::Text {
                label: "title",
                text: title,
            },
            (None, None, (None, body)) => Self::Text {
                label: "body",
                text: body.unwrap_or_default(),
            },
        }
    }
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Comments { paths } => run_comments(paths),
        Command::Arch => match arch::check_workspace(Path::new(".")) {
            Ok(findings) => report(&findings),
            Err(error) => failure(&format!("arch: {error}")),
        },
        Command::Guard { base, ci, labels } => {
            let mode = if ci { Mode::Ci { labels } } else { Mode::Local };
            run_guard(&base, &mode)
        }
        Command::Coverage => runner::coverage(),
        Command::Mutants { base } => runner::mutants(&base),
        Command::Ci => runner::ci(),
        Command::Attribution {
            file,
            range,
            title,
            body,
        } => run_attribution(&MessageSource::from_args(file, range, (title, body))),
    }
}

fn run_attribution(source: &MessageSource) -> ExitCode {
    let messages = match attribution_messages(source) {
        Ok(messages) => messages,
        Err(error) => return failure(&error),
    };
    let findings: Vec<Finding> = messages
        .iter()
        .flat_map(|(label, message)| {
            attribution::attribution_lines(message)
                .into_iter()
                .map(move |line| Finding {
                    path: PathBuf::from(label),
                    line,
                    message: "AI attribution: remove this line".to_owned(),
                })
        })
        .collect();
    report(&findings)
}

fn attribution_messages(source: &MessageSource) -> Result<Vec<(String, String)>, String> {
    match source {
        MessageSource::File(file) => {
            let message = std::fs::read_to_string(file)
                .map_err(|error| format!("{}: {error}", file.display()))?;
            Ok(vec![(file.display().to_string(), message)])
        }
        MessageSource::Range(range) => {
            let commits = git::commit_messages(Path::new("."), range)
                .map_err(|error| format!("attribution: {error}"))?;
            Ok(commits
                .into_iter()
                .map(|(hash, message)| {
                    (
                        format!("commit {}", hash.get(..7).unwrap_or(&hash)),
                        message,
                    )
                })
                .collect())
        }
        MessageSource::Text { label, text } => Ok(vec![((*label).to_owned(), text.clone())]),
    }
}

fn run_comments(paths: Vec<PathBuf>) -> ExitCode {
    let paths = if paths.is_empty() {
        match git::rust_files(Path::new(".")) {
            Ok(files) => files,
            Err(error) => return failure(&format!("comments: cannot list files: {error}")),
        }
    } else {
        paths
    };
    let mut findings = Vec::new();
    for path in paths.iter().filter(|path| path.is_file()) {
        match std::fs::read_to_string(path) {
            Ok(source) => findings.extend(comments::check_source(path, &source)),
            Err(error) => return failure(&format!("{}: {error}", path.display())),
        }
    }
    report(&findings)
}

fn run_guard(base: &str, mode: &Mode) -> ExitCode {
    let findings = match guard::check(Path::new("."), base) {
        Ok(findings) => findings,
        Err(error) => return failure(&format!("guard: {error}")),
    };
    for finding in &findings {
        println!("{finding}");
    }
    match guard::decide(&findings, mode) {
        Outcome::Clean => ExitCode::SUCCESS,
        Outcome::Warn => {
            println!("guard: stop and report these to the contributor");
            ExitCode::SUCCESS
        }
        Outcome::Fail => {
            println!("guard: add the guard-approved label once a contributor has reviewed these");
            ExitCode::from(1)
        }
    }
}

fn report(findings: &[Finding]) -> ExitCode {
    for finding in findings {
        println!("{finding}");
    }
    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn failure(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(2)
}
