//! Behaviour of `cargo xtask comments`, the no-blob-comments check.
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use std::path::Path;
use std::process::Command;

use xtask::comments::check_source;
use xtask::finding::Finding;

fn findings(source: &str) -> Vec<(usize, String)> {
    check_source(Path::new("src/a.rs"), source)
        .into_iter()
        .map(|finding| (finding.line, finding.message))
        .collect()
}

fn none() -> Vec<(usize, String)> {
    Vec::new()
}

#[test]
fn a_single_why_comment_is_allowed() {
    let source = "fn f() {\n    // Retry once: the server drops the first request.\n    g();\n}\n";
    assert_eq!(findings(source), none());
}

#[test]
fn two_consecutive_comment_lines_are_a_blob() {
    let source = "fn f() {\n    g();\n    // first line\n    // second line\n    h();\n}\n";
    let found = findings(source);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, 3);
    assert!(found[0].1.contains("comment block"));
}

#[test]
fn blob_findings_report_the_block_length() {
    let found = findings("// one\n// two\n// three\nfn f() {}\n");
    assert_eq!(found.len(), 1);
    assert!(found[0].1.contains("3 lines"), "{}", found[0].1);
}

#[test]
fn each_blob_reports_its_own_line_and_length() {
    let source = "// a\n// b\nfn f() {}\n// c\n// d\n// e\n";
    let found = findings(source);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0].0, 1);
    assert!(found[0].1.contains("2 lines"), "{}", found[0].1);
    assert_eq!(found[1].0, 4);
    assert!(found[1].1.contains("3 lines"), "{}", found[1].1);
}

#[test]
fn trailing_comments_on_code_lines_are_not_a_blob() {
    assert_eq!(findings("let a = 1; // one\nlet b = 2; // two\n"), none());
}

#[test]
fn doc_comments_are_never_flagged() {
    let source =
        "//! Crate docs.\n//! More.\n/// Item docs.\n/// More.\n/// Even more.\nfn f() {}\n";
    assert_eq!(findings(source), none());
}

#[test]
fn block_comments_are_rejected() {
    for source in ["/* x */\n", "/** x */\nfn f() {}\n", "/*! x */\n"] {
        let found = findings(source);
        assert_eq!(found.len(), 1, "{source:?}");
        assert!(found[0].1.contains("block comment"));
    }
}

#[test]
fn nested_block_comment_is_one_finding() {
    assert_eq!(findings("/* a /* b */ c */\n").len(), 1);
}

#[test]
fn lines_after_a_multiline_token_are_counted() {
    let source = "/* a\nb */\nlet s = \"x\ny\";\n// one\n// two\n";
    let lines: Vec<usize> = findings(source).into_iter().map(|(line, _)| line).collect();
    assert_eq!(lines, [1, 5]);
}

#[test]
fn commented_out_code_is_rejected() {
    for source in ["// let x = 1;\n", "// fn f() {\n", "// }\n"] {
        let found = findings(source);
        assert_eq!(found.len(), 1, "{source:?}");
        assert!(found[0].1.contains("commented-out code"));
    }
}

#[test]
fn comment_markers_inside_literals_are_ignored() {
    let source =
        "let a = \"http://x\";\nlet b = r#\"// a\n// b\"#;\nlet c = b\"/* */\";\nlet d = '/';\n";
    assert_eq!(findings(source), none());
}

#[test]
fn crlf_sources_are_checked_like_lf() {
    let blob = "fn f() {\n    g();\n    // first line\n    // second line\n}\n";
    let code = "x();\n// let y = 2;\n";
    for source in [blob, code] {
        let crlf = source.replace('\n', "\r\n");
        assert_ne!(findings(source), none());
        assert_eq!(findings(&crlf), findings(source));
    }
}

#[test]
fn findings_print_as_clickable_locations() {
    let finding = Finding {
        path: "src/a.rs".into(),
        line: 3,
        message: "comment block".to_owned(),
    };
    assert_eq!(finding.to_string(), "src/a.rs:3: comment block");
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

#[test]
fn rust_files_include_untracked_but_not_ignored_files() {
    let dir = tempfile::tempdir().expect("temp dir");
    git(dir.path(), &["init", "--quiet"]);
    for name in ["tracked.rs", "new.rs", "ignored.rs", "notes.md"] {
        std::fs::write(dir.path().join(name), "fn f() {}\n").expect("write");
    }
    std::fs::write(dir.path().join(".gitignore"), "ignored.rs\n").expect("write");
    git(dir.path(), &["add", "tracked.rs"]);
    let mut files = xtask::git::rust_files(dir.path()).expect("list files");
    files.sort();
    assert_eq!(files, [Path::new("new.rs"), Path::new("tracked.rs")]);
}

fn xtask(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("xtask runs")
}

#[test]
fn the_command_reports_findings_and_exits_1() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("a.rs"), "// one\n// two\n").expect("write");
    std::fs::write(dir.path().join("b.rs"), "fn f() {}\n").expect("write");
    let output = xtask(dir.path(), &["comments", "a.rs", "b.rs"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("a.rs:1: comment block"));
}

#[test]
fn the_command_passes_clean_files_and_skips_missing_ones() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("b.rs"), "fn f() {}\n").expect("write");
    let output = xtask(dir.path(), &["comments", "b.rs", "gone.rs"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn without_paths_the_command_checks_every_rust_file_in_the_repository() {
    let dir = tempfile::tempdir().expect("temp dir");
    git(dir.path(), &["init", "--quiet"]);
    std::fs::write(dir.path().join("a.rs"), "/* nope */\n").expect("write");
    let output = xtask(dir.path(), &["comments"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("a.rs:1: "));
}
