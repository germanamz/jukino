//! Behaviour of the Claude Code hook scripts in `.claude/hooks/`.
#![cfg(unix)]
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use std::ffi::OsStr;
use std::io::{ErrorKind, Write as _};
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn hook_with_path(script: &str, input: &serde_json::Value, path: Option<&str>) -> Output {
    let path = path.map(|path| ("PATH", OsStr::new(path)));
    hook_with_env(script, input, path.as_slice())
}

fn hook_with_env(script: &str, input: &serde_json::Value, env: &[(&str, &OsStr)]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let mut command = Command::new("/bin/bash");
    command
        .arg(root.join(".claude/hooks").join(script))
        .current_dir(root)
        .env("CLAUDE_PROJECT_DIR", root)
        .envs(env.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("bash runs");
    let written = child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.to_string().as_bytes());
    assert!(
        written.as_ref().is_ok()
            || written.is_err_and(|error| error.kind() == ErrorKind::BrokenPipe)
    );
    child.wait_with_output().expect("hook finishes")
}

fn hook(script: &str, input: &serde_json::Value) -> Output {
    hook_with_path(script, input, None)
}

fn bash(command: &str) -> Output {
    hook(
        "guard-bash.sh",
        &serde_json::json!({ "tool_input": { "command": command } }),
    )
}

fn decision(output: &Output) -> Option<String> {
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json["hookSpecificOutput"]["permissionDecision"]
        .as_str()
        .map(str::to_owned)
}

/// A Claude Code session whose review-gate state lives in its own temporary directory.
struct Session {
    tmp: tempfile::TempDir,
    id: &'static str,
}

impl Session {
    fn new(id: &'static str) -> Self {
        let tmp = tempfile::tempdir().expect("temporary directory");
        Self { tmp, id }
    }

    fn hook(&self, script: &str, fields: serde_json::Value) -> Output {
        let mut input = fields;
        input["session_id"] = self.id.into();
        hook_with_env(script, &input, &[("TMPDIR", self.tmp.path().as_os_str())])
    }

    fn bash(&self, command: &str, tool_use_id: &str) -> Output {
        self.hook(
            "guard-bash.sh",
            serde_json::json!({
                "hook_event_name": "PreToolUse",
                "tool_name": "Bash",
                "tool_use_id": tool_use_id,
                "tool_input": { "command": command },
            }),
        )
    }

    fn decision(&self, command: &str, tool_use_id: &str) -> Option<String> {
        let output = self.bash(command, tool_use_id);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        decision(&output)
    }

    fn record(&self, fields: serde_json::Value) {
        let output = self.hook("review-wave.sh", fields);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, Vec::<u8>::new(), "the hook adds no context");
    }

    fn finish(&self, event: &str, tool_use_id: &str) {
        self.record(serde_json::json!({
            "hook_event_name": event,
            "tool_name": "Bash",
            "tool_use_id": tool_use_id,
        }));
    }

    fn message(&self) {
        self.record(serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "prompt": "next step",
        }));
    }

    fn open_wave(&self) {
        let first = self.decision("git commit -m x", "approved");
        assert_eq!(first.as_deref(), Some("ask"));
        self.finish("PostToolUse", "approved");
    }
}

#[test]
fn an_approved_commit_lets_commits_and_pushes_through_until_the_next_message() {
    let session = Session::new("wave-session");
    assert_eq!(
        session.decision("git commit -m x", "t1").as_deref(),
        Some("ask")
    );
    session.finish("PostToolUse", "t1");
    for command in [
        "git push origin chore/x",
        "git commit -m y",
        "cd /repo && git add a b && git commit -F /tmp/message && git push",
        "git -C /repo push -u origin chore/x",
    ] {
        let decision = session.decision(command, "t2");
        assert_eq!(decision.as_deref(), Some("allow"), "{command}");
    }
    session.message();
    assert_eq!(session.decision("git push", "t3").as_deref(), Some("ask"));
}

#[test]
fn an_approved_commit_that_fails_still_opens_the_wave() {
    let session = Session::new("wave-session");
    assert_eq!(
        session.decision("git commit -m x", "t1").as_deref(),
        Some("ask")
    );
    session.finish("PostToolUseFailure", "t1");
    assert_eq!(session.decision("git push", "t2").as_deref(), Some("allow"));
}

#[test]
fn a_denied_commit_opens_no_wave() {
    let session = Session::new("wave-session");
    assert_eq!(
        session.decision("git commit -m x", "t1").as_deref(),
        Some("ask")
    );
    session.finish("PostToolUse", "t2");
    assert_eq!(session.decision("git push", "t3").as_deref(), Some("ask"));
}

#[test]
fn inside_a_wave_pull_requests_still_ask() {
    let session = Session::new("wave-session");
    session.open_wave();
    for command in [
        "gh pr create --title x",
        "gh pr merge 3",
        "git push && gh pr create --title x",
        "gh api repos/a/b/pulls -f title=x",
    ] {
        let decision = session.decision(command, "t2");
        assert_eq!(decision.as_deref(), Some("ask"), "{command}");
    }
}

#[test]
fn inside_a_wave_other_commands_keep_the_normal_permission_checks() {
    let session = Session::new("wave-session");
    session.open_wave();
    for command in [
        "git commit -m x && rm -rf target",
        "git push 2>&1 | tail -1",
        "git commit -m x && git reset --hard HEAD~1",
        "sudo git push",
        "ls",
    ] {
        assert_eq!(session.decision(command, "t2"), None, "{command}");
    }
}

#[test]
fn inside_a_wave_bypasses_are_still_blocked() {
    let session = Session::new("wave-session");
    session.open_wave();
    for command in ["git push --force", "git commit -n -m x"] {
        let output = session.bash(command, "t2");
        assert_eq!(output.status.code(), Some(2), "{command} should be blocked");
    }
}

#[test]
fn session_ids_that_are_not_plain_names_open_no_wave() {
    let session = Session::new("../escape");
    assert_eq!(
        session.decision("git commit -m x", "t1").as_deref(),
        Some("ask")
    );
    session.finish("PostToolUse", "t1");
    assert_eq!(session.decision("git push", "t2").as_deref(), Some("ask"));
}

fn assert_blocked(commands: &[&str]) {
    for command in commands {
        let output = bash(command);
        assert_eq!(output.status.code(), Some(2), "{command} should be blocked");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Blocked"),
            "{command}"
        );
    }
}

#[test]
fn commands_that_bypass_checks_are_blocked() {
    assert_blocked(&[
        "git commit --no-verify -m x",
        "git commit -n -m x",
        "git commit -nm x",
        "git commit -anm x",
        "git -c core.hooksPath=/dev/null commit -m x",
        "LEFTHOOK=0 git push",
        "LEFTHOOK_EXCLUDE=x git commit -m y",
        "git push -f",
        "git push --force",
        "git push --force-with-lease",
        "git push origin +main",
        "gh pr edit 3 --add-label guard-approved",
    ]);
}

#[test]
fn bypasses_through_other_command_forms_are_blocked() {
    assert_blocked(&[
        "git -C . commit -n -m x",
        "git --no-pager commit -n -m x",
        "/usr/bin/git commit -n -m x",
        "(git commit -n -m x)",
        "echo ok && git push -f origin x",
        "git -C . push -f",
        "git --git-dir .git push --force-w",
        "git commit --no-veri -m x",
        "git push --no-veri",
    ]);
}

#[test]
fn other_ways_to_disable_hooks_are_blocked() {
    assert_blocked(&[
        "LEFTHOOK=false git commit -m x",
        "LEFTHOOK=\"0\" git push",
        "LEFTHOOK='0' git push",
        "git -c core.hookspath=/x commit -m y",
        "git config core.HOOKSPATH /x",
        "lefthook uninstall",
    ]);
}

#[test]
fn every_form_of_commit_push_and_pull_request_asks_for_permission() {
    for command in [
        "git commit -m \"feat: x\"",
        "git -C /repo commit -m x",
        "git --no-pager commit -m x",
        "/usr/bin/git push origin x",
        "git push origin chore/x",
        "cd /repo && git commit -m x",
        "gh pr create --title x",
        "gh pr merge 3",
        "gh api repos/a/b/pulls -f title=x",
    ] {
        let output = bash(command);
        assert_eq!(output.status.code(), Some(0), "{command}");
        assert_eq!(
            decision(&output).as_deref(),
            Some("ask"),
            "{command} should ask"
        );
    }
}

#[test]
fn ordinary_commands_pass_without_a_prompt() {
    for command in [
        "git log -n 5",
        "git -C . log -n 3",
        "head -n 3 f",
        "cargo xtask ci",
        "ls *",
        "ls -f",
        "git status --porcelain",
        "gh api repos/a/b/pulls",
        "git fetch origin",
    ] {
        let output = bash(command);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(decision(&output), None, "{command} should not prompt");
    }
}

#[test]
fn edits_to_non_rust_files_are_ignored() {
    let output = hook(
        "after-edit.sh",
        &serde_json::json!({ "tool_input": { "file_path": "README.md" } }),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
}

#[test]
fn a_second_stop_in_the_same_turn_is_let_through() {
    let output = hook(
        "stop-checks.sh",
        &serde_json::json!({ "stop_hook_active": true }),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
}

#[test]
fn without_jq_the_edit_and_stop_hooks_warn_without_blocking() {
    let stop = hook_with_path(
        "stop-checks.sh",
        &serde_json::json!({ "stop_hook_active": false }),
        Some("/nonexistent"),
    );
    assert_eq!(stop.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&stop.stderr).contains("jq"));
    let edit = hook_with_path(
        "after-edit.sh",
        &serde_json::json!({ "tool_input": { "file_path": "a.rs" } }),
        Some("/nonexistent"),
    );
    assert_eq!(edit.status.code(), Some(1));
}

#[test]
fn without_jq_the_bash_guard_blocks() {
    let output = hook_with_path(
        "guard-bash.sh",
        &serde_json::json!({ "tool_input": { "command": "ls" } }),
        Some("/nonexistent"),
    );
    assert_eq!(output.status.code(), Some(2));
}
