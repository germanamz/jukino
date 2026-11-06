# Workspace

The Cargo workspace that holds Jukino: which crates exist, what each one is for, how the
ports-and-adapters architecture maps onto them, and the configuration they share. Read it to
know where code lives. Where a particular change goes is decided by the placement procedure in
[CONVENTIONS.md](../CONVENTIONS.md#5-placement-procedure).

## Concepts

Core, driven port, driving port, adapter and composition root mean what the
[vocabulary](../CONVENTIONS.md#2-vocabulary) says. Domain terms such as request, folder and
request path come from the [domain glossary](../CONVENTIONS.md#domain-glossary).

## Behaviour

- `jukino` and `jukino-ui` accept only `--help` and `--version` so far. An unknown flag is a
  usage error, and running either binary with no arguments prints the help as a usage error
  (exit code 2).
- The core exposes one value object, `RequestPath`, with its error type `RequestPathError`.
  Its doc comments in `crates/jukino-core/src/request_path.rs` state the parsing rules.
- Every crate builds on the toolchain pinned in `rust-toolchain.toml`: Rust 1.99.1 with
  rustfmt, clippy and llvm-tools-preview.

## Design

### Crates

| Crate | Path | Role |
| --- | --- | --- |
| `jukino-core` | `crates/jukino-core/` | The core: domain types and, as they arrive, driven-port traits. No I/O and no technology dependencies. |
| `jukino` | `crates/jukino/` | The CLI binary. `main.rs` is the composition root and `cli.rs` the driving adapter, built on clap. |
| `jukino-ui` | `crates/jukino-ui/` | A placeholder for the human interface: a second composition root with the same clap setup. |
| `xtask` | `xtask/` | Repository tooling, the checks behind `cargo xtask` ([Checks](checks.md)). Product crates never depend on it. |

Adapter crates will live under `crates/adapters/<name>/`, and none exist yet. A driven port
and its adapter arrive with the first feature that needs them. The expected ones (sending
requests, storing collections, running JavaScript, the clock) are listed in
[Jukino's hexagon](../CONVENTIONS.md#3-jukinos-hexagon). The MCP server will be a driving
adapter in a crate of its own.

`arch.toml` classifies every crate as core, composition root, adapter or tooling, and lists
the core's allowed external dependencies (none yet) and the couplings allowed between adapters
(none yet). `cargo xtask arch` enforces it ([Checks](checks.md#cargo-xtask-checks)).

### Repository layout

```text
VISION.md                  what Jukino does for the people and agents using it
README.md                  what it is, setup, commands, links
CONVENTIONS.md             the rules every contributor follows
AGENTS.md, CLAUDE.md       the agents' short form of CONVENTIONS.md
docs/                      design documents, one per area (this file is one)
tickets/                   planned work, as Tusk tickets
Cargo.toml, Cargo.lock     the workspace, shared lints and dependency versions
rust-toolchain.toml        the pinned toolchain
rustfmt.toml, clippy.toml  formatter settings and lint thresholds
arch.toml                  architecture rules for `cargo xtask arch`
deny.toml                  supply-chain rules for `cargo deny`
committed.toml             commit-message rules for `committed`
lefthook.yml               git hooks
tusk.toml                  the Tusk workspace manifest
.gitattributes             LF line endings on every platform
.cargo/                    the `xtask` alias and cargo-mutants settings
.config/nextest.toml       the nextest CI profile
.claude/                   Claude Code settings, hooks, path-scoped rules and skills
.github/                   CI workflows and the pull-request template
crates/jukino-core/        the core
crates/jukino/             the CLI binary
crates/jukino-ui/          the placeholder UI binary
xtask/                     the repository checks
```

The [Workflow](workflow.md) document describes the files under `.claude/`, `.github/`,
`docs/`, `tickets/` and `tusk.toml`. The [Checks](checks.md) document describes the check
configuration.

### Shared configuration

- `[workspace.package]` sets version 0.1.0, edition 2024, `rust-version = "1.99"` and the
  Apache-2.0 license for every crate. The workspace uses resolver 3.
- `[workspace.dependencies]` pins each shared dependency's version once, and every crate sets
  `[lints] workspace = true` so the workspace lints apply everywhere.
- `Cargo.lock` is committed, because the workspace ships binaries.
- `.cargo/config.toml` defines `cargo xtask` as `cargo run --package xtask --`.

## Decisions

- **HTTP is domain, not technology.** Requests, folders, collections, middleware graphs and
  chain runs are core types, and only the machinery (HTTP client, storage, JavaScript engine)
  sits behind ports. This is spelled out in CONVENTIONS.md and AGENTS.md because the generic
  rule "ports do not name a technology" would otherwise be misread for an HTTP tool.
- **Crates by role, modules by feature inside the core** (D9, D10). A crate boundary is
  something `cargo xtask arch` can check, while feature modules keep related code together.
- **`RequestPath` as the first value object.** The checks needed real logic to exercise:
  tests, coverage and mutation testing. `RequestPath` models a request's place in the folder
  tree. The first storage feature may reshape or replace it. Rejected: proving the checks with
  fixtures only.
- **`jukino-ui` is a binary, not a library.** Desktop interfaces usually ship as their own
  executable, so the UI gets its own composition root. Its framework is not chosen.
- **clap with the derive feature in both binaries**, since every future command needs
  argument parsing.
- **Smoke tests run the built binary** through `env!("CARGO_BIN_EXE_jukino")` and
  `std::process::Command`, so they need no extra test dependency.

## Open questions

- The UI framework for `jukino-ui`.
- The HTTP client library, JavaScript engine and on-disk format behind the first adapters.
  Each is decided with the feature that first needs it.
