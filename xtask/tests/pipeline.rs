//! Behaviour of the `cargo xtask ci` pipeline description.

use xtask::pipeline::{CI_STEPS, REQUIRED_TOOLS, Step, missing_tools, run_steps};

#[test]
fn ci_steps_run_in_the_spec_order() {
    let names: Vec<&str> = CI_STEPS.iter().map(|step| step.name).collect();
    assert_eq!(
        names,
        [
            "fmt", "clippy", "doc", "comments", "arch", "guard", "nextest", "doctest", "coverage",
            "deny", "machete"
        ]
    );
}

fn step(name: &str) -> &'static Step {
    CI_STEPS
        .iter()
        .find(|step| step.name == name)
        .unwrap_or_else(|| panic!("no step {name}"))
}

#[test]
fn every_step_runs_through_cargo_with_the_spec_arguments() {
    let expected: [(&str, &[&str]); 10] = [
        ("fmt", &["fmt", "--all", "--check"]),
        (
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
        ("doc", &["doc", "--workspace", "--no-deps"]),
        ("comments", &["xtask", "comments"]),
        ("arch", &["xtask", "arch"]),
        ("guard", &["xtask", "guard"]),
        ("nextest", &["nextest", "run", "--workspace"]),
        ("doctest", &["test", "--workspace", "--doc"]),
        ("coverage", &["xtask", "coverage"]),
        ("deny", &["deny", "check"]),
    ];
    for (name, args) in expected {
        assert_eq!(step(name).program, "cargo", "{name}");
        assert_eq!(step(name).args, args, "{name}");
    }
}

#[test]
fn machete_runs_as_its_own_binary() {
    assert_eq!(step("machete").program, "cargo-machete");
    assert_eq!(step("machete").args, Vec::<&str>::new().as_slice());
}

#[test]
fn doc_step_denies_warnings() {
    assert_eq!(step("doc").env, [("RUSTDOCFLAGS", "-D warnings")]);
}

#[test]
fn missing_tools_name_their_install_command() {
    let missing = missing_tools(|_| false);
    assert_eq!(missing.len(), REQUIRED_TOOLS.len());
    for (message, (tool, _)) in missing.iter().zip(REQUIRED_TOOLS) {
        assert!(message.contains(tool), "{message}");
        assert!(
            message.contains(&format!("cargo install --locked {tool}")),
            "{message}"
        );
    }
    assert_eq!(missing_tools(|_| true), Vec::<String>::new());
}

#[test]
fn required_tools_cover_every_cargo_subcommand_the_pipeline_uses() {
    let tools: Vec<&str> = REQUIRED_TOOLS.iter().map(|(tool, _)| *tool).collect();
    for tool in [
        "cargo-nextest",
        "cargo-llvm-cov",
        "cargo-deny",
        "cargo-machete",
        "cargo-mutants",
        "committed",
    ] {
        assert!(tools.contains(&tool), "{tool}");
    }
}

#[test]
fn run_steps_stops_at_the_first_failure() {
    let mut ran = Vec::new();
    let result = run_steps(&CI_STEPS, |step| {
        ran.push(step.name);
        step.name != "arch"
    });
    assert_eq!(result, Err("arch"));
    assert_eq!(ran, ["fmt", "clippy", "doc", "comments", "arch"]);
}

#[test]
fn run_steps_succeeds_when_every_step_passes() {
    let mut count = 0;
    assert_eq!(
        run_steps(&CI_STEPS, |_| {
            count += 1;
            true
        }),
        Ok(())
    );
    assert_eq!(count, CI_STEPS.len());
}
