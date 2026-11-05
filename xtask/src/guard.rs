//! The guard: changes that a contributor must approve.
//!
//! It reports edits to existing tests, changes to gate configuration, gate code and
//! rule text, lint-table changes and new `#[expect]` suppressions. Locally it only
//! warns, and the agent stops and reports; in CI it fails until a contributor adds
//! the [`APPROVAL_LABEL`] to the pull request.

pub mod protected;
pub mod tests_diff;

use std::path::{Path, PathBuf};

use crate::finding::Finding;
use crate::git::{self, ChangeStatus, ChangedFile, GitError};

/// The pull-request label that approves guarded changes in CI.
pub const APPROVAL_LABEL: &str = "guard-approved";

/// Where the guard runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// On a contributor's machine: findings never fail.
    Local,
    /// In CI, with the pull request's labels.
    Ci {
        /// The pull request's labels.
        labels: Vec<String>,
    },
}

/// The guard's verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing to approve.
    Clean,
    /// Findings to report, but nothing fails.
    Warn,
    /// Findings without approval in CI.
    Fail,
}

/// Decides the verdict for `findings` in `mode`.
#[must_use]
pub fn decide(findings: &[Finding], mode: &Mode) -> Outcome {
    if findings.is_empty() {
        return Outcome::Clean;
    }
    match mode {
        Mode::Local => Outcome::Warn,
        Mode::Ci { labels } if labels.iter().any(|label| label == APPROVAL_LABEL) => Outcome::Warn,
        Mode::Ci { .. } => Outcome::Fail,
    }
}

/// Runs the guard on the working tree at `root` against the merge base of `HEAD`
/// and `base_ref`.
///
/// # Errors
///
/// Fails when `base_ref` cannot be resolved or git fails.
pub fn check(root: &Path, base_ref: &str) -> Result<Vec<Finding>, GitError> {
    let base = git::merge_base(root, base_ref)?;
    let mut findings = Vec::new();
    for file in git::changed_files(root, &base)? {
        let current = (file.status != ChangeStatus::Deleted)
            .then(|| std::fs::read_to_string(root.join(&file.path)).ok())
            .flatten();
        let before = match file.status {
            ChangeStatus::Added => None,
            ChangeStatus::Modified | ChangeStatus::Deleted => {
                git::file_at(root, &base, &file.path)?
            }
        };
        findings.extend(file_findings(&file, before.as_deref(), current.as_deref()));
    }
    Ok(findings)
}

/// The findings for one changed file, given its content at the base and now.
#[must_use]
pub fn file_findings(
    file: &ChangedFile,
    base: Option<&str>,
    current: Option<&str>,
) -> Vec<Finding> {
    let mut messages = Vec::new();
    if protected::is_protected(&file.path) {
        messages.push((1, format!("protected file {}", file.status)));
    }
    if file.path.ends_with("Cargo.toml") {
        match protected::lints_changed(base, current) {
            Ok(false) => {}
            Ok(true) => messages.push((1, "lints table changed".to_owned())),
            Err(error) => messages.push((1, format!("cannot compare lints: {error}"))),
        }
    }
    let is_rust = Path::new(&file.path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"));
    if is_rust {
        messages.extend(rust_messages(&file.path, base, current));
    }
    messages
        .into_iter()
        .map(|(line, message)| Finding {
            path: PathBuf::from(&file.path),
            line,
            message,
        })
        .collect()
}

fn rust_messages(path: &str, base: Option<&str>, current: Option<&str>) -> Vec<(usize, String)> {
    let mut messages = base.map_or_else(Vec::new, |base| {
        tests_diff::compare(path, base, current)
            .unwrap_or_else(|error| vec![(1, format!("cannot compare tests: {error}"))])
    });
    let added = current.map_or(0, |current| protected::added_expects(base, current));
    if added > 0 {
        messages.push((1, format!("{added} new #[expect] attribute(s)")));
    }
    messages
}
