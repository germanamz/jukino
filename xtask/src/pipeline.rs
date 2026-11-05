//! What `cargo xtask ci`, `coverage` and `mutants` run, as data.

/// One external command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// Short name shown in output.
    pub name: &'static str,
    /// The program to run.
    pub program: &'static str,
    /// Its arguments.
    pub args: &'static [&'static str],
    /// Extra environment variables.
    pub env: &'static [(&'static str, &'static str)],
}

const fn cargo(name: &'static str, args: &'static [&'static str]) -> Step {
    Step {
        name,
        program: "cargo",
        args,
        env: &[],
    }
}

/// The local CI pipeline, in order. The same checks run in GitHub Actions.
///
/// `cargo-machete` runs as its own binary: invoked as `cargo machete` with the
/// `CARGO_PKG_*` variables that `cargo run` passes down, it reads `machete` as a path.
pub const CI_STEPS: [Step; 11] = [
    cargo("fmt", &["fmt", "--all", "--check"]),
    cargo(
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ),
    Step {
        name: "doc",
        program: "cargo",
        args: &["doc", "--workspace", "--no-deps"],
        env: &[("RUSTDOCFLAGS", "-D warnings")],
    },
    cargo("comments", &["xtask", "comments"]),
    cargo("arch", &["xtask", "arch"]),
    cargo("guard", &["xtask", "guard"]),
    cargo("nextest", &["nextest", "run", "--workspace"]),
    cargo("doctest", &["test", "--workspace", "--doc"]),
    cargo("coverage", &["xtask", "coverage"]),
    cargo("deny", &["deny", "check"]),
    Step {
        name: "machete",
        program: "cargo-machete",
        args: &[],
        env: &[],
    },
];

/// Where the coverage summary is written.
pub const COVERAGE_REPORT: &str = "target/coverage-summary.json";

/// Runs the tests under coverage and writes the JSON summary to [`COVERAGE_REPORT`].
/// `xtask/src/runner.rs` only launches tools and is excluded.
pub const COVERAGE: Step = cargo(
    "coverage",
    &[
        "llvm-cov",
        "nextest",
        "--workspace",
        "--json",
        "--summary-only",
        "--ignore-filename-regex",
        r"xtask[/\\]src[/\\]runner\.rs",
        "--output-path",
        COVERAGE_REPORT,
    ],
);

/// Where the diff to mutate is written.
pub const MUTANTS_DIFF: &str = "target/mutants.diff";

/// Mutation-tests the code in [`MUTANTS_DIFF`].
pub const MUTANTS: Step = cargo(
    "mutants",
    &["mutants", "--workspace", "--in-diff", MUTANTS_DIFF],
);

/// The tools the pipeline needs, with how to install each.
pub const REQUIRED_TOOLS: [(&str, &str); 6] = [
    ("cargo-nextest", "cargo install --locked cargo-nextest"),
    ("cargo-llvm-cov", "cargo install --locked cargo-llvm-cov"),
    ("cargo-deny", "cargo install --locked cargo-deny"),
    ("cargo-machete", "cargo install --locked cargo-machete"),
    ("cargo-mutants", "cargo install --locked cargo-mutants"),
    ("committed", "cargo install --locked committed"),
];

/// One message per required tool that `is_on_path` does not find.
pub fn missing_tools(is_on_path: impl Fn(&str) -> bool) -> Vec<String> {
    REQUIRED_TOOLS
        .iter()
        .filter(|(tool, _)| !is_on_path(tool))
        .map(|(tool, install)| format!("{tool} is not installed: run `{install}`"))
        .collect()
}

/// Runs `steps` in order with `execute`, stopping at the first that fails.
///
/// # Errors
///
/// Returns the name of the step that failed.
pub fn run_steps(
    steps: &[Step],
    mut execute: impl FnMut(&Step) -> bool,
) -> Result<(), &'static str> {
    steps.iter().try_for_each(|step| {
        if execute(step) {
            Ok(())
        } else {
            Err(step.name)
        }
    })
}
