//! Launches the external tools described in [`crate::pipeline`].
//!
//! This module only starts processes and reports their results; every decision
//! lives in tested code elsewhere. It is excluded from coverage and mutation
//! testing.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the runner reports progress on the terminal"
)]

use std::path::Path;
use std::process::{Command, ExitCode};

use crate::pipeline::{self, Step};
use crate::{coverage, git};

fn run(step: &Step) -> bool {
    Command::new(step.program)
        .args(step.args)
        .envs(step.env.iter().copied())
        .status()
        .is_ok_and(|status| status.success())
}

fn is_on_path(executable: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            dir.join(executable).is_file() || dir.join(format!("{executable}.exe")).is_file()
        })
    })
}

/// `cargo xtask ci`: every check, in order.
#[must_use]
pub fn ci() -> ExitCode {
    let missing = pipeline::missing_tools(is_on_path);
    if !missing.is_empty() {
        for message in &missing {
            eprintln!("{message}");
        }
        return ExitCode::from(2);
    }
    let result = pipeline::run_steps(&pipeline::CI_STEPS, |step| {
        println!("==> {}", step.name);
        run(step)
    });
    match result {
        Ok(()) => {
            println!("ci: all checks passed");
            ExitCode::SUCCESS
        }
        Err(name) => {
            eprintln!("ci: {name} failed");
            ExitCode::from(1)
        }
    }
}

/// `cargo xtask coverage`: run the tests under coverage and check the thresholds.
#[must_use]
pub fn coverage() -> ExitCode {
    if !run(&pipeline::COVERAGE) {
        eprintln!("coverage: the instrumented test run failed");
        return ExitCode::from(1);
    }
    let report = match std::fs::read_to_string(pipeline::COVERAGE_REPORT) {
        Ok(report) => report,
        Err(error) => {
            eprintln!(
                "coverage: cannot read {}: {error}",
                pipeline::COVERAGE_REPORT
            );
            return ExitCode::from(2);
        }
    };
    match coverage::evaluate(&report, "crates/jukino-core/", &coverage::THRESHOLDS) {
        Ok(violations) if violations.is_empty() => {
            println!("coverage: thresholds met");
            ExitCode::SUCCESS
        }
        Ok(violations) => {
            for violation in &violations {
                println!("coverage: {violation}");
            }
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("coverage: unreadable report: {error}");
            ExitCode::from(2)
        }
    }
}

/// `cargo xtask mutants`: mutation-test the code changed since `base`.
#[must_use]
pub fn mutants(base: &str) -> ExitCode {
    let diff = match git::mutation_diff(Path::new("."), base) {
        Ok(Some(diff)) => diff,
        Ok(None) => {
            println!("mutants: no changes to mutate");
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("mutants: {error}");
            return ExitCode::from(2);
        }
    };
    if let Err(error) = std::fs::write(pipeline::MUTANTS_DIFF, diff) {
        eprintln!("mutants: cannot write {}: {error}", pipeline::MUTANTS_DIFF);
        return ExitCode::from(2);
    }
    if run(&pipeline::MUTANTS) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
