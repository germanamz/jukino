# Checks

The tools that hold every change to Jukino's conventions: lints, the custom `cargo xtask`
checks, tests with coverage and mutation testing, supply-chain and commit-message checks, git
hooks and CI. The rules themselves are in [CONVENTIONS.md](../CONVENTIONS.md), whose
[enforcement map](../CONVENTIONS.md#7-enforcement-map) says which check covers which rule.
This document describes how the checks work and why they are built this way.

## Behaviour

- `cargo xtask ci` runs every check in the order CI runs them and stops at the first failure:
  fmt, clippy, doc, comments, arch, guard, nextest, doctests, coverage, deny, machete. When a
  tool is missing, it prints the command that installs it.
- A check prints each finding as `path:line: message`, so editors make it clickable. A check
  with findings exits with code 1. A check that cannot run, for example because the base ref
  is missing in a shallow clone, exits with code 2 and names the cause. It never reports
  "no findings" instead.
- `cargo xtask guard` only warns locally, so a contributor's approved change never leaves an
  agent stuck. With `--ci --labels <list>` it fails unless the list holds `guard-approved`.
- `cargo xtask mutants` is not part of `cargo xtask ci`, because it is slow. The definition of
  done requires it, and CI runs it on every pull request.

## Design

### Principles

- **Tools over prose.** The research behind the conventions found that agents drop prose rules
  as the rules multiply, copy existing violations, and edit tests that block them. So a rule a
  tool can check becomes tool configuration plus one line of prose, and only judgment rules
  get explanatory text.
- **No gate passes vacuously.** Each check has fixture tests that show it failing on a
  violation and passing on clean input. A check also fails when what it checks is missing: the
  arch check when a configured crate is absent, the coverage check when the report has no data.
- **One implementation per check.** Claude Code hooks, lefthook and GitHub Actions all call
  the same `cargo xtask` commands. Each check is a pure function over source text or data.
  Launching external tools is thin glue in `xtask/src/runner.rs`, the only code excluded from
  coverage and mutation testing.
- **CI inputs are flags.** The commit range, pull-request title and body, labels and base ref
  reach xtask as explicit arguments (`--range`, `--title`, `--body`, `--labels`, `--base`).
  That logic is therefore tested locally, and the workflows only pass `github.event` values in.
- **xtask meets the product's bar.** Its code is held to the same lints, tests and coverage as
  the product crates, because it guards the rules.

### Lints

- `Cargo.toml`'s `[workspace.lints]` forbids `unsafe_code` and denies `missing_docs`,
  `unused_must_use`, clippy's `all` and `pedantic` groups, `cognitive_complexity`,
  `excessive_nesting` and a set of restriction lints: `unwrap_used`, `expect_used`, `todo`,
  `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `allow_attributes`,
  `allow_attributes_without_reason`, `wildcard_enum_match_arm`, `string_slice` and
  `mod_module_files`. `unreachable_pub` and `rust_2018_idioms` warn, and warnings fail too.
- A lint is suppressed only with `#[expect(lint, reason = "…")]`. `allow_attributes` rejects
  `#[allow]`.
- `clippy.toml` sets the limits from decision D1 (30 lines, cognitive complexity 15, `self`
  plus 3 parameters, three nested blocks), calibrated to how clippy counts, and lets test code
  use `unwrap`, `expect`, `panic`, printing and indexing.
- `crates/jukino-core/clippy.toml` repeats those thresholds, because clippy reads only the
  nearest `clippy.toml` and does not merge them. It adds `max-fn-params-bools = 0` and
  disallows the standard library's file system, network, process, environment, stdio, thread
  and clock APIs, each with the port to use instead.
- `cargo doc` runs with `RUSTDOCFLAGS="-D warnings"`, so a broken intra-doc link fails.

### `cargo xtask` checks

- **`comments`** lexes each Rust file with `ra-ap-rustc_lexer`, so `//` inside a string, raw
  string or byte string is never taken for a comment. It reports a blob (two or more
  consecutive lines holding only a non-doc `//` comment) once, with its length; any block
  comment, a nested one counting once; and a `//` comment ending in `;`, `{` or `}`, taken as
  commented-out code. Doc comments are never reported, and CRLF files are checked like LF
  files. With no arguments it checks every Rust file git tracks or would track. Given paths,
  it checks those and skips missing ones.
- **`arch`** reads `cargo metadata` and `arch.toml`. Every workspace crate must be classified
  and every configured crate must exist. The core depends on no workspace crate and only on
  allow-listed external crates, counting normal, dev and build dependencies. A crate may
  dev-depend on itself to enable test features. Allow-list entries and couplings need a
  reason. Adapters depend only on the core and declared couplings. Only composition roots
  depend on adapters, composition roots do not depend on each other, and product crates do not
  depend on tooling. Every crate sets `[lints] workspace = true`. As a second layer behind the
  clippy lists, it parses the core's sources with `syn` and rejects any path under `std::fs`,
  `std::net`, `std::env`, `std::process`, `std::io` or `std::thread`.
- **`guard`** compares the working tree, untracked files included, with its merge base against
  `--base` (default `origin/main`). It reports existing tests changed: a `#[test]` function
  deleted, renamed, given `#[ignore]` or `#[should_panic]`, or changed in any token
  (reformatting is not a change), and a deleted file that held tests. In `tests/` files and
  `#[cfg(test)]` modules it also reports changed helpers and file or module attributes. It
  reports changes to gate files (the list is in `xtask/src/guard/protected.rs`): the check
  configuration, `.cargo/`, `.github/`, `xtask/`, `.claude/settings.json`, `.claude/hooks/`,
  `.claude/rules/`, `.claude/skills/` and the rule texts `CONVENTIONS.md`, `AGENTS.md` and
  `CLAUDE.md`. It also reports a change to `[workspace.lints]`, a crate `[lints]` table that
  becomes anything but `workspace = true`, and every new `#[expect]` attribute. Adding a test
  is never reported. A source file it cannot parse is reported, not skipped.
- **`coverage`** runs `cargo llvm-cov nextest`, writes `target/coverage-summary.json` and
  requires workspace lines and functions at 90% or more and `jukino-core` lines at 95% or more.
- **`mutants`** runs `cargo mutants --in-diff` on the changes since the merge base, untracked
  files included, with the settings in `.cargo/mutants.toml`: nextest as the test tool, a copy
  of `.git` in each scratch tree (the arch check lists files through git), and `runner.rs`
  excluded. Any missed mutant fails.
- **`attribution`** rejects AI co-author trailers, session trailers and links, and
  "generated with" lines in a commit message file, a commit range, a pull-request title or a
  pull-request body. Human co-authors and ordinary prose pass.
- **`ci`** runs the pipeline. Its step list is data in `xtask/src/pipeline.rs`, so the order,
  the arguments and the required-tool table are tested.

### Tests

The tests run under cargo-nextest. Its `ci` profile in `.config/nextest.toml` turns off
fail-fast and retries. Doctests run separately with `cargo test --doc`, because nextest does
not run them. Where tests go and how they are written is in
[CONVENTIONS.md](../CONVENTIONS.md#tests-t).

### Supply chain

`deny.toml` allows only licenses compatible with Apache-2.0 (Apache-2.0, MIT, BSD-2-Clause,
BSD-3-Clause, ISC, Unicode-3.0, Zlib). It fails on advisories and yanked crates, warns on
duplicate versions, and allows crates from crates.io only. `cargo machete` finds unused
dependencies. It runs as its own binary, because when run as `cargo machete` under `cargo run`
it misreads its arguments.

### Commit messages

`committed` checks Conventional Commits against `committed.toml` (rule R3), and
`cargo xtask attribution` rejects AI attribution (rule R5). Both run on every commit message
locally and, in CI, on every commit in the pull request and on its title. The attribution check
also runs on the pull-request description.

### Git hooks

`lefthook.yml` installs three hooks with `lefthook install`: `pre-commit` runs `cargo fmt` and
the comments check on staged Rust files, `commit-msg` runs `committed` and the attribution
check, and `pre-push` runs `cargo xtask ci`.

### CI

- `.github/workflows/ci.yml` runs on pull requests and pushes to `main`. `lint` on Ubuntu runs
  fmt, clippy, doc, comments, arch, deny and machete. `test` runs nextest with the `ci`
  profile and the doctests on Ubuntu, macOS and Windows. `coverage` runs on Ubuntu. On pull
  requests only, `mutants` runs with the full history and a 90-minute timeout.
- `.github/workflows/pr-checks.yml` runs on pull requests, including when one is edited,
  labelled or unlabelled. `guard` runs with the pull request's labels. `commits` runs
  `committed` on the commit range and the title, and the attribution check on the range, the
  title and the description. These jobs live apart from `ci.yml`, so a label or description
  change re-runs only them.
- Both workflows read the toolchain from `rust-toolchain.toml`, cache with
  `Swatinem/rust-cache`, install tools with `taiki-e/install-action` and pin third-party
  actions to full commit SHAs. A newer push cancels the running checks of the same pull request.

## Decisions

- **Warnings fail from the start.** This departs from the research's "warn first, promote
  later": a new repository has no violations to tolerate.
- **`#[expect]` instead of `#[allow]`.** An `expect` fails once its lint stops firing, so stale
  suppressions cannot linger. This is the Rust form of "exceeding a limit needs an inline
  justification" (Q5).
- **A real Rust lexer for the comments check**, so a URL in a string literal is never a
  comment. Rejected: line-based pattern matching.
- **`committed` for commit messages.** Rejected: Node's commitlint, which would add a Node
  toolchain only to lint commits, and a parser written in xtask.
- **The guard warns locally and blocks in CI.** A contributor approves a guarded change with
  the `guard-approved` label, which agents never add (D15). An approved change never leaves an
  agent stuck locally.
- **Rule texts are gates.** A convention loosened in prose is still a loosened gate, so
  `CONVENTIONS.md`, `AGENTS.md`, `CLAUDE.md` and everything under `.claude/` that steers agents
  are protected like the check configuration.
- **Coverage minimums are Jukino's own**, since the research sets none. `runner.rs` is the only
  exclusion, because every decision it runs is tested elsewhere.
- **Windows is in the test matrix** because folder-based storage will have path rules.
- **Failing paths are proven by fixture tests, not by a throwaway pull request** with
  deliberate violations. The passing path in CI is proven by every pull request.
