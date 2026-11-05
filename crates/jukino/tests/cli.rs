//! Smoke tests that boot the real `jukino` binary.
#![expect(
    clippy::expect_used,
    reason = "a binary that fails to launch is a test failure"
)]

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jukino"))
        .args(args)
        .output()
        .expect("the jukino binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn version_flag_prints_the_package_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), "jukino 0.1.0\n");
}

#[test]
fn help_flag_describes_the_command() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    assert!(text(&output.stdout).contains("Usage: jukino"));
}

#[test]
fn unknown_flag_is_a_usage_error() {
    let output = run(&["--bogus"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("unexpected argument"));
}

#[test]
fn no_arguments_prints_help_as_a_usage_error() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("Usage: jukino"));
}
