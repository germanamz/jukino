//! Which changes loosen or touch a gate.

use toml::{Table, Value};

/// File names protected wherever they appear: a nested one overrides the root one.
const PROTECTED_NAMES: [&str; 4] = [
    "clippy.toml",
    ".clippy.toml",
    "rustfmt.toml",
    ".rustfmt.toml",
];

/// Files protected at these exact paths.
const PROTECTED_FILES: [&str; 11] = [
    ".gitattributes",
    "deny.toml",
    "committed.toml",
    "arch.toml",
    "lefthook.yml",
    "rust-toolchain.toml",
    ".config/nextest.toml",
    ".claude/settings.json",
    "CONVENTIONS.md",
    "AGENTS.md",
    "CLAUDE.md",
];

/// Directories whose every file is protected.
const PROTECTED_DIRECTORIES: [&str; 6] = [
    ".cargo/",
    ".github/",
    ".claude/hooks/",
    ".claude/rules/",
    ".claude/skills/",
    "xtask/",
];

/// Whether `path` (relative to the repository root) holds gate configuration, gate
/// code or rule text.
#[must_use]
pub fn is_protected(path: &str) -> bool {
    let path = path.replace('\\', "/");
    let name = path.rsplit('/').next().unwrap_or_default();
    PROTECTED_NAMES.contains(&name)
        || PROTECTED_FILES.contains(&path.as_str())
        || PROTECTED_DIRECTORIES
            .iter()
            .any(|directory| path.starts_with(directory))
}

/// Whether a `Cargo.toml` change touches lint configuration: any change to
/// `[workspace.lints]`, or a crate `[lints]` table that changed to anything other
/// than `workspace = true`.
///
/// # Errors
///
/// Fails when either version is not valid TOML.
pub fn lints_changed(base: Option<&str>, current: Option<&str>) -> Result<bool, toml::de::Error> {
    let (base, current) = (parse(base)?, parse(current)?);
    let workspace_lints = |table: &Table| {
        table
            .get("workspace")
            .and_then(|workspace| workspace.get("lints"))
            .cloned()
    };
    let crate_lints = |table: &Table| table.get("lints").cloned();
    let inherits = crate_lints(&current)
        .as_ref()
        .and_then(Value::as_table)
        .is_some_and(|lints| {
            lints.len() == 1 && lints.get("workspace").and_then(Value::as_bool) == Some(true)
        });
    Ok(workspace_lints(&base) != workspace_lints(&current)
        || (crate_lints(&base) != crate_lints(&current) && !inherits))
}

fn parse(text: Option<&str>) -> Result<Table, toml::de::Error> {
    text.map_or_else(|| Ok(Table::new()), toml::from_str)
}

/// How many more `#[expect(…)]` and `#![expect(…)]` attributes `current` has than
/// `base` (`None` for a new file).
#[must_use]
pub fn added_expects(base: Option<&str>, current: &str) -> usize {
    expects(current).saturating_sub(base.map_or(0, expects))
}

fn expects(text: &str) -> usize {
    text.matches("#[expect(").count() + text.matches("#![expect(").count()
}
