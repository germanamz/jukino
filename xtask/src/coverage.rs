//! Coverage thresholds, checked against `cargo llvm-cov`'s JSON summary.

use serde::Deserialize;

/// Minimum coverage, in percent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// Lines covered across the workspace.
    pub workspace_lines: f64,
    /// Functions covered across the workspace.
    pub workspace_functions: f64,
    /// Lines covered in the core crate.
    pub core_lines: f64,
}

/// The project's thresholds (see `CONVENTIONS.md`).
pub const THRESHOLDS: Thresholds = Thresholds {
    workspace_lines: 90.0,
    workspace_functions: 90.0,
    core_lines: 95.0,
};

#[derive(Deserialize)]
struct Report {
    data: Vec<Export>,
}

#[derive(Deserialize)]
struct Export {
    files: Vec<FileReport>,
    totals: Summary,
}

#[derive(Deserialize)]
struct FileReport {
    filename: String,
    summary: Summary,
}

#[derive(Deserialize)]
struct Summary {
    lines: Counts,
    functions: Counts,
}

#[derive(Deserialize, Clone, Copy, Default)]
struct Counts {
    count: u64,
    covered: u64,
}

impl Counts {
    #[expect(
        clippy::cast_precision_loss,
        reason = "coverage counts are far below 2^52"
    )]
    fn percent(self) -> f64 {
        self.covered as f64 * 100.0 / self.count as f64
    }
}

/// Checks a `cargo llvm-cov --json --summary-only` report against `thresholds` and
/// returns one message per threshold missed. The core figure sums the files whose
/// path contains `core_dir`; a report without such files is itself a violation.
///
/// # Errors
///
/// Fails when the report is not llvm-cov JSON.
pub fn evaluate(
    report_json: &str,
    core_dir: &str,
    thresholds: &Thresholds,
) -> Result<Vec<String>, serde_json::Error> {
    let report: Report = serde_json::from_str(report_json)?;
    if report.data.is_empty() {
        return Ok(vec!["coverage report holds no data".to_owned()]);
    }
    Ok(report
        .data
        .iter()
        .flat_map(|export| export_violations(export, core_dir, thresholds))
        .collect())
}

fn export_violations(export: &Export, core_dir: &str, thresholds: &Thresholds) -> Vec<String> {
    let mut violations = Vec::new();
    let totals = &export.totals;
    let functions = thresholds.workspace_functions;
    below(
        &mut violations,
        "workspace line coverage",
        totals.lines,
        thresholds.workspace_lines,
    );
    below(
        &mut violations,
        "workspace function coverage",
        totals.functions,
        functions,
    );
    let core = core_lines(&export.files, core_dir);
    if core.count == 0 {
        violations.push(format!("no coverage data for {core_dir}"));
    } else {
        below(
            &mut violations,
            "jukino-core line coverage",
            core,
            thresholds.core_lines,
        );
    }
    violations
}

fn core_lines(files: &[FileReport], core_dir: &str) -> Counts {
    files
        .iter()
        .filter(|file| file.filename.contains(core_dir))
        .fold(Counts::default(), |sum, file| Counts {
            count: sum.count + file.summary.lines.count,
            covered: sum.covered + file.summary.lines.covered,
        })
}

fn below(violations: &mut Vec<String>, what: &str, counts: Counts, threshold: f64) {
    if counts.count == 0 {
        violations.push(format!("{what} has no data"));
        return;
    }
    let percent = counts.percent();
    if percent < threshold {
        violations.push(format!("{what} {percent:.1}% is below {threshold}%"));
    }
}
