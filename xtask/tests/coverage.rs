//! Behaviour of the coverage thresholds.
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use xtask::coverage::{THRESHOLDS, Thresholds, evaluate};

const CORE: &str = "crates/jukino-core/";

/// One file: (path, lines, covered lines, functions, covered functions).
type FileFigures = (&'static str, u64, u64, u64, u64);

fn report(files: &[FileFigures]) -> String {
    let summary = |lines: u64, covered: u64, functions: u64, called: u64| {
        format!(
            r#"{{"lines":{{"count":{lines},"covered":{covered}}},"functions":{{"count":{functions},"covered":{called}}}}}"#
        )
    };
    let entries: Vec<String> = files
        .iter()
        .map(|(path, lines, covered, functions, called)| {
            format!(
                r#"{{"filename":"/repo/{path}","summary":{}}}"#,
                summary(*lines, *covered, *functions, *called)
            )
        })
        .collect();
    let total = |pick: fn(&FileFigures) -> u64| files.iter().map(pick).sum::<u64>();
    let totals = summary(
        total(|f| f.1),
        total(|f| f.2),
        total(|f| f.3),
        total(|f| f.4),
    );
    format!(
        r#"{{"data":[{{"files":[{}],"totals":{totals}}}],"type":"llvm.coverage.json.export"}}"#,
        entries.join(",")
    )
}

fn violations(files: &[FileFigures]) -> Vec<String> {
    evaluate(&report(files), CORE, &THRESHOLDS).expect("valid report")
}

#[test]
fn the_thresholds_are_the_conventions_numbers() {
    let Thresholds {
        workspace_lines,
        workspace_functions,
        core_lines,
    } = THRESHOLDS;
    assert_eq!(
        (workspace_lines, workspace_functions, core_lines),
        (90.0, 90.0, 95.0)
    );
}

#[test]
fn coverage_passes_at_the_thresholds() {
    let files = [
        ("crates/jukino-core/src/lib.rs", 20, 19, 4, 4),
        ("xtask/src/a.rs", 80, 71, 6, 5),
    ];
    assert_eq!(violations(&files), Vec::<String>::new());
}

#[test]
fn workspace_line_coverage_can_fail() {
    let files = [
        ("crates/jukino-core/src/lib.rs", 20, 20, 5, 5),
        ("xtask/src/a.rs", 980, 879, 5, 5),
    ];
    let found = violations(&files);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("line coverage 89.9%") && found[0].contains("90%"),
        "{found:?}"
    );
}

#[test]
fn workspace_function_coverage_can_fail() {
    let files = [
        ("crates/jukino-core/src/lib.rs", 10, 10, 500, 500),
        ("xtask/src/a.rs", 10, 10, 500, 399),
    ];
    let found = violations(&files);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("function coverage 89.9%"), "{found:?}");
}

#[test]
fn core_line_coverage_can_fail() {
    let files = [
        ("crates/jukino-core/src/lib.rs", 1000, 949, 5, 5),
        ("xtask/src/a.rs", 1000, 1000, 5, 5),
    ];
    let found = violations(&files);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("jukino-core line coverage 94.9%") && found[0].contains("95%"),
        "{found:?}"
    );
}

#[test]
fn core_figure_counts_only_core_files() {
    let files = [
        ("crates/jukino-core/src/lib.rs", 100, 100, 5, 5),
        ("xtask/src/a.rs", 100, 85, 5, 5),
    ];
    assert_eq!(violations(&files), Vec::<String>::new());
}

#[test]
fn missing_core_data_is_a_violation() {
    let found = violations(&[("xtask/src/a.rs", 100, 100, 5, 5)]);
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("no coverage data") && found[0].contains(CORE));
}

#[test]
fn a_malformed_report_is_an_error() {
    assert!(evaluate("{}", CORE, &THRESHOLDS).is_err());
}

#[test]
fn a_report_without_data_is_a_violation() {
    let found = evaluate(
        r#"{"data":[],"type":"llvm.coverage.json.export"}"#,
        CORE,
        &THRESHOLDS,
    )
    .expect("valid report");
    assert_eq!(found, ["coverage report holds no data"]);
}

#[test]
fn zero_workspace_counts_are_violations() {
    let found = violations(&[]);
    assert_eq!(
        found,
        [
            "workspace line coverage has no data",
            "workspace function coverage has no data",
            "no coverage data for crates/jukino-core/",
        ]
    );
}
