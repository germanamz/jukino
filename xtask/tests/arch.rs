//! Behaviour of `cargo xtask arch`, the dependency and boundary rules.
#![expect(
    clippy::expect_used,
    reason = "test helpers fail the test when setup fails"
)]

use std::path::{Path, PathBuf};

use xtask::arch::sources::check_source;
use xtask::arch::{
    ArchConfig, DependencyEdge, DependencyKind, PackageNode, WorkspaceGraph, check_graph,
    check_workspace, load_graph,
};

const CONFIG: &str = r#"
[core]
crate = "jukino-core"
allowed-dependencies = [{ name = "itoa", reason = "pure number formatting" }]

[composition-roots]
crates = ["jukino", "jukino-ui"]

[adapters]
directory = "crates/adapters"
couplings = [{ from = "jukino-outbox", to = "jukino-store", reason = "shares its transaction" }]

[tooling]
crates = ["xtask"]
"#;

fn config() -> ArchConfig {
    ArchConfig::parse(CONFIG).expect("valid config")
}

fn package(name: &str, dir: &str, dependencies: &[(&str, DependencyKind, bool)]) -> PackageNode {
    PackageNode {
        name: name.to_owned(),
        manifest_dir: PathBuf::from(dir),
        inherits_workspace_lints: true,
        dependencies: dependencies
            .iter()
            .map(|(dependency, kind, workspace_member)| DependencyEdge {
                name: (*dependency).to_owned(),
                kind: *kind,
                workspace_member: *workspace_member,
            })
            .collect(),
    }
}

fn base_packages() -> Vec<PackageNode> {
    vec![
        package("jukino-core", "crates/jukino-core", &[]),
        package(
            "jukino",
            "crates/jukino",
            &[("jukino-core", DependencyKind::Normal, true)],
        ),
        package("jukino-ui", "crates/jukino-ui", &[]),
        package("xtask", "xtask", &[]),
    ]
}

fn messages(extra: Vec<PackageNode>) -> Vec<String> {
    let mut packages = base_packages();
    packages.extend(extra);
    check_graph(&config(), &WorkspaceGraph { packages })
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn with_core(dependencies: &[(&str, DependencyKind, bool)]) -> Vec<String> {
    let mut packages = base_packages();
    packages[0] = package("jukino-core", "crates/jukino-core", dependencies);
    check_graph(&config(), &WorkspaceGraph { packages })
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn none() -> Vec<String> {
    Vec::new()
}

#[test]
fn the_real_workspace_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let text = std::fs::read_to_string(root.join("arch.toml")).expect("arch.toml");
    let config = ArchConfig::parse(&text).expect("valid arch.toml");
    let graph = load_graph(root).expect("cargo metadata");
    assert_eq!(check_graph(&config, &graph), Vec::new());
    assert_eq!(check_workspace(root).expect("check runs"), Vec::new());
}

#[test]
fn a_clean_graph_has_no_findings() {
    assert_eq!(messages(Vec::new()), none());
}

#[test]
fn an_unclassified_crate_fails() {
    let found = messages(vec![package("stray", "crates/stray", &[])]);
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("`stray`"), "{found:?}");
}

#[test]
fn a_missing_configured_crate_fails() {
    let mut packages = base_packages();
    packages.retain(|package| package.name != "jukino-ui");
    let found = check_graph(&config(), &WorkspaceGraph { packages });
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("`jukino-ui`"));
    assert_eq!(found[0].path, Path::new("arch.toml"));
}

#[test]
fn core_must_not_depend_on_workspace_crates() {
    let found = with_core(&[("jukino", DependencyKind::Normal, true)]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("core depends on workspace crate `jukino`"));
}

#[test]
fn core_external_dependency_needs_an_allow_list_entry() {
    for kind in [
        DependencyKind::Normal,
        DependencyKind::Dev,
        DependencyKind::Build,
    ] {
        let found = with_core(&[("serde", kind, false)]);
        assert_eq!(found.len(), 1, "{kind:?}");
        assert!(found[0].contains("`serde`") && found[0].contains("allow-list"));
    }
    assert_eq!(
        with_core(&[("itoa", DependencyKind::Normal, false)]),
        none()
    );
}

#[test]
fn allow_list_entries_need_a_reason() {
    let text = CONFIG.replace("pure number formatting", " ");
    let config = ArchConfig::parse(&text).expect("valid config");
    let found = check_graph(
        &config,
        &WorkspaceGraph {
            packages: base_packages(),
        },
    );
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("`itoa`") && found[0].message.contains("reason"));
}

#[test]
fn couplings_need_a_reason() {
    let text = CONFIG.replace("shares its transaction", "");
    let config = ArchConfig::parse(&text).expect("valid config");
    let found = check_graph(
        &config,
        &WorkspaceGraph {
            packages: base_packages(),
        },
    );
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("reason"));
}

#[test]
fn adapters_may_depend_on_the_core() {
    let adapter = package(
        "jukino-store",
        "crates/adapters/store",
        &[
            ("jukino-core", DependencyKind::Normal, true),
            ("serde", DependencyKind::Normal, false),
        ],
    );
    assert_eq!(messages(vec![adapter]), none());
}

#[test]
fn adapters_must_not_depend_on_each_other() {
    let store = package("jukino-store", "crates/adapters/store", &[]);
    let http = package(
        "jukino-http",
        "crates/adapters/http",
        &[("jukino-store", DependencyKind::Dev, true)],
    );
    let found = messages(vec![store, http]);
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("`jukino-http`") && found[0].contains("`jukino-store`"));
}

#[test]
fn a_declared_coupling_allows_one_adapter_to_use_another() {
    let store = package("jukino-store", "crates/adapters/store", &[]);
    let outbox = package(
        "jukino-outbox",
        "crates/adapters/outbox",
        &[("jukino-store", DependencyKind::Normal, true)],
    );
    assert_eq!(messages(vec![store, outbox]), none());
}

#[test]
fn only_composition_roots_depend_on_adapters() {
    let mut packages = base_packages();
    packages.push(package("jukino-store", "crates/adapters/store", &[]));
    packages[1].dependencies.push(DependencyEdge {
        name: "jukino-store".to_owned(),
        kind: DependencyKind::Normal,
        workspace_member: true,
    });
    packages[3].dependencies.push(DependencyEdge {
        name: "jukino-store".to_owned(),
        kind: DependencyKind::Normal,
        workspace_member: true,
    });
    let found = check_graph(&config(), &WorkspaceGraph { packages });
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].message.contains("`xtask`"));
}

#[test]
fn composition_roots_do_not_depend_on_each_other() {
    let mut packages = base_packages();
    packages[2].dependencies.push(DependencyEdge {
        name: "jukino".to_owned(),
        kind: DependencyKind::Normal,
        workspace_member: true,
    });
    let found = check_graph(&config(), &WorkspaceGraph { packages });
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("`jukino-ui`") && found[0].message.contains("`jukino`"));
}

#[test]
fn product_crates_do_not_depend_on_tooling() {
    let mut packages = base_packages();
    packages[1].dependencies.push(DependencyEdge {
        name: "xtask".to_owned(),
        kind: DependencyKind::Dev,
        workspace_member: true,
    });
    let found = check_graph(&config(), &WorkspaceGraph { packages });
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("`xtask`"));
}

#[test]
fn findings_point_at_the_offending_manifest() {
    let found = check_graph(
        &config(),
        &WorkspaceGraph {
            packages: vec![package("stray", "crates/stray", &[])],
        },
    );
    assert!(
        found
            .iter()
            .any(|finding| finding.path == Path::new("crates/stray/Cargo.toml"))
    );
}

fn source_lines(source: &str) -> Vec<usize> {
    check_source(Path::new("core.rs"), source)
        .expect("valid Rust")
        .into_iter()
        .map(|finding| finding.line)
        .collect()
}

#[test]
fn core_sources_must_not_name_io_modules() {
    let source = "use std::fs;\nuse std::{env, io::Read};\nfn f() {\n    std::process::exit(1);\n}\nfn g(_: ::std::net::TcpStream) {}\nfn h() { std::thread::yield_now(); }\n";
    assert_eq!(source_lines(source), [1, 2, 2, 4, 6, 7]);
}

#[test]
fn other_std_and_core_modules_are_allowed() {
    let source = "use std::fmt;\nuse core::str::FromStr;\nfn f() -> std::collections::BTreeMap<u8, u8> { std::collections::BTreeMap::new() }\n";
    assert_eq!(source_lines(source), Vec::<usize>::new());
}

#[test]
fn findings_name_the_module_and_the_remedy() {
    let found = check_source(Path::new("core.rs"), "use std::fs::File;\n").expect("valid Rust");
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("`std::fs`") && found[0].message.contains("port"));
}

#[test]
fn unparsable_sources_are_errors() {
    assert!(check_source(Path::new("core.rs"), "fn {").is_err());
}

fn xtask_arch(dir: &Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir)
        .arg("arch")
        .output()
        .expect("xtask runs")
}

#[test]
fn the_command_passes_on_this_repository() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let output = xtask_arch(root);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn the_command_fails_with_code_2_without_arch_toml() {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = xtask_arch(dir.path());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("arch.toml"));
}

/// A one-crate workspace, under git, whose core `lib.rs` holds `source`.
fn workspace_with_core_source(source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let write = |path: &str, text: &str| {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    };
    write(
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"core\"]\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
    );
    write(
        "core/Cargo.toml",
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lints]\nworkspace = true\n",
    );
    write("core/src/lib.rs", source);
    write(
        "arch.toml",
        "[core]\ncrate = \"core\"\n[composition-roots]\ncrates = []\n[adapters]\ndirectory = \"crates/adapters\"\n[tooling]\ncrates = []\n",
    );
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["init", "--quiet"])
        .status()
        .expect("git runs");
    assert!(status.success());
    dir
}

#[test]
fn the_workspace_check_reports_forbidden_imports_in_core_sources() {
    let dir = workspace_with_core_source("use std::fs;\n");
    let found: Vec<String> = check_workspace(dir.path())
        .expect("check runs")
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        found,
        ["core/src/lib.rs:1: core must not use `std::fs`: reach the outside world through a port"]
    );
}

#[test]
fn a_crate_without_workspace_lints_fails() {
    let mut packages = base_packages();
    packages[1].inherits_workspace_lints = false;
    let found = check_graph(&config(), &WorkspaceGraph { packages });
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].message.contains("`jukino`")
            && found[0].message.contains("[lints] workspace = true")
    );
    assert_eq!(found[0].path, Path::new("crates/jukino/Cargo.toml"));
}

#[test]
fn the_workspace_check_reads_each_crates_lints_table() {
    let dir = workspace_with_core_source("pub fn f() {}\n");
    std::fs::write(
        dir.path().join("core/Cargo.toml"),
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write");
    let found: Vec<String> = check_workspace(dir.path())
        .expect("check runs")
        .into_iter()
        .map(|finding| finding.message)
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("[lints] workspace = true"));
}

#[test]
fn a_crate_may_dev_depend_on_itself_to_enable_test_features() {
    assert_eq!(
        with_core(&[("jukino-core", DependencyKind::Dev, true)]),
        none()
    );
}
