//! Behaviour of `cargo xtask guard`: changes to existing tests and to gates.
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use std::path::Path;
use std::process::{Command, Output};

use xtask::finding::Finding;
use xtask::git::{ChangeStatus, ChangedFile};
use xtask::guard::protected::{added_expects, is_protected, lints_changed};
use xtask::guard::tests_diff::compare;
use xtask::guard::{Mode, Outcome, decide, file_findings};

const BASE: &str = "#[test]\nfn adds() {\n    assert_eq!(1 + 1, 2);\n}\n";

const SOURCE_FILE: &str = "crates/a/src/lib.rs";
const TEST_FILE: &str = "crates/a/tests/t.rs";

fn changes(base: &str, current: Option<&str>) -> Vec<(usize, String)> {
    compare(SOURCE_FILE, base, current).expect("valid Rust")
}

fn test_file_changes(base: &str, current: &str) -> Vec<(usize, String)> {
    compare(TEST_FILE, base, Some(current)).expect("valid Rust")
}

fn change(line: usize, message: &str) -> Vec<(usize, String)> {
    vec![(line, message.to_owned())]
}

#[test]
fn disabling_a_test_file_with_an_inner_attribute_is_reported() {
    let base = "#![cfg(unix)]\n#[test]\nfn a() {}\n";
    let current = "#![cfg(any())]\n#[test]\nfn a() {}\n";
    assert_eq!(
        test_file_changes(base, current),
        change(1, "test code `file attributes` changed")
    );
}

#[test]
fn changing_or_deleting_a_helper_in_a_test_file_is_reported() {
    let base = "fn helper() -> u8 {\n    1\n}\n#[test]\nfn a() {}\n";
    let changed = base.replace("    1", "    2");
    assert_eq!(
        test_file_changes(base, &changed),
        change(1, "test code `fn helper` changed")
    );
    let deleted = "#[test]\nfn a() {}\n";
    assert_eq!(
        test_file_changes(base, deleted),
        change(1, "test code `fn helper` deleted or renamed")
    );
}

#[test]
fn disabling_a_test_module_is_reported() {
    let base = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn a() {}\n}\n";
    let current = base.replace("cfg(test)", "cfg(any())");
    assert_eq!(
        changes(base, Some(&current)),
        change(2, "test code `mod tests` changed")
    );
}

#[test]
fn helpers_inside_a_test_module_are_test_code() {
    let base = "#[cfg(test)]\nmod tests {\n    fn h() -> u8 {\n        1\n    }\n}\n";
    let current = base.replace("        1", "        2");
    assert_eq!(
        changes(base, Some(&current)),
        change(3, "test code `fn tests::h` changed")
    );
}

#[test]
fn adding_to_test_code_is_not_reported() {
    let base = "fn helper() {}\n#[test]\nfn a() {}\n";
    let current = format!("{base}fn other() {{}}\n#[test]\nfn b() {{}}\n");
    assert_eq!(test_file_changes(base, &current), none());
    let module = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn a() {}\n}\n";
    let grown = module.replace("fn a() {}", "fn a() {}\n    #[test]\n    fn b() {}");
    assert_eq!(changes(module, Some(&grown)), none());
}

#[test]
fn helpers_in_ordinary_source_files_are_not_test_code() {
    let base = "fn helper() -> u8 {\n    1\n}\n";
    assert_eq!(changes(base, Some(&base.replace("    1", "    2"))), none());
}

fn none() -> Vec<(usize, String)> {
    Vec::new()
}

#[test]
fn adding_a_test_is_not_reported() {
    let current = format!("{BASE}#[test]\nfn subtracts() {{}}\n");
    assert_eq!(changes(BASE, Some(&current)), none());
}

#[test]
fn reformatting_a_test_is_not_reported() {
    let current = "#[test]\nfn adds() { assert_eq!(1+1, 2); }\n";
    assert_eq!(changes(BASE, Some(current)), none());
}

#[test]
fn deleting_or_renaming_a_test_is_reported() {
    let renamed = BASE.replace("fn adds", "fn sums");
    let found = changes(BASE, Some(&renamed));
    assert_eq!(found, [(2, "test adds deleted or renamed".to_owned())]);
}

#[test]
fn ignoring_a_test_is_reported() {
    let current = BASE.replace("#[test]\n", "#[test]\n#[ignore]\n");
    assert_eq!(
        changes(BASE, Some(&current)),
        [(3, "test adds gained #[ignore]".to_owned())]
    );
}

#[test]
fn should_panic_is_reported() {
    let current = BASE.replace("#[test]\n", "#[test]\n#[should_panic]\n");
    assert_eq!(
        changes(BASE, Some(&current)),
        [(3, "test adds gained #[should_panic]".to_owned())]
    );
}

#[test]
fn changing_a_test_body_is_reported() {
    let current = BASE.replace("1 + 1, 2", "1 + 1, 3");
    assert_eq!(
        changes(BASE, Some(&current)),
        [(2, "test adds changed".to_owned())]
    );
}

#[test]
fn tests_in_nested_modules_and_async_test_attributes_are_found() {
    let base = "mod a {\n    mod b {\n        #[test]\n        fn t() {}\n    }\n}\n#[tokio::test]\nasync fn u() {}\n";
    let found = changes(base, Some("mod a {}\n"));
    assert_eq!(
        found,
        [
            (4, "test a::b::t deleted or renamed".to_owned()),
            (8, "test u deleted or renamed".to_owned()),
        ]
    );
}

#[test]
fn deleting_a_file_with_tests_is_reported() {
    assert_eq!(changes(BASE, None), [(1, "test file deleted".to_owned())]);
    assert_eq!(changes("fn helper() {}\n", None), none());
}

const GATE_PATHS: [&str; 23] = [
    ".gitattributes",
    "clippy.toml",
    "crates/x/clippy.toml",
    "crates/x/rustfmt.toml",
    "deny.toml",
    "committed.toml",
    "arch.toml",
    "rustfmt.toml",
    "lefthook.yml",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    ".cargo/mutants.toml",
    ".config/nextest.toml",
    ".github/workflows/ci.yml",
    ".claude/settings.json",
    ".claude/hooks/a.sh",
    "xtask/src/lib.rs",
    "CONVENTIONS.md",
    "AGENTS.md",
    "CLAUDE.md",
    ".claude/rules/core.md",
    ".claude/skills/x/SKILL.md",
    "xtask\\src\\main.rs",
];

#[test]
fn gate_files_and_rule_texts_are_protected() {
    for path in GATE_PATHS {
        assert!(is_protected(path), "{path} should be protected");
    }
}

#[test]
fn product_files_are_not_protected() {
    for path in [
        "crates/jukino-core/src/lib.rs",
        "VISION.md",
        "tusk.toml",
        "docs/deny.toml",
    ] {
        assert!(!is_protected(path), "{path} should not be protected");
    }
}

#[test]
fn lints_table_changes_are_detected() {
    let base = "[workspace.lints.clippy]\nunwrap_used = \"deny\"\n\n[workspace.dependencies]\nclap = \"4\"\n";
    let loosened = base.replace("\"deny\"", "\"allow\"");
    let new_dependency = base.replace("clap = \"4\"", "clap = \"4\"\nsyn = \"3\"");
    assert!(lints_changed(Some(base), Some(&loosened)).expect("toml"));
    assert!(!lints_changed(Some(base), Some(&new_dependency)).expect("toml"));
}

#[test]
fn a_new_crate_inheriting_workspace_lints_is_not_a_lints_change() {
    let manifest = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
    assert!(!lints_changed(None, Some(manifest)).expect("toml"));
}

#[test]
fn crate_level_lint_overrides_and_opting_out_are_lints_changes() {
    let inherits = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
    let overrides = "[package]\nname = \"a\"\n\n[lints.clippy]\nunwrap_used = \"allow\"\n";
    let opted_out = "[package]\nname = \"a\"\n";
    assert!(lints_changed(None, Some(overrides)).expect("toml"));
    assert!(lints_changed(Some(inherits), Some(opted_out)).expect("toml"));
}

#[test]
fn new_expect_attributes_are_counted() {
    let base = "#[expect(clippy::a, reason = \"x\")]\nfn f() {}\n";
    let current = format!(
        "{base}#![expect(clippy::b, reason = \"y\")]\n#[expect(clippy::c, reason = \"z\")]\nfn g() {{}}\n"
    );
    assert_eq!(added_expects(Some(base), &current), 2);
    assert_eq!(added_expects(Some(&current), base), 0);
    assert_eq!(added_expects(None, base), 1);
}

fn finding(message: &str) -> Finding {
    Finding {
        path: "a.rs".into(),
        line: 1,
        message: message.to_owned(),
    }
}

#[test]
fn local_mode_only_warns() {
    assert_eq!(decide(&[finding("x")], &Mode::Local), Outcome::Warn);
}

#[test]
fn ci_mode_fails_without_the_label() {
    let mode = Mode::Ci {
        labels: vec!["docs".to_owned()],
    };
    assert_eq!(decide(&[finding("x")], &mode), Outcome::Fail);
}

#[test]
fn ci_mode_passes_with_guard_approved() {
    let mode = Mode::Ci {
        labels: vec!["docs".to_owned(), "guard-approved".to_owned()],
    };
    assert_eq!(decide(&[finding("x")], &mode), Outcome::Warn);
}

#[test]
fn no_findings_is_clean_in_both_modes() {
    let ci = Mode::Ci { labels: Vec::new() };
    assert_eq!(decide(&[], &Mode::Local), Outcome::Clean);
    assert_eq!(decide(&[], &ci), Outcome::Clean);
}

fn changed(path: &str, status: ChangeStatus) -> ChangedFile {
    ChangedFile {
        path: path.to_owned(),
        status,
    }
}

fn messages(file: &ChangedFile, base: Option<&str>, current: Option<&str>) -> Vec<String> {
    file_findings(file, base, current)
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn file_findings_cover_protected_files_tests_expects_and_lints() {
    let protected = changed("deny.toml", ChangeStatus::Added);
    assert_eq!(
        messages(&protected, None, Some("x")),
        ["protected file added"]
    );
    let test_file = changed("crates/a/tests/t.rs", ChangeStatus::Modified);
    let current = BASE.replace("1 + 1, 2", "1 + 1, 3");
    assert_eq!(
        messages(&test_file, Some(BASE), Some(&current)),
        ["test adds changed"]
    );
    let source = changed("crates/a/src/lib.rs", ChangeStatus::Added);
    let expects = "#[expect(clippy::a, reason = \"x\")]\nfn f() {}\n";
    assert_eq!(
        messages(&source, None, Some(expects)),
        ["1 new #[expect] attribute(s)"]
    );
    let manifest = changed("crates/a/Cargo.toml", ChangeStatus::Added);
    assert_eq!(
        messages(&manifest, None, Some("[lints.clippy]\na = \"allow\"\n")),
        ["lints table changed"]
    );
    let deleted = changed("crates/a/tests/t.rs", ChangeStatus::Deleted);
    assert_eq!(messages(&deleted, Some(BASE), None), ["test file deleted"]);
}

#[test]
fn unparsable_sources_are_reported_instead_of_skipped() {
    let file = changed("src/a.rs", ChangeStatus::Modified);
    let found = messages(&file, Some(BASE), Some("fn {"));
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("cannot compare tests"), "{found:?}");
}

#[test]
fn findings_carry_the_path_and_line() {
    let file = changed("crates/a/tests/t.rs", ChangeStatus::Modified);
    let current = BASE.replace("#[test]\n", "#[test]\n#[ignore]\n");
    let found = file_findings(&file, Some(BASE), Some(&current));
    assert_eq!(found[0].path, Path::new("crates/a/tests/t.rs"));
    assert_eq!(found[0].line, 3);
}

fn git(dir: &Path, args: &[&str]) -> Output {
    let output = Command::new("git")
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
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    git(dir.path(), &["init", "--quiet"]);
    std::fs::create_dir_all(dir.path().join("tests")).expect("mkdir");
    std::fs::write(dir.path().join("tests/t.rs"), BASE).expect("write");
    std::fs::write(dir.path().join("README.md"), "x\n").expect("write");
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "--no-verify", "-m", "initial"],
    );
    dir
}

#[test]
fn untracked_files_count_as_added() {
    let dir = repository();
    std::fs::write(dir.path().join("new.rs"), "fn f() {}\n").expect("write");
    std::fs::write(dir.path().join("README.md"), "y\n").expect("write");
    let mut files = xtask::git::changed_files(dir.path(), "HEAD").expect("diff");
    files.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(
        files,
        [
            changed("README.md", ChangeStatus::Modified),
            changed("new.rs", ChangeStatus::Added)
        ]
    );
}

#[test]
fn staged_new_files_and_deleted_files_have_their_status() {
    let dir = repository();
    std::fs::write(dir.path().join("staged.rs"), "fn f() {}\n").expect("write");
    git(dir.path(), &["add", "staged.rs"]);
    std::fs::remove_file(dir.path().join("README.md")).expect("delete");
    let mut files = xtask::git::changed_files(dir.path(), "HEAD").expect("diff");
    files.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(
        files,
        [
            changed("README.md", ChangeStatus::Deleted),
            changed("staged.rs", ChangeStatus::Added)
        ]
    );
}

#[test]
fn an_unknown_base_ref_is_an_error() {
    let dir = repository();
    let error = xtask::git::merge_base(dir.path(), "origin/nope").expect_err("unknown ref");
    assert!(error.to_string().contains("origin/nope"), "{error}");
}

#[test]
fn a_file_at_a_revision_is_read_and_a_missing_one_is_none() {
    let dir = repository();
    assert_eq!(
        xtask::git::file_at(dir.path(), "HEAD", "tests/t.rs").expect("git show"),
        Some(BASE.to_owned())
    );
    assert_eq!(
        xtask::git::file_at(dir.path(), "HEAD", "nope.rs").expect("git show"),
        None
    );
}

#[test]
fn the_repository_check_sees_working_tree_changes() {
    let dir = repository();
    std::fs::write(
        dir.path().join("tests/t.rs"),
        BASE.replace("1 + 1, 2", "1 + 1, 3"),
    )
    .expect("write");
    std::fs::write(dir.path().join("deny.toml"), "x = 1\n").expect("write");
    let mut found: Vec<String> = xtask::guard::check(dir.path(), "HEAD")
        .expect("guard runs")
        .into_iter()
        .map(|finding| finding.to_string())
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            "deny.toml:1: protected file added",
            "tests/t.rs:2: test adds changed"
        ]
    );
}

fn xtask_guard(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir)
        .arg("guard")
        .args(args)
        .output()
        .expect("xtask runs")
}

#[test]
fn the_command_warns_locally_and_fails_in_ci_without_the_label() {
    let dir = repository();
    std::fs::write(dir.path().join("deny.toml"), "x = 1\n").expect("write");
    let local = xtask_guard(dir.path(), &["--base", "HEAD"]);
    assert_eq!(local.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&local.stdout).contains("stop and report"));
    let ci = xtask_guard(dir.path(), &["--base", "HEAD", "--ci", "--labels", "docs"]);
    assert_eq!(ci.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&ci.stdout).contains("guard-approved"));
    let approved = xtask_guard(
        dir.path(),
        &["--base", "HEAD", "--ci", "--labels", "docs,guard-approved"],
    );
    assert_eq!(approved.status.code(), Some(0));
}

#[test]
fn the_command_is_silent_and_succeeds_without_findings() {
    let dir = repository();
    let output = xtask_guard(dir.path(), &["--base", "HEAD", "--ci"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
}

#[test]
fn the_command_exits_2_when_the_base_cannot_be_resolved() {
    let dir = repository();
    let output = xtask_guard(dir.path(), &["--base", "origin/nope"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("origin/nope"));
}

#[test]
fn the_mutation_diff_includes_tracked_changes_and_untracked_files() {
    let dir = repository();
    std::fs::write(dir.path().join("README.md"), "changed\n").expect("write");
    std::fs::write(dir.path().join("new.rs"), "fn f() {}\n").expect("write");
    let diff = xtask::git::diff_including_untracked(dir.path(), "HEAD").expect("diff");
    assert!(diff.contains("+changed"), "{diff}");
    assert!(
        diff.contains("+++ b/new.rs") && diff.contains("+fn f() {}"),
        "{diff}"
    );
}

#[test]
fn an_unchanged_tree_has_an_empty_mutation_diff() {
    let dir = repository();
    assert_eq!(
        xtask::git::diff_including_untracked(dir.path(), "HEAD").expect("diff"),
        ""
    );
}

#[test]
fn the_mutation_diff_names_a_missing_base() {
    let dir = repository();
    let error = xtask::git::mutation_diff(dir.path(), "origin/nope").expect_err("unknown ref");
    assert!(error.to_string().contains("origin/nope"), "{error}");
}

#[test]
fn the_mutation_diff_is_none_without_changes() {
    let dir = repository();
    assert_eq!(
        xtask::git::mutation_diff(dir.path(), "HEAD").expect("diff"),
        None
    );
}

#[test]
fn the_mutation_diff_ignores_user_diff_settings() {
    let dir = repository();
    git(dir.path(), &["config", "diff.noprefix", "true"]);
    git(dir.path(), &["config", "color.ui", "always"]);
    std::fs::write(dir.path().join("README.md"), "changed\n").expect("write");
    std::fs::write(dir.path().join("new.rs"), "fn f() {}\n").expect("write");
    let diff = xtask::git::mutation_diff(dir.path(), "HEAD")
        .expect("diff")
        .expect("changes");
    assert!(
        diff.contains("+++ b/README.md") && diff.contains("+++ b/new.rs"),
        "{diff}"
    );
    assert!(!diff.contains('\u{1b}'), "{diff}");
}

#[test]
fn adding_a_disabling_attribute_to_a_test_file_is_reported() {
    let base = "#[test]\nfn a() {}\n";
    let current = format!("#![cfg(any())]\n{base}");
    assert_eq!(
        test_file_changes(base, &current),
        change(1, "test code `file attributes` changed")
    );
}

#[test]
fn every_kind_of_named_item_in_a_test_file_is_test_code() {
    let base = "const LIMIT: u8 = 1;\nstatic NAME: &str = \"a\";\nstruct Fixture {\n    a: u8,\n}\nenum Shape {\n    A,\n}\ntype Alias = u8;\ntrait Probe {}\nmacro_rules! check {\n    () => {};\n}\n";
    for (old, new, key) in [
        ("LIMIT: u8 = 1", "LIMIT: u8 = 2", "const LIMIT"),
        ("\"a\"", "\"b\"", "static NAME"),
        ("    a: u8,", "    a: u16,", "struct Fixture"),
        ("    A,", "    B,", "enum Shape"),
        ("Alias = u8", "Alias = u16", "type Alias"),
        ("trait Probe {}", "trait Probe: Sized {}", "trait Probe"),
        ("() => {};", "($x:expr) => {};", "macro check"),
    ] {
        let found = test_file_changes(base, &base.replace(old, new));
        assert_eq!(found.len(), 1, "{key}: {found:?}");
        assert_eq!(found[0].1, format!("test code `{key}` changed"));
    }
}

#[test]
fn ordinary_modules_are_not_test_code() {
    let base = "#[cfg(unix)]\nmod platform {\n    fn a() -> u8 {\n        1\n    }\n}\n";
    let current = base
        .replace("#[cfg(unix)]\n", "#[cfg(unix)]\n#[doc = \"x\"]\n")
        .replace("        1", "        2");
    assert_eq!(changes(base, Some(&current)), none());
}
