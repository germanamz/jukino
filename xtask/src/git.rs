//! Git queries the checks need.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Lists the `.rs` files under `root` that git tracks or would track: committed,
/// staged and untracked files, but not ignored ones. Paths are relative to `root`.
///
/// # Errors
///
/// Fails when git cannot be started or reports an error, for example because `root`
/// is not inside a repository.
pub fn rust_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.rs",
        ])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| PathBuf::from(String::from_utf8_lossy(name).into_owned()))
        .collect())
}

/// How a file differs between the base revision and the working tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeStatus {
    /// New since the base, tracked or not.
    Added,
    /// Present in both, with different content or type.
    Modified,
    /// Present at the base, gone now.
    Deleted,
}

impl fmt::Display for ChangeStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
        })
    }
}

/// A file that differs from the base revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    /// Path relative to the repository root, with `/` separators.
    pub path: String,
    /// How it changed.
    pub status: ChangeStatus,
}

/// A git command that failed, with what git said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitError(pub String);

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for GitError {}

fn run(root: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| GitError(format!("cannot run git: {error}")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(GitError(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

/// Finds the commit where `HEAD` and `base_ref` diverged.
///
/// # Errors
///
/// Fails when `base_ref` does not resolve, for example in a shallow clone without
/// `origin/main`; the message names the ref.
pub fn merge_base(root: &Path, base_ref: &str) -> Result<String, GitError> {
    run(root, &["merge-base", "HEAD", base_ref])
        .map(|commit| commit.trim().to_owned())
        .map_err(|error| {
            GitError(format!(
                "cannot find the merge base of HEAD and `{base_ref}`: {error}"
            ))
        })
}

/// Lists the files that differ between `base` and the working tree, including
/// untracked files that are not ignored.
///
/// # Errors
///
/// Fails when git fails.
pub fn changed_files(root: &Path, base: &str) -> Result<Vec<ChangedFile>, GitError> {
    let diff = run(
        root,
        &["diff", "--name-status", "--no-renames", "-z", base, "--"],
    )?;
    let mut fields = diff.split('\0').filter(|field| !field.is_empty());
    let mut files = Vec::new();
    while let (Some(status), Some(path)) = (fields.next(), fields.next()) {
        let status = match status {
            "A" => ChangeStatus::Added,
            "D" => ChangeStatus::Deleted,
            _ => ChangeStatus::Modified,
        };
        files.push(ChangedFile {
            path: path.to_owned(),
            status,
        });
    }
    let untracked = run(root, &["ls-files", "-z", "--others", "--exclude-standard"])?;
    files.extend(
        untracked
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(|path| ChangedFile {
                path: path.to_owned(),
                status: ChangeStatus::Added,
            }),
    );
    Ok(files)
}

/// Reads `path` as it was at `rev`, or `None` when it did not exist there or is not
/// UTF-8 text.
///
/// # Errors
///
/// Fails only when git cannot be started.
pub fn file_at(root: &Path, rev: &str, path: &str) -> Result<Option<String>, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!("{rev}:{path}")])
        .output()
        .map_err(|error| GitError(format!("cannot run git: {error}")))?;
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten())
}

/// Diff options that keep the output parseable whatever the user's git config says
/// (`diff.noprefix`, `color.ui`, external diff tools).
const DIFF_FLAGS: [&str; 4] = [
    "--no-color",
    "--no-ext-diff",
    "--src-prefix=a/",
    "--dst-prefix=b/",
];

/// The diff to mutation-test: everything changed since the merge base of `HEAD` and
/// `base_ref`, including untracked files, or `None` when nothing changed.
///
/// # Errors
///
/// Fails when `base_ref` cannot be resolved (the message names it) or git fails.
pub fn mutation_diff(root: &Path, base_ref: &str) -> Result<Option<String>, GitError> {
    let base = merge_base(root, base_ref)?;
    let diff = diff_including_untracked(root, &base)?;
    Ok((!diff.trim().is_empty()).then_some(diff))
}

/// The unified diff from `base` to the working tree, followed by a diff adding
/// each untracked, non-ignored file, so new files are mutated too.
///
/// # Errors
///
/// Fails when git fails.
pub fn diff_including_untracked(root: &Path, base: &str) -> Result<String, GitError> {
    let mut diff = run(root, &[&["diff"][..], &DIFF_FLAGS, &[base, "--"]].concat())?;
    let untracked = run(root, &["ls-files", "-z", "--others", "--exclude-standard"])?;
    for path in untracked.split('\0').filter(|path| !path.is_empty()) {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("diff")
            .args(DIFF_FLAGS)
            .args(["--no-index", "--", "/dev/null", path])
            .output()
            .map_err(|error| GitError(format!("cannot run git: {error}")))?;
        diff.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    Ok(diff)
}

/// The hash and full message of every commit in `range`, newest first.
///
/// # Errors
///
/// Fails when the range does not resolve or git fails.
pub fn commit_messages(root: &Path, range: &str) -> Result<Vec<(String, String)>, GitError> {
    let log = run(root, &["log", "--format=%H%x00%B%x1e", range, "--"])?;
    Ok(log
        .split('\u{1e}')
        .filter_map(|record| record.trim_start_matches('\n').split_once('\0'))
        .map(|(hash, message)| (hash.to_owned(), message.to_owned()))
        .collect())
}
