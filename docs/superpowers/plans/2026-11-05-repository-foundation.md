# Repository Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the empty Jukino repository into a Rust workspace whose linting, commit
linting, testing, architecture rules, agent rules and review gate are enforced by tools.

**Architecture:** A Cargo workspace with a pure `jukino-core` crate, two composition-root
binaries (`jukino` CLI, `jukino-ui` placeholder), and an `xtask` crate that holds every
custom check behind `cargo xtask <check>`. Each check is a pure function over source text
or data, wrapped by thin process glue; Claude Code hooks, lefthook and GitHub Actions all
call the same xtask commands.

**Tech Stack:** Rust 1.99.1 (edition 2024), clap, ra-ap-rustc_lexer, syn, quote,
cargo_metadata, toml, serde, serde_json, tempfile; cargo-nextest, cargo-llvm-cov,
cargo-mutants, cargo-deny, cargo-machete, committed, lefthook, jq, Tusk, Ollama.

**Spec:** `docs/superpowers/specs/2026-11-05-repository-foundation-design.md`

## Global Constraints

- **Review gate, overriding the default commit steps of this plan format:** never run
  `git commit`, `git push` or `gh pr create`. No task ends in a commit. When all tasks
  are done, say in one line that the work is ready for review and stop. Commit, push and
  open the PR only on a direct instruction from the contributor naming that step.
- Never paste diffs or file contents into chat.
- Work on branch `chore/repository-foundation` (creating a branch is not a commit).
- Toolchain pinned to `1.99.1`; `rust-version = "1.99"`; edition 2024; `resolver = "3"`.
- Dependencies must be Apache-2.0 compatible; add each with `cargo add` and a stated
  reason. No `package.json` or Node tooling in the repo.
- No AI attribution anywhere (commits, PR text, code, docs).
- No blob comments: at most one consecutive non-doc `//` line, no `/* */`; explanations
  go in `///` / `//!`. Every public item has docs.
- Suppress a lint only with `#[expect(lint, reason = "…")]`; never `#[allow]`.
- Thresholds: function length 30, cognitive complexity 15, nesting 3, 3 params
  excluding `self`; coverage workspace lines ≥ 90%, functions ≥ 90%, `jukino-core`
  lines ≥ 95%; missed mutants in the diff = failure.
- `AGENTS.md` at most 100 lines. Say "contributor", never "owner".
- Every gate must be shown to fail on a deliberate violation.

## Review Focus

1. **CRLF line endings** (Windows checkouts): the comments check finds the same blobs
   and commented-out code in `\r\n` sources as in `\n` sources. Test in Task 3.
2. **Comment-like text that is not a comment** — `"http://x"`, `r#"// x"#`, `b"/* */"`,
   `'/'` — is never flagged, and a nested `/* /* */ */` yields exactly one finding.
   Tests in Task 3.
3. **Missing base ref** (shallow CI clone, no `origin/main`): `guard` and `mutants` exit
   with code 2 and an error naming the ref; they never report "no findings". Test in
   Task 5.
4. **Hook-bypass variants** — `git commit -nm x`, `git -c core.hooksPath=/dev/null commit`,
   `LEFTHOOK=0 git push`, `git push -f`, `git push --force-with-lease` — are all blocked,
   while `git log -n 5` and `head -n 3 f` are allowed. Tests in Task 8.
5. **Non-ASCII request paths** — `café/ñandú/créer` — parse and round-trip, and a segment
   with interior spaces (`get user`) is accepted. Tests in Task 2.

---

## File Structure

```text
Cargo.toml                      workspace: members, package defaults, lints, shared deps
rust-toolchain.toml             1.99.1 + rustfmt, clippy, llvm-tools-preview
rustfmt.toml, clippy.toml       formatter settings; workspace clippy thresholds
.cargo/config.toml              `xtask` alias
.cargo/mutants.toml             cargo-mutants settings (nextest, excluded glue)
.config/nextest.toml            default + ci profiles
.gitignore                      target/, .tusk/, .claude/skills/tusk/, .claude/settings.local.json
arch.toml, deny.toml, committed.toml, lefthook.yml, tusk.toml
crates/jukino-core/{Cargo.toml,clippy.toml,src/lib.rs,src/request_path.rs,tests/request_path.rs}
crates/jukino/{Cargo.toml,src/main.rs,src/cli.rs,tests/cli.rs}
crates/jukino-ui/{Cargo.toml,src/main.rs,src/cli.rs,tests/cli.rs}
xtask/Cargo.toml
xtask/src/main.rs               clap dispatch, printing, exit codes (only place that prints)
xtask/src/lib.rs                module list + crate docs
xtask/src/finding.rs            Finding (path, line, message) shared by every check
xtask/src/comments.rs           no-blob-comments check
xtask/src/arch.rs               arch.toml model + dependency-graph rules
xtask/src/arch/sources.rs       forbidden std paths in core sources (syn)
xtask/src/guard.rs              guard decision logic
xtask/src/guard/tests_diff.rs   test-function extraction and comparison (syn + quote)
xtask/src/guard/protected.rs    protected paths, lints-table and `expect` comparison
xtask/src/git.rs                git queries (merge base, changed files, file at rev, diff)
xtask/src/coverage.rs           llvm-cov JSON summary -> threshold violations
xtask/src/attribution.rs        AI-attribution line detection
xtask/src/pipeline.rs           `ci` step list and required-tool table (data only)
xtask/src/runner.rs             runs external commands (glue; excluded from coverage/mutants)
xtask/tests/{comments,arch,guard,coverage,attribution,pipeline,hooks}.rs
.claude/settings.json, .claude/hooks/{guard-bash,after-edit,stop-checks}.sh
.claude/rules/{core,adapters,tests}.md, .claude/skills/placing-a-change/SKILL.md
.github/workflows/ci.yml, .github/pull_request_template.md
VISION.md, README.md, CONVENTIONS.md, AGENTS.md, CLAUDE.md
```

---

### Task 1: Workspace skeleton, toolchain, lints and both binaries

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`,
  `.cargo/config.toml`, `.gitignore`, `crates/jukino-core/{Cargo.toml,clippy.toml,src/lib.rs}`,
  `crates/jukino/{Cargo.toml,src/main.rs,src/cli.rs}`, `crates/jukino-ui/{Cargo.toml,src/main.rs,src/cli.rs}`,
  `xtask/{Cargo.toml,src/main.rs,src/lib.rs}`
- Test: `crates/jukino/tests/cli.rs`, `crates/jukino-ui/tests/cli.rs`

**Interfaces:**
- Produces: workspace `[workspace.package]` (`version = "0.1.0"`, `edition = "2024"`,
  `rust-version = "1.99"`, `license = "Apache-2.0"`,
  `repository = "https://github.com/germanamz/jukino"`); every crate sets
  `[lints] workspace = true`; `cargo xtask` alias; xtask binary with an empty clap
  subcommand enum `Command` that later tasks extend.

- [ ] **Step 1: Branch and tools.** `git checkout -b chore/repository-foundation`.
  Install: `cargo install --locked cargo-nextest cargo-llvm-cov cargo-deny cargo-machete cargo-mutants committed`.
  Expected: each `<tool> --version` prints a version.

- [ ] **Step 2: Toolchain and workspace config.** `rust-toolchain.toml`:
  `channel = "1.99.1"`, `components = ["rustfmt", "clippy", "llvm-tools-preview"]`.
  `.cargo/config.toml`: `[alias] xtask = "run --package xtask --"`.
  `.gitignore`: `target/`, `.tusk/`, `.claude/skills/tusk/`, `.claude/settings.local.json`.
  `rustfmt.toml`: `edition = "2024"` and `newline_style = "Unix"` only.
  Root `Cargo.toml` lints, exactly:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "deny"
unused_must_use = "deny"
unreachable_pub = "warn"
rust_2018_idioms = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
cognitive_complexity = "deny"
unwrap_used = "deny"
expect_used = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
wildcard_enum_match_arm = "deny"
string_slice = "deny"
mod_module_files = "deny"
excessive_nesting = "deny"
```

- [ ] **Step 3: Write the failing smoke tests** in `crates/jukino/tests/cli.rs`, running
  `env!("CARGO_BIN_EXE_jukino")` with `std::process::Command`:
  - `version_flag_prints_the_package_version`: `--version` exits 0, stdout `== "jukino 0.1.0\n"`.
  - `help_flag_describes_the_command`: `--help` exits 0, stdout contains `"Usage: jukino"`.
  - `unknown_flag_is_a_usage_error`: `--bogus` exits with code 2, stderr contains `"unexpected argument"`.
  - `no_arguments_prints_help_as_a_usage_error`: no args exits with code 2, stderr contains `"Usage: jukino"`.
  Same four in `crates/jukino-ui/tests/cli.rs` with `jukino-ui`.

- [ ] **Step 4: Run them to verify they fail.** Run: `cargo nextest run`.
  Expected: FAIL (binaries do not exist / compile error).

- [ ] **Step 5: Implement both binaries.** `cli.rs` holds `pub(crate) struct Cli` with
  `#[derive(clap::Parser)]`, `#[command(version, about, arg_required_else_help = true)]`;
  `main.rs` (crate docs: the composition root) calls `Cli::parse()`. `clap` with the
  `derive` feature, reason: argument parsing for every future command. `jukino-core`
  gets crate docs only (`//!` stating it is the pure core: no I/O, clock, env or
  randomness). xtask gets `lib.rs` (crate docs) and `main.rs` with a `Command` enum.
  Do not add `#[expect(clippy::print_stdout)]` anywhere yet: nothing prints, and an
  unfulfilled `expect` fails.

- [ ] **Step 6: Run tests.** Run: `cargo nextest run`. Expected: 8 passed.

- [ ] **Step 7: Calibrate clippy thresholds** with a throwaway probe file in
  `crates/jukino-core/src/` (deleted afterwards). Find the smallest
  `too-many-arguments-threshold` that accepts `fn f(&self, a, b, c)` and the
  `excessive-nesting-threshold` that accepts three nested control blocks inside a
  function body but rejects four. If clippy counts `self`, use the value that allows
  `self` + 3 and note in `CONVENTIONS.md` (Task 10) that free functions then get one
  extra parameter. Write root `clippy.toml`: `too-many-lines-threshold = 30`,
  `cognitive-complexity-threshold = 15`, the two calibrated values,
  `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`,
  `allow-print-in-tests`, `allow-indexing-slicing-in-tests` all `true`.

- [ ] **Step 8: Core clippy.toml** at `crates/jukino-core/clippy.toml`: the root contents
  plus `max-fn-params-bools = 0`, and
  `disallowed-types` = `std::fs::File`, `std::fs::OpenOptions`, `std::net::TcpStream`,
  `std::net::TcpListener`, `std::net::UdpSocket`, `std::process::Command`,
  `std::process::Child`;
  `disallowed-methods` = `std::fs::{read, read_to_string, write, read_dir, create_dir, create_dir_all, remove_file, remove_dir, remove_dir_all, copy, rename, metadata}`,
  `std::env::{var, var_os, vars, args, args_os, current_dir, set_current_dir, temp_dir, home_dir}`,
  `std::process::exit`, `std::thread::{spawn, sleep}`, `std::time::SystemTime::now`,
  `std::time::Instant::now`, `std::io::{stdin, stdout, stderr}`, each with a `reason`
  naming the port that should be used instead.

- [ ] **Step 9: Verify lints and a failing violation.** Run:
  `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings`. Expected:
  clean. Then add `let _ = std::fs::read("x");` inside a temporary core function, run
  clippy, expect a `disallowed_methods` error, and remove it.

- [ ] **Step 10: Checkpoint.** Do not commit; continue.

---

### Task 2: `RequestPath` value object in the core

**Files:**
- Create: `crates/jukino-core/src/request_path.rs`; modify `crates/jukino-core/src/lib.rs`
- Test: `crates/jukino-core/tests/request_path.rs`

**Interfaces:**
- Produces: `jukino_core::RequestPath` (`Debug, Clone, PartialEq, Eq`) with
  `pub fn parse(input: &str) -> Result<RequestPath, RequestPathError>`,
  `pub fn segments(&self) -> impl Iterator<Item = &str> + '_`, `impl Display`,
  `impl FromStr<Err = RequestPathError>`; `jukino_core::RequestPathError`
  (`Debug, Clone, PartialEq, Eq`, implements `core::error::Error`) with variants
  `Empty`, `EmptySegment { index: usize }`, `ReservedSegment { index: usize }`,
  `ForbiddenCharacter { index: usize, character: char }`. `index` is 0-based.

- [ ] **Step 1: Write the failing tests:**

```rust
parses_nested_folders_and_the_request_name:    "users/admins/list" -> segments ["users","admins","list"]
a_single_segment_is_a_request_at_the_collection_root: "health" -> ["health"]
display_round_trips_the_parsed_text:            for "users/admins/list" and "café/ñandú/créer", to_string() == input
from_str_agrees_with_parse:                     "a/b".parse::<RequestPath>() == RequestPath::parse("a/b")
a_segment_with_interior_spaces_is_accepted:     "users/get user" -> ["users","get user"]
empty_input_is_rejected:                        "" -> Err(Empty)
empty_segments_are_rejected:                    "/a" -> EmptySegment{index:0}; "a/" -> EmptySegment{index:1}; "a//b" -> EmptySegment{index:1}
dot_segments_are_rejected:                      "a/./b" -> ReservedSegment{index:1}; "../a" -> ReservedSegment{index:0}
backslash_is_rejected:                          "a\\b" -> ForbiddenCharacter{index:0, character:'\\'}
control_characters_are_rejected:                "a/b\nc" -> ForbiddenCharacter{index:1, character:'\n'}; also '\t' and '\u{7f}'
errors_explain_themselves:                      Display of each variant equals:
  "request path is empty"
  "segment 1 of the request path is empty"
  "segment 0 of the request path is `.` or `..`"
  "segment 0 of the request path contains forbidden character '\\\\'"   (i.e. format!("{character:?}"))
```

- [ ] **Step 2: Run to verify failure.** `cargo nextest run -p jukino-core`. Expected: compile error, `RequestPath` not found.
- [ ] **Step 3: Implement** in `request_path.rs`, re-exported from `lib.rs`. Item docs
  describe the invariants (non-empty, `/` separates segments, no `\`, no `.`/`..`, no
  control characters per `char::is_control`) and the errors. Check segments in order;
  the first failing segment decides the error.
- [ ] **Step 4: Run tests.** `cargo nextest run -p jukino-core`. Expected: all pass.
- [ ] **Step 5: Show the tests can fail:** flip the `..` check, run, see
  `dot_segments_are_rejected` fail, restore.
- [ ] **Step 6: Checkpoint.** Do not commit; continue.

---

### Task 3: `cargo xtask comments`

**Files:**
- Create: `xtask/src/finding.rs`, `xtask/src/comments.rs`, `xtask/src/git.rs` (only `tracked_rust_files` now); modify `xtask/src/{lib.rs,main.rs}`
- Test: `xtask/tests/comments.rs`

**Interfaces:**
- Produces: `xtask::finding::Finding { pub path: PathBuf, pub line: usize, pub message: String }`
  with `Display` = `"{path}:{line}: {message}"` (1-based line);
  `xtask::comments::check_source(path: &Path, source: &str) -> Vec<Finding>`;
  `xtask::git::tracked_rust_files(root: &Path) -> io::Result<Vec<PathBuf>>`;
  CLI `cargo xtask comments [PATHS...]` (no paths = all tracked `.rs` files), exit 1
  when any finding, 0 otherwise. `main.rs` is the only module that prints; it carries
  `#![expect(clippy::print_stdout, clippy::print_stderr, reason = "xtask reports findings on the terminal")]`.

- [ ] **Step 1: Write the failing tests** (sources are string literals inside the test):
  - `a_single_why_comment_is_allowed`: one `//` line above code -> no findings.
  - `two_consecutive_comment_lines_are_a_blob`: lines 3–4 only `//` -> one finding, line 3, message contains `"comment block"`.
  - `trailing_comments_on_code_lines_are_not_a_blob`: two code lines each ending in `// x` -> none.
  - `doc_comments_are_never_flagged`: ten `///` lines and `//!` lines -> none.
  - `block_comments_are_rejected`: `/* x */`, `/** x */`, `/*! x */` -> one finding each, message contains `"block comment"`.
  - `nested_block_comment_is_one_finding`: `/* a /* b */ c */` -> exactly 1.
  - `commented_out_code_is_rejected`: `// let x = 1;`, `// fn f() {`, `// }` -> one finding each, message contains `"commented-out code"`.
  - `comment_markers_inside_literals_are_ignored`: `"http://x"`, `r#"// a\n// b"#`, `b"/* */"`, `'/'` -> none.
  - `crlf_sources_are_checked_like_lf`: the blob and commented-out sources above with `\r\n` give the same findings and lines.
  - `findings_print_as_clickable_locations`: `Finding` displays as `"src/a.rs:3: …"`.
- [ ] **Step 2: Run to verify failure.** `cargo nextest run -p xtask`. Expected: compile error.
- [ ] **Step 3: Implement** `check_source` with `ra_ap_rustc_lexer::tokenize(source, FrontmatterAllowed::No)`
  (`ra-ap-rustc_lexer = "0.183"`, reason: exact Rust lexing so string contents are never
  read as comments). Walk tokens with byte offsets; a line "holds only a comment" when its
  only non-whitespace token is a `LineComment { doc_style: None }`. Strip a trailing `\r`
  before the commented-out-code suffix test. Deps also: `clap` (derive).
- [ ] **Step 4: Run tests.** Expected: all pass.
- [ ] **Step 5: Dogfood.** `cargo xtask comments`. Expected: exit 0 on the whole tree.
- [ ] **Step 6: Checkpoint.** Do not commit; continue.

---

### Task 4: `cargo xtask arch` and `arch.toml`

**Files:**
- Create: `arch.toml`, `xtask/src/arch.rs`, `xtask/src/arch/sources.rs`
- Test: `xtask/tests/arch.rs`

**Interfaces:**
- Consumes: `Finding`.
- Produces: `ArchConfig` deserialized (serde, kebab-case) from:

```toml
[core]
crate = "jukino-core"
allowed-dependencies = []          # entries: { name = "…", reason = "…" }

[composition-roots]
crates = ["jukino", "jukino-ui"]

[adapters]
directory = "crates/adapters"
couplings = []                     # entries: { from = "…", to = "…", reason = "…" }

[tooling]
crates = ["xtask"]
```

  `WorkspaceGraph { packages: Vec<PackageNode> }`,
  `PackageNode { name: String, manifest_dir: PathBuf /* relative to root */, dependencies: Vec<DependencyEdge> }`,
  `DependencyEdge { name: String, kind: DependencyKind /* Normal | Dev | Build */, workspace_member: bool }`;
  `load_graph(root: &Path) -> Result<WorkspaceGraph, ArchError>` (via `cargo_metadata`);
  `check_graph(config: &ArchConfig, graph: &WorkspaceGraph) -> Vec<Finding>`;
  `sources::check_source(path: &Path, source: &str) -> Result<Vec<Finding>, syn::Error>`;
  CLI `cargo xtask arch` (exit 1 on findings, 2 on load errors).

- [ ] **Step 1: Write the failing tests**, building `WorkspaceGraph` values directly:
  - `the_real_workspace_passes`: `load_graph` on the repo root + `arch.toml` -> no findings.
  - `an_unclassified_crate_fails`: package `stray` at `crates/stray` -> finding naming `stray`.
  - `a_missing_configured_crate_fails`: config names `jukino-ui`, graph lacks it -> finding (non-vacuous).
  - `core_must_not_depend_on_workspace_crates`.
  - `core_external_dependency_needs_an_allow_list_entry`: normal, dev and build kinds each fail; allowed name passes.
  - `allow_list_entries_need_a_reason`: empty `reason` -> finding.
  - `adapters_must_not_depend_on_each_other`: unless a coupling `{from, to}` exists.
  - `only_composition_roots_depend_on_adapters`, `composition_roots_do_not_depend_on_each_other`,
    `product_crates_do_not_depend_on_tooling`.
  - `core_sources_must_not_name_io_modules`: `use std::fs;`, `use std::{env, io::Read};`,
    `std::process::exit(1)`, `::std::net::TcpStream` -> one finding each with the line;
    `use std::fmt;` and `core::str` -> none.
- [ ] **Step 2: Run to verify failure.** Expected: compile error.
- [ ] **Step 3: Implement.** Deps: `cargo_metadata` (workspace graph), `serde` + `toml`
  (config), `syn` with `full`, `visit` and `proc-macro2` with `span-locations` (line
  numbers). Forbidden prefixes: `std::fs`, `std::net`, `std::env`, `std::process`,
  `std::io`, `std::thread`, matched segment-wise on every `syn::Path` and every flattened
  `use` tree. Sources scanned: every `.rs` under the core crate's directory.
- [ ] **Step 4: Run tests.** Expected: all pass. Then `cargo xtask arch`: exit 0.
- [ ] **Step 5: Checkpoint.** Do not commit; continue.

---

### Task 5: `cargo xtask guard`

**Files:**
- Create: `xtask/src/guard.rs`, `xtask/src/guard/{tests_diff.rs,protected.rs}`; extend `xtask/src/git.rs`
- Test: `xtask/tests/guard.rs`

**Interfaces:**
- Consumes: `Finding`.
- Produces: `ChangeStatus { Added, Modified, Deleted }`,
  `ChangedFile { path: String, status: ChangeStatus }`;
  `git::merge_base(root, base_ref: &str) -> Result<String, GitError>` (error names the ref),
  `git::changed_files(root, base: &str) -> Result<Vec<ChangedFile>, GitError>` (`git diff --name-status --no-renames <base>` plus untracked non-ignored files as `Added`),
  `git::file_at(root, rev, path) -> Result<Option<String>, GitError>`,
  `git::diff_including_untracked(root, base) -> Result<String, GitError>` (used by Task 6);
  `tests_diff::compare(base: &str, current: Option<&str>) -> Result<Vec<(usize, String)>, syn::Error>`
  (line of the test in the current file, or in the base file for deletions, plus message);
  `protected::is_protected(path: &str) -> bool`, `protected::lints_changed(base: Option<&str>, current: Option<&str>) -> Result<bool, toml::de::Error>`,
  `protected::added_expects(base: Option<&str>, current: &str) -> usize`;
  `Mode { Local, Ci { labels: Vec<String> } }`, `Outcome { Clean, Warn, Fail }`,
  `decide(findings: &[Finding], mode: &Mode) -> Outcome`;
  CLI `cargo xtask guard [--base <ref>] [--ci] [--labels a,b]` — default base `origin/main`;
  exit 0 for `Clean`/`Warn`, 1 for `Fail`, 2 when the base cannot be resolved.

- [ ] **Step 1: Write the failing tests** (pure functions on strings, plus temp git repos via `tempfile` for git):
  - `adding_a_test_is_not_reported`.
  - `deleting_or_renaming_a_test_is_reported`: message contains `"deleted or renamed"` and the `module::fn` key.
  - `ignoring_a_test_is_reported` (`#[ignore]`), `should_panic_is_reported`, `changing_a_test_body_is_reported`; formatting-only change -> not reported.
  - `tests_in_nested_modules_and_async_test_attributes_are_found`: `mod a { mod b { #[test] fn t() {} } }` and `#[tokio::test]`.
  - `deleting_a_file_with_tests_is_reported`.
  - `protected_paths`: true for `clippy.toml`, `crates/x/clippy.toml`, `deny.toml`, `committed.toml`, `arch.toml`, `rustfmt.toml`, `lefthook.yml`, `rust-toolchain.toml`, `.cargo/config.toml`, `.cargo/mutants.toml`, `.config/nextest.toml`, `.github/workflows/ci.yml`, `.claude/settings.json`, `.claude/hooks/a.sh`, `xtask/src/lib.rs`, `CONVENTIONS.md`, `AGENTS.md`, `CLAUDE.md`, `.claude/rules/core.md`, `.claude/skills/x/SKILL.md`; false for `crates/jukino-core/src/lib.rs`, `VISION.md`, `tusk.toml`.
  - `lints_table_changes_are_detected`: `[workspace.lints.clippy]` value change -> true; dependency-only change -> false.
  - `new_expect_attributes_are_counted`: base 1, current 2 -> 1; `#![expect(` counts.
  - `local_mode_only_warns`, `ci_mode_fails_without_the_label`, `ci_mode_passes_with_guard_approved`, `no_findings_is_clean_in_both_modes`.
  - `untracked_files_count_as_added`: temp repo, new untracked `.rs` -> `ChangedFile{status: Added}`.
  - `an_unknown_base_ref_is_an_error`: `merge_base(root, "origin/nope")` -> `Err`, message contains `"origin/nope"` (Review Focus 3).
- [ ] **Step 2: Run to verify failure.** Expected: compile error.
- [ ] **Step 3: Implement.** Test functions: `fn` items whose attribute path's last
  segment is `test`, keyed by inline module path + name; compared by
  `quote::ToTokens::to_token_stream().to_string()` of the whole item (`quote` dep,
  reason: token-level comparison). Messages: `"test {key} deleted or renamed"`,
  `"test {key} gained #[ignore]"`, `"test {key} gained #[should_panic]"`,
  `"test {key} changed"`, `"test file deleted"`, `"protected file {status}"`,
  `"lints table changed"`, `"{n} new #[expect] attribute(s)"`. In `Warn`, print
  `"guard: stop and report these to the contributor"` after the findings; in `Fail`,
  `"guard: add the guard-approved label once a contributor has reviewed these"`.
  `tempfile` is a dev-dependency (reason: isolated git repos in tests).
- [ ] **Step 4: Run tests.** Expected: all pass. `cargo xtask guard` prints warnings
  (every protected file is new) and exits 0.
- [ ] **Step 5: Checkpoint.** Do not commit; continue.

---

### Task 6: Coverage, mutants and the `ci` pipeline

**Files:**
- Create: `xtask/src/{coverage.rs,pipeline.rs,runner.rs}`, `.cargo/mutants.toml`, `.config/nextest.toml`
- Test: `xtask/tests/{coverage.rs,pipeline.rs}`

**Interfaces:**
- Consumes: `git::merge_base`, `git::diff_including_untracked`, all earlier checks.
- Produces: `coverage::Thresholds { workspace_lines: f64, workspace_functions: f64, core_lines: f64 }`,
  `coverage::THRESHOLDS = { 90.0, 90.0, 95.0 }`,
  `coverage::evaluate(report_json: &str, core_dir: &str, thresholds: &Thresholds) -> Result<Vec<String>, serde_json::Error>`
  (violation messages; core figure = sum over files whose path contains `core_dir`);
  `pipeline::Step { name: &'static str, program: &'static str, args: &'static [&'static str], env: &'static [(&'static str, &'static str)] }`,
  `pipeline::CI_STEPS: &[Step]`, `pipeline::REQUIRED_TOOLS: &[(&str /* executable */, &str /* install command */)]`,
  `pipeline::missing_tools(is_on_path: impl Fn(&str) -> bool) -> Vec<String>`;
  `runner::run(step: &Step) -> io::Result<ExitStatus>`;
  CLI `cargo xtask coverage`, `cargo xtask mutants [--base <ref>]`, `cargo xtask ci`.

- [ ] **Step 1: Write the failing tests:**
  - `coverage_passes_at_the_thresholds`: fixture llvm-cov JSON (`data[0].totals` and `data[0].files[].summary`) at exactly 90/90/95 -> no violations.
  - `each_threshold_can_fail`: 89.9 lines, 89.9 functions, core 94.9 -> one message each naming the figure and threshold.
  - `core_figure_counts_only_core_files`.
  - `ci_steps_run_in_the_spec_order`: names `== ["fmt","clippy","doc","comments","arch","guard","nextest","doctest","coverage","deny","machete"]`.
  - `doc_step_denies_warnings`: `doc` step env contains `("RUSTDOCFLAGS", "-D warnings")`.
  - `missing_tools_name_their_install_command`: with nothing on PATH, every message contains `"cargo install --locked"`.
- [ ] **Step 2: Run to verify failure.** Expected: compile error.
- [ ] **Step 3: Implement.** `serde_json` dep (reason: llvm-cov JSON summary).
  `coverage` runs `cargo llvm-cov nextest --workspace --json --summary-only --ignore-filename-regex 'xtask/src/runner\.rs'`
  and evaluates stdout. `mutants` writes `git::diff_including_untracked` to
  `target/mutants.diff`; if empty it prints `"no changes to mutate"` and exits 0,
  else runs `cargo mutants --workspace --in-diff target/mutants.diff` and passes its exit
  code through. `ci` checks `missing_tools`, then runs `CI_STEPS` in order, stopping at
  the first failure (`guard` in local mode never stops it). `.cargo/mutants.toml`:
  `test_tool = "nextest"`, `exclude_globs = ["xtask/src/runner.rs"]`.
  `.config/nextest.toml`: `[profile.ci]` with `fail-fast = false`, `retries = 0`.
  `runner.rs` holds no decisions: everything it does is a `Step` value.
- [ ] **Step 4: Run tests.** Expected: all pass.
- [ ] **Step 5: Prove the gates.** `cargo xtask coverage`: passes. Add an untested public
  function to the core, rerun, see the core-lines violation, remove it.
  `cargo xtask mutants`: no missed mutants (fix tests if any are missed; never weaken them).
- [ ] **Step 6: Checkpoint.** Do not commit; continue.

---

### Task 7: Supply chain, commit linting and git hooks

**Files:**
- Create: `deny.toml`, `committed.toml`, `lefthook.yml`, `xtask/src/attribution.rs`
- Test: `xtask/tests/attribution.rs`

**Interfaces:**
- Produces: `attribution::attribution_lines(message: &str) -> Vec<usize>` (1-based);
  CLI `cargo xtask attribution (--file <path> | --range <a..b> | --title <text>)`, exit 1 on any hit.

- [ ] **Step 1: Write the failing tests:** a line is flagged when, case-insensitively, it
  starts with `co-authored-by:` and names `claude`, `anthropic`, `copilot`, `chatgpt`,
  `openai`, `gemini`, `codex` or `cursor`; starts with `claude-session:`; contains
  `claude.ai/code/session_`; or contains `generated with` together with one of those
  names. Tests: each pattern flagged with its line number; `Co-authored-by: Ana <ana@x.io>`
  not flagged; a body mentioning "Claude Code" in prose not flagged.
- [ ] **Step 2: Run to verify failure**, **Step 3: implement**, **Step 4: run to pass.**
- [ ] **Step 5: Config files.**
  `deny.toml`: licenses allow `Apache-2.0`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`,
  `Unicode-3.0`, `Zlib` (add others only if a real dependency needs a compatible one;
  confirm `ra-ap-rustc_lexer`'s legacy `MIT / Apache-2.0` string is accepted);
  advisories: vulnerabilities and yanked deny; bans: `multiple-versions = "warn"`;
  sources: `unknown-registry = "deny"`, `unknown-git = "deny"`.
  `committed.toml`: conventional style, allowed types `feat fix docs style refactor perf test build ci chore revert`,
  subject ≤ 72, imperative subject, no fixup, no WIP, merge commits rejected; confirm the
  key names with `committed --dump-config -`.
  `lefthook.yml`: `pre-commit` runs `cargo fmt --check` and `cargo xtask comments {staged_files}` (glob `*.rs`);
  `commit-msg` runs `committed --commit-file {1}` and `cargo xtask attribution --file {1}`;
  `pre-push` runs `cargo xtask ci`.
- [ ] **Step 6: Prove the gates.** `cargo deny check` and `cargo machete`: pass.
  Write `bad message` to a scratchpad file: `committed --commit-file <file>` fails; a
  message with a `Claude-Session:` line fails `cargo xtask attribution --file`.
  `lefthook install && lefthook validate`: pass. With a temporary blob comment in a core
  file, `lefthook run pre-commit --all-files` fails; remove it.
- [ ] **Step 7: Checkpoint.** Do not commit; continue.

---

### Task 8: Claude Code settings and hooks

**Files:**
- Create: `.claude/settings.json`, `.claude/hooks/{guard-bash.sh,after-edit.sh,stop-checks.sh}`
- Test: `xtask/tests/hooks.rs` (`#![cfg(unix)]`; runs the scripts with JSON on stdin)

**Interfaces:**
- Consumes: `cargo xtask comments`, `cargo xtask arch`.
- Produces: settings with `permissions.ask` = `Bash(git commit:*)`, `Bash(git push:*)`,
  `Bash(gh pr create:*)`, `Bash(gh pr merge:*)`; hooks `PreToolUse` (matcher `Bash`) ->
  `guard-bash.sh`, `PostToolUse` (matcher `Edit|Write`) -> `after-edit.sh`, `Stop` ->
  `stop-checks.sh`, all referenced via `"$CLAUDE_PROJECT_DIR"/.claude/hooks/…`.

- [ ] **Step 1: Write the failing tests** (exit code 2 = blocked, 0 = allowed; input
  `{"tool_input":{"command":"…"}}`):
  - blocked: `git commit --no-verify -m x`, `git commit -n -m x`, `git commit -nm x`,
    `git commit -anm x`, `git -c core.hooksPath=/dev/null commit -m x`, `LEFTHOOK=0 git push`,
    `LEFTHOOK_EXCLUDE=x git commit -m y`, `git push -f`, `git push --force`,
    `git push --force-with-lease`, `gh pr edit 3 --add-label guard-approved`.
  - allowed: `git commit -m "feat: x"`, `git log -n 5`, `head -n 3 f`, `git push origin chore/x`, `cargo xtask ci`.
  - `after-edit.sh` with a non-`.rs` path exits 0 with no output.
  - `stop-checks.sh` with `{"stop_hook_active":true}` exits 0 without running checks.
- [ ] **Step 2: Run to verify failure.** Expected: scripts missing.
- [ ] **Step 3: Implement** with `jq` (fail closed with a clear message if `jq` is
  missing). `guard-bash.sh` prints the reason to stderr and exits 2. `after-edit.sh`:
  for an existing `.rs` file, `rustfmt --edition 2024 <file>` then
  `cargo xtask comments <file>`; findings to stderr, exit 2. `stop-checks.sh`: exit 0
  when `stop_hook_active` is true or `git status --porcelain -- '*.rs' '*.toml'` is
  empty; otherwise run fmt check, clippy, `cargo xtask comments`, `cargo xtask arch`,
  `cargo nextest run`, and on failure print the failing step names to stderr and exit 2.
  It does not run `guard`: a Stop hook can only show output by blocking, and the
  foundation's guard warnings would block every turn; `guard` runs in `cargo xtask ci`,
  pre-push and CI.
- [ ] **Step 4: Run tests.** Expected: all pass.
- [ ] **Step 5: Checkpoint.** Do not commit; continue.

---

### Task 9: CI workflow and PR template

**Files:**
- Create: `.github/workflows/ci.yml`, `.github/pull_request_template.md`

- [ ] **Step 1: Workflow.** Triggers: `push` to `main`; `pull_request` types `opened,
  synchronize, reopened, labeled, unlabeled`. `permissions: contents: read`. Jobs:
  `lint` (ubuntu: fmt, clippy, doc, comments, arch, deny, machete), `test` (matrix
  ubuntu/macos/windows: nextest `--profile ci`, doctests), `coverage` (ubuntu),
  `guard`, `mutants`, `commits` (PR only, `fetch-depth: 0`). `guard` runs
  `cargo xtask guard --base origin/${{ github.base_ref }} --ci --labels "$LABELS"`;
  `commits` runs `committed <base_sha>..<head_sha>`, `cargo xtask attribution --range …`
  and checks the title via `cargo xtask attribution --title "$PR_TITLE"` and
  `committed --commit-file` on a file holding `$PR_TITLE`. PR title and labels reach
  scripts only through `env:` (never `${{ }}` inside `run:`), to prevent injection.
  Actions: `actions/checkout`, `Swatinem/rust-cache`, `taiki-e/install-action`, each
  pinned to a full commit SHA looked up with `gh api` (comment the tag beside it).
- [ ] **Step 2: Validate.** `brew install actionlint && actionlint .github/workflows/ci.yml`. Expected: no errors.
- [ ] **Step 3: PR template** with headings: Summary; Motivation; Placement step and
  layers touched; Implementation details; Design decisions and rule departures; Tests
  added; Evidence; Definition of done (checklist mirroring `AGENTS.md`); Follow-ups.
  Each heading has a one-line HTML comment saying what goes there.
- [ ] **Step 4: Checkpoint.** Do not commit; continue.

---

### Task 10: Vision, conventions and agent rules

**Files:**
- Create: `VISION.md`, `README.md`, `CONVENTIONS.md`, `AGENTS.md`, `CLAUDE.md`,
  `.claude/rules/{core,adapters,tests}.md`, `.claude/skills/placing-a-change/SKILL.md`

- [ ] **Step 1: Read the source.** Read the whole clean-hexagon doc ("Clean ports and
  adapters agent rules", <https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof>) through the
  Claude Docs connector before writing any rule text.
- [ ] **Step 2: `VISION.md`.** The spec's quoted product text, cleaned up but not
  extended; sections Behaviours (folders and requests, collections, graph-middleware,
  chain runs) and Interfaces (humans: CLI, UI planned; agents: MCP server and harness
  mods, Claude Code first). End with a note that contributors extend it.
- [ ] **Step 3: `CONVENTIONS.md`** with the eight sections the spec lists, rule IDs kept
  from the doc, every rule restated for Rust and Jukino with Rust examples, the
  enforcement map pointing at the real commands and files, the D1–D18 table, and the
  clippy calibration result from Task 1.
- [ ] **Step 4: `AGENTS.md`** in the spec's eleven-part order, including the Tusk line
  ("look up project knowledge with the Tusk MCP tools before reading whole documents");
  `CLAUDE.md` = `@AGENTS.md` plus the hooks and skill lines; path-scoped rules with
  `paths:` front matter (confirm the current syntax); the skill with steps 0–7, the
  escape hatch and the two workflows, and a description that triggers when planning a change.
- [ ] **Step 5: `README.md`**: what Jukino is (link `VISION.md`), prerequisites (Rust via
  rustup, the cargo tools from Task 1, lefthook, jq, Tusk, Ollama with
  `nomic-embed-text`), setup (`lefthook install`, `tusk claude install`), commands,
  recommended repository settings (squash merge, branch protection requiring CI), links
  to `CONVENTIONS.md` and `AGENTS.md`.
- [ ] **Step 6: Verify.** `wc -l AGENTS.md` ≤ 100; every rule ID in `AGENTS.md`,
  `.claude/rules/*` and the skill appears in `CONVENTIONS.md`
  (`grep -oE '\b[A-Z]{1,2}[0-9]{1,2}\b'` over both, compare); no file says "owner".
- [ ] **Step 7: Checkpoint.** Do not commit; continue.

---

### Task 11: Tusk workspace and mod

**Files:**
- Create: `tusk.toml` (via `tusk init`); modify `.gitignore` only if `tusk init` appends a duplicate

- [ ] **Step 1:** `tusk init --name jukino` (adopts the markdown as `note` via the vault pack).
- [ ] **Step 2: Edit `tusk.toml`:** `ignore` = `target/`, `.claude/`, `.github/`,
  `xtask/`, `docs/superpowers/plans/`, `AGENTS.md`, `CLAUDE.md`, `README.md`;
  `[embeddings]` provider `ollama`, endpoint `http://localhost:11434`, model
  `nomic-embed-text`, `dim = 768`, `query-prefix = "search_query: "`,
  `document-prefix = "search_document: "`; `[query] lexical-weight = 0.5`;
  `[context]` pins `VISION.md` with a small `max-bytes`.
- [ ] **Step 3:** `ollama pull nomic-embed-text`, `tusk reindex`, `tusk claude install`.
- [ ] **Step 4: Verify.** `tusk doctor` exits 0; `tusk claude status` reports installed;
  `tusk query 'type=note' --semantic 'where do retries and timeouts go'` lists
  `CONVENTIONS.md` first. Stop Ollama, rerun the query, and record in the README what
  happens without it.
- [ ] **Step 5: Checkpoint.** Do not commit; continue.

---

### Task 12: Whole-foundation verification and hand-off

- [ ] **Step 1:** `cargo clean && cargo xtask ci`. Expected: every step passes; `guard` warns about the new protected files.
- [ ] **Step 2:** `cargo xtask mutants`. Expected: no missed mutants.
- [ ] **Step 3: Gate demonstrations not covered by fixture tests**, each reverted after:
  `std::fs` call in core (clippy), coverage drop (Task 6), bad commit message and
  attribution line (Task 7), and in a live session: a `git commit --no-verify` attempt is
  blocked by the PreToolUse hook, an edit adding a blob comment is reported by the
  after-edit hook, and ending a turn with a failing test is blocked once by the Stop hook.
- [ ] **Step 4:** Confirm the Tusk hooks and the project hooks both fire in a new session.
- [ ] **Step 5: Hand-off.** Tell the contributor in one line that the foundation is
  ready for review, naming any check that is not green. Stop. On a direct instruction to
  commit, use these Conventional Commits on the branch:
  `build: set up the cargo workspace, toolchain and lints` (Task 1),
  `feat(core): add the RequestPath value object` (Task 2),
  `build(xtask): add comments, arch and guard checks` (Tasks 3–5),
  `build(xtask): add coverage, mutants and ci pipeline` (Task 6),
  `build: add supply-chain, commit and git-hook checks` (Task 7),
  `chore(claude): add review-gate permissions and hooks` (Task 8),
  `ci: add the GitHub Actions workflow and PR template` (Task 9),
  `docs: add vision, conventions and agent rules` (Task 10, plus the spec and this plan),
  `chore: set up the Tusk workspace` (Task 11).
  Push and the PR (description filling every template section) each wait for their own
  direct instruction; the PR needs the `guard-approved` label from a contributor.
