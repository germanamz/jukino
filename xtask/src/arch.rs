//! The architecture check: which crates may depend on which.
//!
//! Every workspace member takes one role from `arch.toml`: the core, a composition
//! root, an adapter (any crate under the adapters directory) or tooling. The core
//! depends on no workspace crate and only on allow-listed external crates. Adapters
//! depend only on the core and on declared couplings. Only composition roots depend
//! on adapters, composition roots do not depend on each other, and product crates
//! do not depend on tooling.

pub mod sources;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::finding::Finding;
use crate::git;

/// The rules in `arch.toml`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ArchConfig {
    /// The core crate and its external allow-list.
    pub core: CoreRules,
    /// The crates allowed to wire adapters, one per deployable.
    pub composition_roots: CrateList,
    /// Where adapters live and which of them may use each other.
    pub adapters: AdapterRules,
    /// Repository tooling crates.
    pub tooling: CrateList,
}

/// The `[core]` table.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CoreRules {
    /// Package name of the core crate.
    #[serde(rename = "crate")]
    pub name: String,
    /// External crates the core may depend on.
    #[serde(default)]
    pub allowed_dependencies: Vec<AllowedDependency>,
}

/// An external crate the core may use, with the reason it is allowed.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowedDependency {
    /// Package name.
    pub name: String,
    /// Why the core may depend on it.
    pub reason: String,
}

/// A table holding a list of package names.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateList {
    /// Package names.
    pub crates: Vec<String>,
}

/// The `[adapters]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterRules {
    /// Directory, relative to the workspace root, holding every adapter crate.
    pub directory: PathBuf,
    /// Adapter-to-adapter dependencies that are allowed.
    #[serde(default)]
    pub couplings: Vec<Coupling>,
}

/// One adapter allowed to depend on another.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coupling {
    /// The depending adapter.
    pub from: String,
    /// The adapter it may depend on.
    pub to: String,
    /// Why the two cannot be separated.
    pub reason: String,
}

impl ArchConfig {
    /// Parses the text of `arch.toml`.
    ///
    /// # Errors
    ///
    /// Fails when the text is not TOML or does not match the schema above.
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

/// The workspace members and their declared dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceGraph {
    /// Every workspace member.
    pub packages: Vec<PackageNode>,
}

/// One workspace member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageNode {
    /// Package name.
    pub name: String,
    /// Directory of its manifest, relative to the workspace root.
    pub manifest_dir: PathBuf,
    /// Whether its manifest sets `[lints] workspace = true`.
    pub inherits_workspace_lints: bool,
    /// Its declared dependencies of every kind.
    pub dependencies: Vec<DependencyEdge>,
}

/// One declared dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    /// Package name of the dependency.
    pub name: String,
    /// Which section declares it.
    pub kind: DependencyKind,
    /// Whether the dependency is another workspace member.
    pub workspace_member: bool,
}

/// The manifest section a dependency is declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// `[dependencies]`.
    Normal,
    /// `[dev-dependencies]`.
    Dev,
    /// `[build-dependencies]`.
    Build,
}

/// Why the architecture check could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchError(pub String);

impl fmt::Display for ArchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ArchError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Core,
    CompositionRoot,
    Adapter,
    Tooling,
}

/// Runs every architecture rule on the workspace at `root`: the dependency graph
/// against `arch.toml`, then the core's sources against [`sources::check_source`].
///
/// # Errors
///
/// Fails when `arch.toml`, `cargo metadata`, git or a core source cannot be read.
pub fn check_workspace(root: &Path) -> Result<Vec<Finding>, ArchError> {
    let text = std::fs::read_to_string(root.join("arch.toml"))
        .map_err(|error| ArchError(format!("cannot read arch.toml: {error}")))?;
    let config =
        ArchConfig::parse(&text).map_err(|error| ArchError(format!("arch.toml: {error}")))?;
    let graph = load_graph(root)?;
    let mut findings = check_graph(&config, &graph);
    if let Some(core) = graph
        .packages
        .iter()
        .find(|package| package.name == config.core.name)
    {
        findings.extend(check_core_sources(root, &core.manifest_dir)?);
    }
    Ok(findings)
}

fn check_core_sources(root: &Path, core_dir: &Path) -> Result<Vec<Finding>, ArchError> {
    let files = git::rust_files(root).map_err(|error| ArchError(format!("git: {error}")))?;
    let mut findings = Vec::new();
    for file in files.iter().filter(|file| file.starts_with(core_dir)) {
        let source = std::fs::read_to_string(root.join(file))
            .map_err(|error| ArchError(format!("{}: {error}", file.display())))?;
        let found = sources::check_source(file, &source)
            .map_err(|error| ArchError(format!("{}: {error}", file.display())))?;
        findings.extend(found);
    }
    Ok(findings)
}

/// Reads the workspace members and their dependencies with `cargo metadata`.
///
/// # Errors
///
/// Fails when `cargo metadata` fails, for example outside a Cargo workspace.
pub fn load_graph(root: &Path) -> Result<WorkspaceGraph, ArchError> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(root)
        .no_deps()
        .exec()
        .map_err(|error| ArchError(format!("cargo metadata: {error}")))?;
    let members: Vec<String> = metadata
        .packages
        .iter()
        .map(|package| package.name.to_string())
        .collect();
    let workspace_root = metadata.workspace_root.as_std_path();
    let packages = metadata
        .packages
        .iter()
        .map(|package| package_node(package, &members, workspace_root))
        .collect();
    Ok(WorkspaceGraph { packages })
}

fn package_node(
    package: &cargo_metadata::Package,
    members: &[String],
    workspace_root: &Path,
) -> PackageNode {
    PackageNode {
        name: package.name.to_string(),
        manifest_dir: package
            .manifest_path
            .as_std_path()
            .parent()
            .and_then(|dir| dir.strip_prefix(workspace_root).ok())
            .map(Path::to_path_buf)
            .unwrap_or_default(),
        inherits_workspace_lints: inherits_workspace_lints(package.manifest_path.as_std_path()),
        dependencies: package
            .dependencies
            .iter()
            .map(|dependency| DependencyEdge {
                name: dependency.name.clone(),
                kind: kind_of(dependency.kind),
                workspace_member: members.contains(&dependency.name),
            })
            .collect(),
    }
}

fn inherits_workspace_lints(manifest: &Path) -> bool {
    std::fs::read_to_string(manifest)
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| table.get("lints")?.get("workspace")?.as_bool())
        == Some(true)
}

fn kind_of(kind: cargo_metadata::DependencyKind) -> DependencyKind {
    match kind {
        cargo_metadata::DependencyKind::Development => DependencyKind::Dev,
        cargo_metadata::DependencyKind::Build => DependencyKind::Build,
        cargo_metadata::DependencyKind::Normal | cargo_metadata::DependencyKind::Unknown => {
            DependencyKind::Normal
        }
    }
}

/// Checks the dependency graph against the rules in `config`.
#[must_use]
pub fn check_graph(config: &ArchConfig, graph: &WorkspaceGraph) -> Vec<Finding> {
    let mut findings = config_findings(config, graph);
    let roles: HashMap<&str, Role> = graph
        .packages
        .iter()
        .filter_map(|package| role_of(config, package).map(|role| (package.name.as_str(), role)))
        .collect();
    for package in &graph.packages {
        let messages = match roles.get(package.name.as_str()) {
            None => vec![format!(
                "`{}` has no role in arch.toml: name it as the core, a composition root or \
                 tooling, or move it under {}",
                package.name,
                config.adapters.directory.display()
            )],
            Some(role) => dependency_messages(config, package, *role, &roles),
        };
        let lints = (!package.inherits_workspace_lints).then(|| {
            format!(
                "`{}` must set `[lints] workspace = true` in its manifest",
                package.name
            )
        });
        let path = package.manifest_dir.join("Cargo.toml");
        findings.extend(messages.into_iter().chain(lints).map(|message| Finding {
            path: path.clone(),
            line: 1,
            message,
        }));
    }
    findings
}

fn config_findings(config: &ArchConfig, graph: &WorkspaceGraph) -> Vec<Finding> {
    missing_crates(config, graph)
        .into_iter()
        .chain(missing_reasons(config))
        .map(|message| Finding {
            path: PathBuf::from("arch.toml"),
            line: 1,
            message,
        })
        .collect()
}

fn missing_crates(config: &ArchConfig, graph: &WorkspaceGraph) -> Vec<String> {
    std::iter::once(&config.core.name)
        .chain(&config.composition_roots.crates)
        .chain(&config.tooling.crates)
        .filter(|name| !graph.packages.iter().any(|package| package.name == **name))
        .map(|name| format!("arch.toml names `{name}`, which is not a workspace member"))
        .collect()
}

fn missing_reasons(config: &ArchConfig) -> Vec<String> {
    let allowed = config
        .core
        .allowed_dependencies
        .iter()
        .filter(|entry| entry.reason.trim().is_empty())
        .map(|entry| format!("allow-list entry `{}` needs a reason", entry.name));
    let couplings = config
        .adapters
        .couplings
        .iter()
        .filter(|coupling| coupling.reason.trim().is_empty())
        .map(|coupling| {
            format!(
                "coupling `{}` -> `{}` needs a reason",
                coupling.from, coupling.to
            )
        });
    allowed.chain(couplings).collect()
}

fn role_of(config: &ArchConfig, package: &PackageNode) -> Option<Role> {
    if package.name == config.core.name {
        Some(Role::Core)
    } else if config.composition_roots.crates.contains(&package.name) {
        Some(Role::CompositionRoot)
    } else if config.tooling.crates.contains(&package.name) {
        Some(Role::Tooling)
    } else if package.manifest_dir.starts_with(&config.adapters.directory) {
        Some(Role::Adapter)
    } else {
        None
    }
}

fn dependency_messages(
    config: &ArchConfig,
    package: &PackageNode,
    role: Role,
    roles: &HashMap<&str, Role>,
) -> Vec<String> {
    package
        .dependencies
        .iter()
        .filter(|dependency| dependency.name != package.name)
        .filter_map(|dependency| {
            let target = roles.get(dependency.name.as_str()).copied();
            let edge = DependencyUse {
                from: &package.name,
                role,
                dependency,
                target,
            };
            violation(config, &edge)
        })
        .collect()
}

/// One dependency declared by a crate whose role is known.
struct DependencyUse<'a> {
    from: &'a str,
    role: Role,
    dependency: &'a DependencyEdge,
    target: Option<Role>,
}

fn violation(config: &ArchConfig, edge: &DependencyUse<'_>) -> Option<String> {
    let (from, to) = (edge.from, &edge.dependency.name);
    if !edge.dependency.workspace_member {
        let allowed = edge.role != Role::Core
            || config
                .core
                .allowed_dependencies
                .iter()
                .any(|entry| entry.name == *to);
        return (!allowed)
            .then(|| format!("core depends on `{to}`, which is not in arch.toml's allow-list"));
    }
    match (edge.role, edge.target) {
        (Role::Core, _) => Some(format!(
            "core depends on workspace crate `{to}`; the core depends on no other workspace crate"
        )),
        (Role::Adapter, Some(Role::Core)) => None,
        (Role::Adapter, _) => (!is_coupled(config, from, to)).then(|| {
            format!("adapter `{from}` depends on `{to}`; adapters depend only on the core and on couplings declared in arch.toml")
        }),
        (Role::Tooling, Some(Role::Adapter)) => {
            Some(format!("`{from}` depends on adapter `{to}`; only composition roots wire adapters"))
        }
        (Role::CompositionRoot, Some(Role::CompositionRoot)) => {
            Some(format!("composition root `{from}` depends on composition root `{to}`"))
        }
        (Role::CompositionRoot, Some(Role::Tooling)) => {
            Some(format!("`{from}` depends on tooling crate `{to}`; product crates do not depend on tooling"))
        }
        (Role::Tooling | Role::CompositionRoot, _) => None,
    }
}

fn is_coupled(config: &ArchConfig, from: &str, to: &str) -> bool {
    config
        .adapters
        .couplings
        .iter()
        .any(|coupling| coupling.from == from && coupling.to == to)
}
