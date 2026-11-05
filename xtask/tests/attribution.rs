//! Behaviour of `cargo xtask attribution`, which rejects AI attribution lines.
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use std::path::Path;
use std::process::{Command, Output};

use xtask::attribution::attribution_lines;

#[test]
fn ai_co_author_trailers_are_flagged() {
    let message =
        "feat: add x\n\nWhy it matters.\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";
    assert_eq!(attribution_lines(message), [5]);
    for name in [
        "Claude",
        "Anthropic",
        "GitHub Copilot",
        "ChatGPT",
        "OpenAI",
        "Gemini",
        "Codex",
        "Cursor",
    ] {
        assert_eq!(
            attribution_lines(&format!("fix: y\n\nco-authored-by: {name} <a@b.c>\n")),
            [3],
            "{name}"
        );
    }
}

#[test]
fn session_trailers_and_links_are_flagged() {
    assert_eq!(attribution_lines("fix: y\n\nClaude-Session: abc\n"), [3]);
    assert_eq!(
        attribution_lines("fix: y\n\nSee https://claude.ai/code/session_01abc\n"),
        [3]
    );
}

#[test]
fn generated_with_lines_are_flagged() {
    assert_eq!(
        attribution_lines("docs: z\n\nGenerated with [Claude Code](https://claude.com)\n"),
        [3]
    );
    assert_eq!(
        attribution_lines("docs: z\n\nGenerated with care\n"),
        Vec::<usize>::new()
    );
}

#[test]
fn human_co_authors_and_prose_are_not_flagged() {
    let message =
        "feat: x\n\nWe tried Claude Code for reviews.\n\nCo-authored-by: Ana <ana@example.com>\n";
    assert_eq!(attribution_lines(message), Vec::<usize>::new());
}

fn xtask(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir)
        .arg("attribution")
        .args(args)
        .output()
        .expect("xtask runs")
}

#[test]
fn the_command_checks_a_message_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("bad"), "fix: a\n\nClaude-Session: x\n").expect("write");
    std::fs::write(dir.path().join("good"), "fix: a\n").expect("write");
    let bad = xtask(dir.path(), &["--file", "bad"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stdout).contains("bad:3: AI attribution"));
    assert_eq!(
        xtask(dir.path(), &["--file", "good"]).status.code(),
        Some(0)
    );
    assert_eq!(
        xtask(dir.path(), &["--file", "missing"]).status.code(),
        Some(2)
    );
}

#[test]
fn the_command_checks_a_title() {
    let dir = tempfile::tempdir().expect("temp dir");
    assert_eq!(
        xtask(dir.path(), &["--title", "feat: add x"]).status.code(),
        Some(0)
    );
    let bad = xtask(dir.path(), &["--title", "feat: x, generated with Claude"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stdout).contains("title:1: "));
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

fn commit(dir: &Path, message: &str) {
    git(
        dir,
        &[
            "commit",
            "--quiet",
            "--no-verify",
            "--allow-empty",
            "-m",
            message,
        ],
    );
}

fn history() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    git(dir.path(), &["init", "--quiet"]);
    commit(dir.path(), "chore: start");
    git(dir.path(), &["tag", "start"]);
    commit(dir.path(), "feat: a");
    dir
}

#[test]
fn a_clean_range_passes() {
    let dir = history();
    assert_eq!(
        xtask(dir.path(), &["--range", "start..HEAD"]).status.code(),
        Some(0)
    );
}

#[test]
fn a_range_with_an_attributed_commit_fails() {
    let dir = history();
    commit(dir.path(), "fix: b\n\nClaude-Session: x");
    let bad = xtask(dir.path(), &["--range", "start..HEAD"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stdout).contains(":3: AI attribution"));
}

#[test]
fn an_unknown_range_exits_2() {
    let dir = history();
    assert_eq!(
        xtask(dir.path(), &["--range", "nope..HEAD"]).status.code(),
        Some(2)
    );
}

#[test]
fn the_command_checks_a_pull_request_body() {
    let dir = tempfile::tempdir().expect("temp dir");
    assert_eq!(
        xtask(dir.path(), &["--body", "## Summary\n\nAdds x.\n"])
            .status
            .code(),
        Some(0)
    );
    let bad = xtask(
        dir.path(),
        &[
            "--body",
            "## Summary\n\nhttps://claude.ai/code/session_abc\n",
        ],
    );
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stdout).contains("body:3: AI attribution"));
}
