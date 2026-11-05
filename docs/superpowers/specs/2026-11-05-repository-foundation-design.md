# Jukino repository foundation: design

Date: 2026-11-05
Status: draft, awaiting review

## Intent

Jukino is a new Rust project. Before any product code exists, the repository needs a
foundation that lets humans and coding agents add features safely:

- a root document describing what Jukino is, which the contributors will keep extending;
- robust linting, commit linting and test coverage, enforced by tools rather than by
  prose;
- a project-specific set of agent rules derived from the project's hexagonal-architecture
  report (the long "Turn the hexagon into checkable agent rules" document supplied with
  the request [artifact](https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof#44b4448d-5606)),
  compressed the way that report recommends;
- the contributor review workflow, wired in so agents cannot skip it.

The report's central finding drives the whole design: agents drop prose rules as their
number grows, copy existing violations, and edit tests when tests block them. So every
rule a tool can check becomes tool configuration plus one line of prose, and only
judgment rules get explanatory text.

In this document a *contributor* is any person working on Jukino, with or without an
agent. The contributor working with an agent reviews its changes, gives the go-ahead to
commit, and can approve guarded changes. The next section records the founding
contributor's answers from the design conversation.

## What the contributor said (their decisions)

Product, in the contributor's words, to be written into the root document:

> Jukino is a modern API interaction tool for humans and agents made in Rust. It allows
> managing HTTP requests in folders like a file system, and collections that group
> folders and requests, just like Postman or Bruno.
>
> Jukino allows creating graph-middleware on those folders and collections using
> JavaScript, which maintains state across a request execution.
>
> Jukino allows setting up request chain runs that execute configured requests in the
> specified order like a graph, and allows putting a script to join responses before the
> next request by using its graph-middleware.

Agents will reach Jukino through an MCP server and through mods for specific harnesses;
for now only Claude Code.

Merge bar for an agent's PR:

1. No blob comments in code. Explanations go in documentation comments (`///`, `//!`).
2. A detailed, comprehensive PR description covering the implementation.
3. **Review gate.** Before any commit, the agent stops and waits for the contributor's
   review. The contributor reviews in their code editor and leaves comments there; after
   a PR is opened, review comments may also arrive on the PR. Editor comments reach the
   agent as normal input: injected by the editor integration or pasted from the
   clipboard, so nothing needs to be built for that channel. The agent must not spend
   tokens showing diffs in chat.

   The agent continues past the gate only on a direct instruction from the contributor
   naming the next step, such as "commit", "go ahead and commit" or "open the PR":
   - When work is ready, the agent says so in one line and stops.
   - Review comments are not approval. The agent addresses them, says in one line that
     they are addressed, and stops again.
   - Silence, a question, praise, or a comment that is not an instruction is not
     approval.
   - An instruction covers only the step it names: "commit" does not cover pushing or
     opening a PR.
   - The gate applies again to every later commit, including fixes for PR comments.
4. **Guard label.** When a change edits existing tests or loosens a gate, CI blocks the
   PR until a contributor adds a `guard-approved` label. Locally the same check only warns,
   and the agent stops and reports, so an approved change never leaves it stuck.

Confirmed during the conversation:

- Root document is `VISION.md`; `README.md` stays short and links to it.
- The scaffold contains no product features, only what the pipeline needs to prove itself.
- Workspace starts with a `jukino-core` crate, a `jukino` CLI binary that is the
  composition root, a `jukino-ui` placeholder crate for the future human interface, and
  an `xtask` crate holding custom checks. MCP, JavaScript runtime and storage crates
  arrive with their first feature.
- Each open decision in the report (D1–D18) takes the report's marked default,
  translated to Rust, recorded in `CONVENTIONS.md` where contributors can overrule it.
- The report lives as the Claude Docs doc "Clean ports and adapters agent rules"
  (the clean-hexagon doc, <https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof>). The
  repository references it rather than copying it.
- A root `CONVENTIONS.md` holds the project's full conventions.
- The repository is a Tusk workspace with the Tusk Claude Code mod installed, so agents
  recall project knowledge through Tusk's MCP tools.

## Goals

1. `VISION.md` and a short `README.md` at the root.
2. A Cargo workspace that builds on a pinned stable toolchain.
3. Linting that fails the build: rustfmt, clippy with a strict workspace lint set,
   rustdoc warnings, the no-blob-comments check, architecture checks, unused-dependency
   and supply-chain checks.
4. Commit linting: Conventional Commits on every commit locally and in CI, plus the
   PR title.
5. Testing: nextest, doctests, a coverage gate, mutation testing on changed code, and a
   guard that flags changes to existing tests and to gate configuration.
6. Conventions and agent rules: a root `CONVENTIONS.md` with the full rule set for
   Rust and Jukino, including the resolved decisions; root `AGENTS.md` (imported by
   `CLAUDE.md`), path-scoped rule files and one on-demand skill as compressed extracts
   of it; the clean-hexagon doc referenced as the research behind the rules.
7. Claude Code settings and hooks that carry the invariants: review gate on commit, push
   and PR creation; formatter and comment check on edit; block on check-bypass flags;
   quick checks before the agent finishes a turn.
8. GitHub Actions CI that runs the same checks as the local `cargo xtask ci`.
9. A Tusk workspace (`tusk.toml`) indexing the project's knowledge documents, with the
   Tusk Claude Code mod installed for this workspace.
10. Every gate shown to be able to fail (no gate that passes vacuously).

## Non-goals

- Product features: request storage, HTTP execution, the JavaScript runtime, chain runs,
  the MCP server, the Claude Code mod.
- Choosing the UI framework or building any UI; `jukino-ui` stays a placeholder.
- Release automation, changelog generation, versioning policy.
- Dependabot or Renovate, benchmarks, fuzzing, a documentation site.
- Changing GitHub repository settings (branch protection, merge strategy). These are
  recommended in the README for a contributor with admin access to apply.

## Anti-goals

These would make the work a failure even if every check is green:

- A gate that cannot fail, or a rule that lives only in prose when a tool could check it.
- An agent pasting diffs into chat, or committing, pushing or opening a PR without the
  contributor's explicit go-ahead.
- A Node toolchain (`package.json`, `node_modules`) added only to lint commits.
- Speculative structure: traits, ports, crates or config options with no current user.
- Agent rule files so long they get ignored (root file over about 100 lines).

## Constraints

- Toolchain on this machine: rustc/cargo 1.99.1, rustup 1.29.3, lefthook 1.13.6, jq 1.7.1,
  gh 2.100, Homebrew, tusk v2.10.0 (`~/.local/bin/tusk`), Ollama (with
  `embeddinggemma-2:270m` and `all-minilm` pulled, not `nomic-embed-text`). Not
  installed: cargo-nextest, cargo-llvm-cov, cargo-deny, cargo-machete, cargo-mutants,
  committed.
- Remote: `git@github.com:germanamz/jukino.git`, default branch `main`, one commit
  (Apache-2.0 `LICENSE`).
- License compatibility: dependencies must be compatible with Apache-2.0.
- Project rule: no AI attribution anywhere (commits, PRs, code, docs).
- The review gate applies to building this foundation too: work happens on a branch, and
  nothing is committed until the contributor has reviewed it.

## Design

### Repository layout

```text
VISION.md                 product behaviours (contributors extend this)
README.md                 what it is, setup, commands, links
CONVENTIONS.md            full conventions for every contributor, human or agent
AGENTS.md                 root agent rules (~60–100 lines)
CLAUDE.md                 @AGENTS.md plus Claude-specific notes
Cargo.toml                workspace, shared lints, shared deps
Cargo.lock                committed (the workspace ships a binary)
rust-toolchain.toml       pinned 1.99.1 + rustfmt, clippy, llvm-tools-preview
rustfmt.toml              stable options only
clippy.toml               workspace thresholds and test allowances
deny.toml                 licenses, advisories, bans, sources
committed.toml            commit message rules
lefthook.yml              git hooks
arch.toml                 architecture rules read by `cargo xtask arch`
tusk.toml                 Tusk workspace manifest (committed)
.tusk/                    Tusk index (ignored)
.gitignore                target/, .tusk/, .claude/skills/tusk/, .claude/settings.local.json
.cargo/config.toml        `xtask` alias
.config/nextest.toml      default and CI profiles
.claude/settings.json     permissions and hooks
.claude/hooks/            hook scripts
.claude/rules/            path-scoped rules: core.md, adapters.md, tests.md
.claude/skills/placing-a-change/SKILL.md
.claude/skills/tusk/      Tusk Claude Code mod, generated by `tusk claude install` (ignored)
.github/workflows/ci.yml
.github/pull_request_template.md
crates/jukino-core/       the core: domain types and port traits, no I/O
crates/jukino/            binary: composition root + CLI driving adapter
crates/jukino-ui/         placeholder binary: future UI composition root + driving adapter
crates/adapters/          future adapter crates (does not exist until the first one)
xtask/                    custom checks and the `ci` runner
docs/superpowers/specs/   this design
```

### Jukino-specific reading of the hexagon

For most applications HTTP is technology. For Jukino it is the domain: HTTP methods,
URLs, headers, bodies, status codes, folders, collections, middleware graphs and chain
runs are all core concepts and belong in `jukino-core` as plain Rust types. What stays
outside is the machinery:

| Concern                                         | Lives in                                       | Port shape (future)                |
| ----------------------------------------------- | ---------------------------------------------- | ---------------------------------- |
| Sending a request over the network              | driven adapter wrapping an HTTP client library | core trait, e.g. `RequestSender`   |
| Reading and writing folders/collections on disk | driven adapter                                 | core trait, e.g. `CollectionStore` |
| Running JavaScript middleware                   | driven adapter wrapping a JS engine            | core trait, e.g. `ScriptRuntime`   |
| Clock, randomness, IDs, environment             | driven adapter or passed-in values             | `Clock` trait or plain arguments   |
| CLI                                             | driving adapter in `crates/jukino`             | calls core use cases directly      |
| UI for humans                                   | driving adapter in `crates/jukino-ui`          | calls core use cases directly      |
| MCP server                                      | driving adapter (future crate)                 | calls core use cases directly      |

The script source text and the middleware graph are domain data; the engine that runs
them is an adapter. This distinction goes into `AGENTS.md` because the report's generic
rule "ports must not name a technology" would otherwise be misread for an HTTP tool.

### Workspace and crates

- Edition 2024, `rust-version = "1.99"`, `resolver = "3"`; `rust-toolchain.toml` pins
  1.99.1, the current stable release on this machine.
- `crates/jukino-core` (library). Starts with crate docs and one deliberately small
  value object, `RequestPath`, so that tests, coverage and mutation testing have real
  logic to exercise. **My call.** It models a request's location in the folder tree:
  parsed from `"folder/sub/request"`, each segment non-empty, no `/` or `\`, not `.` or
  `..`, no control characters; `Display` round-trips. It is meant to be replaced or
  reshaped by the first real storage feature. If the contributor prefers, it can be dropped
  and the gates proven with fixtures only.
- `crates/jukino` (binary named `jukino`). `main.rs` is the composition root; a `cli`
  module is the driving adapter. Supports `--help` and `--version` only. Uses `clap`
  with the derive feature (**my call**: the CLI will need argument parsing for every
  future command). Smoke test runs the built binary via `env!("CARGO_BIN_EXE_jukino")`
  and `std::process::Command`, so no extra test dependency.
- `crates/jukino-ui` (binary named `jukino-ui`), a placeholder for the human interface.
  It reserves the crate and its place in the architecture: a second deployable whose
  `main.rs` will be its own composition root, with the UI as its driving adapter. Until
  a UI framework is chosen it has crate docs, a `main` that supports only `--help` and
  `--version` (same `clap` setup as the CLI), and a smoke test. **My call**: a binary
  rather than a library, because desktop UIs usually ship as their own executable.
- `xtask` (binary, not published). Subcommands below. Its own code is held to the same
  lints, tests and coverage as the product crates, because it guards the rules.
- Every crate uses `[lints] workspace = true`.

### Lints

Workspace `[workspace.lints]`:

- `rust`: `unsafe_code = "forbid"`, `missing_docs = "deny"`, `unused_must_use = "deny"`,
  `unreachable_pub = "warn"`, plus the `rust_2018_idioms` group.
- `clippy`: groups `all` and `pedantic` at deny (priority -1), `cognitive_complexity`
  enabled, and these restriction lints at deny: `unwrap_used`, `expect_used`, `todo`,
  `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `allow_attributes`,
  `allow_attributes_without_reason`, `wildcard_enum_match_arm`, `string_slice`,
  `mod_module_files`.
- Suppressing a lint is only possible as `#[expect(lint, reason = "…")]`; `allow` is
  rejected by `allow_attributes`. This is the Rust form of the report's "exceeding a
  ceiling requires an inline justification" (Q5), and `expect` fails once the lint no
  longer fires, so stale suppressions cannot linger.
- `clippy.toml` thresholds (decision D1, report default): function length 30 lines
  (`too-many-lines-threshold`), cognitive complexity 15, at most 3 parameters excluding
  `self`, nesting depth 3. Clippy counts parameters and nesting its own way; the builder
  calibrates `too-many-arguments-threshold` and `excessive-nesting-threshold` so the
  effective limits match these intents, and records the mapping in `CONVENTIONS.md`.
- Test code may use `unwrap`, `expect`, `panic`, printing and indexing:
  `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`,
  `allow-print-in-tests`, `allow-indexing-slicing-in-tests`. Integration-test helpers
  outside `#[test]` functions use crate-level `#![expect(…, reason = "test code")]`.
- The CLI's output module will use a module-level `#![expect(clippy::print_stdout, reason)]`
  once it prints anything itself; until then clap does the printing and an unfulfilled
  `expect` would fail the build.
- Rustdoc: `RUSTDOCFLAGS="-D warnings"` so broken intra-doc links fail.
- Lint warnings fail CI and `cargo xtask ci` (`-D warnings`). **My call**, departing
  from the report's "warn first, promote later": this is a new repository with no
  existing violations to tolerate.

Core-only restrictions, in `crates/jukino-core/clippy.toml` (clippy uses the nearest
`clippy.toml` and does not merge files, so this one repeats the workspace thresholds):

- `disallowed-types` and `disallowed-methods` covering file system, network, process,
  environment, stdio, threads and wall-clock or monotonic time (`SystemTime::now`,
  `Instant::now`).
- `max-fn-params-bools = 0` so public core functions take no boolean flags (Q8).

### Custom checks (`cargo xtask …`)

Alias in `.cargo/config.toml`: `xtask = "run --package xtask --"`. Each check prints
`path:line: message` so findings are clickable, exits non-zero on any finding, and has
fixture-based tests proving it fails on a violation and passes on clean input.

1. **`comments`** (the no-blob-comments rule). Lexes every tracked `.rs` file
   with a real Rust lexer (`ra_ap_rustc_lexer` is the intended crate, to be verified in
   the registry), so `//` inside string literals such as URLs is never mistaken for a
   comment. Fails on:
   - two or more consecutive lines that contain only a non-doc `//` comment (a blob);
   - any `/* … */` block comment, including `/** */` and `/*! */` (use `///` / `//!`);
   - a `//` comment whose text ends in `;`, `{` or `}` (commented-out code heuristic).
     A single `//` line explaining a non-obvious "why", or a trailing `//` on a code line,
     is allowed. Doc comments are never flagged. Runs on the whole tree, or on given paths
     (used by the edit hook).
2. **`arch`** (B1–B5, B9). Reads `cargo metadata` (via the `cargo_metadata` crate) and
   `arch.toml`, then asserts:
   - every workspace member is classified as core, adapter (under `crates/adapters/`),
     composition root (`jukino`, `jukino-ui`; one per deployable), or tooling; an
     unclassified crate fails;
   - the configured crates exist, so the check cannot pass vacuously;
   - core depends on no workspace crate, and on no external crate outside its allow-list
     (normal, dev and build dependencies all counted). The allow-list starts empty; each
     entry carries a reason;
   - adapters depend on no workspace crate except core and declared couplings;
   - only composition roots depend on adapter crates, and composition roots do not
     depend on each other.

   It also walks core sources with `syn` and rejects paths or `use` trees under
     `std::fs`, `std::net`, `std::env`, `std::process`, `std::io` and `std::thread`,
     as a second layer behind the clippy lists.
3. **`guard`** (G5, G9, decision D15 default: agents may add tests but not change
   existing ones without stopping; the contributor's answer extends this to gates). Compares
   the working tree with the merge base of `origin/main` (CI: the PR base) and reports:
   - **Existing tests changed.** Using `syn`: an existing `#[test]` function deleted,
     renamed, given `#[ignore]` or `#[should_panic]`, or with any token-level change to
     its body or attributes; a test file deleted. Adding new tests is not reported.
   - **Gates loosened or touched.** Any added, modified or deleted file under the
     protected paths: every `clippy.toml`, `rustfmt.toml`, `deny.toml`,
     `committed.toml`, `arch.toml`, `lefthook.yml`, `rust-toolchain.toml`,
     `.cargo/**`, `.config/nextest.toml`, `.github/**`, `.claude/settings.json`,
     `.claude/hooks/**`, `xtask/**` (gate code, including coverage thresholds), and the
     rule texts `CONVENTIONS.md`, `AGENTS.md`, `CLAUDE.md`, `.claude/rules/**` and
     `.claude/skills/**` (**my call**: a convention loosened in prose is still a
     loosened gate); and a change to the `[workspace.lints]` table or any crate's
     `[lints]` table. Additions count, because a new per-crate `clippy.toml` replaces the
     workspace one.
   - **New suppressions.** Any added `#[expect(…)]` or `#![expect(…)]` attribute.

   Locally (`cargo xtask ci`, pre-push, Stop hook) findings are printed as warnings and
   the exit code is 0; the agent rule says to stop and report them. With
   `--ci --labels <list>` it exits non-zero unless the list contains `guard-approved`. The
   foundation PR itself will need the label, since it adds every protected file.

4. **`coverage`**. Runs `cargo llvm-cov nextest` and enforces: workspace lines ≥ 90%,
   functions ≥ 90%, and `jukino-core` lines ≥ 95% (**my call**; the report sets no
   number). Per-crate figures come from the JSON summary. The only exclusion is
   `xtask/src/runner.rs`, the glue that launches external tools; it holds no decisions
   (every step it runs is tested data) and is also excluded from mutation testing.
5. **`mutants`**. Runs `cargo mutants --in-diff` against the merge base; any missed
   mutant fails. Not part of the default `ci` run because of cost, but required by the
   definition of done and run in CI on PRs.
6. **`ci`**. Runs, in order, stopping at the first failure: `cargo fmt --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`, `cargo doc` with
   `-D warnings`, `comments`, `arch`, `guard` (warning only), `cargo nextest run`,
   `cargo test --doc`, `coverage`, `cargo deny check`, `cargo machete`. Missing tools
   are reported with the exact install command.

### Commit linting

- Tool: `committed` (crate-ci), a Rust binary configured in `committed.toml`. **My
  call**, chosen over Node commitlint (anti-goal) and over hand-writing a parser in
  xtask.
- Rules: Conventional Commits; types `feat`, `fix`, `docs`, `style`, `refactor`, `perf`,
  `test`, `build`, `ci`, `chore`, `revert`; optional free-form scope; subject at most
  72 characters, imperative mood; no `fixup!`/`WIP` subjects on `main`.
- A separate commit-msg step rejects AI attribution lines (`Co-Authored-By` naming an
  AI, `Claude-Session:`, "Generated with"), implementing the no-attribution rule for
  every contributor's agent.
- CI checks every commit in the PR range and the PR title (the PR title becomes the
  commit under squash merge, which the README recommends).

### Git hooks (`lefthook.yml`)

- `pre-commit`: `cargo fmt --check` and `cargo xtask comments` on staged `.rs` files.
- `commit-msg`: `committed` and the attribution check.
- `pre-push`: `cargo xtask ci`.
- Installed with `lefthook install`; the README documents it.

### Tests

- Runner: cargo-nextest (`.config/nextest.toml` with a `ci` profile: no fail-fast, no
  retries). Doctests run separately with `cargo test --doc` because nextest does not run
  them.
- Core behaviour tests go through the public API in `crates/jukino-core/tests/`
  (report T9: test the driving port, not one file per type). In-module unit tests are
  for dense pure logic only.
- Test names describe behaviour in domain language (T10).
- Core tests use no I/O; this follows from the core's dependency allow-list and clippy
  lists, which apply to test targets too.
- Fakes, not mocks, for future driven ports; no mocking crate without the contributor's
  approval (D14 default). Each future driven port gets a contract suite, written as a
  generic test function or macro run against the fake and the real adapter (T6). The
  skill documents the pattern.
- The CLI smoke test boots the real binary (T8).

### Supply chain

- `deny.toml`: licenses allow-list compatible with Apache-2.0 (Apache-2.0, MIT,
  BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib; the builder adds others only if a
  real dependency needs them and they are compatible); advisories deny vulnerabilities
  and yanked crates; bans warn on duplicate versions; sources allow only crates.io.
- `cargo machete` for unused dependencies.

### CI (`.github/workflows/ci.yml`)

Triggers: pull requests and pushes to `main`. Jobs:

- `lint` (Ubuntu): fmt, clippy, doc, comments, arch, deny, machete.
- `test` (matrix: Ubuntu, macOS, Windows): nextest and doctests. Windows matters
  because folder-based storage will have path rules.
- `coverage` (Ubuntu): `cargo xtask coverage`.
- `guard` (PRs only): `cargo xtask guard --ci --labels …`; fails on any finding unless
  the PR has the `guard-approved` label. Re-runs when labels change
  (`pull_request` types include `labeled` and `unlabeled`).
- `mutants` (PRs only): `cargo xtask mutants` on the PR diff.
- `commits` (PRs only): `committed` on the PR commit range and the PR title.
- Tools installed with `taiki-e/install-action`; Rust via the pinned toolchain file;
  cargo cache with `Swatinem/rust-cache`. Third-party actions pinned to full commit SHAs.
- CI cannot run until the branch is pushed, which happens only after the contributor's
  go-ahead.
- CI-specific logic (commit range, PR title, label lookup, merge base) lives in xtask
  behind explicit inputs (`--base`, `--title`, `--labels`) and is covered by fixture
  tests locally; the workflow only passes `github.event` values in. The foundation PR's
  own CI run proves the wiring on the passing path. **My call**: a throwaway PR with
  deliberate violations, to prove the failing paths in CI, happens only if the contributor
  asks for it.

### Agent rules

Sized per the report's evidence: the root file names commands, paths and the few hard
invariants; tool-enforced rules get one line; judgment rules get a reason and, where
useful, a short good/bad example. Plain imperative wording throughout; `IMPORTANT`
appears once, on the review gate (decision D18).

**`AGENTS.md`** (about 60–100 lines), in this order:

1. One-line description and links to `VISION.md`, `CONVENTIONS.md`, the skill and the
   clean-hexagon doc.
2. **Review gate (IMPORTANT).** When a change is ready, say so in one line and stop. Do
   not paste diffs or file contents into chat. Review comments (from the editor, or on
   the PR once it exists) are not approval: address them, say so in one line, and stop
   again. Commit, push or open a PR only on a direct instruction from the contributor
   that names that step; silence, questions or praise do not count, and "commit" does
   not cover pushing. This applies to every commit, including fixes for PR comments.
3. Commands: `cargo xtask ci`, `cargo xtask mutants`, `cargo nextest run`,
   `cargo xtask comments <paths>`, `cargo xtask arch`.
4. Map: core, composition roots (CLI and UI placeholder), adapters location, xtask,
   where tests go.
5. Invariants, one line each with the reason and the check that enforces it: core has
   no technology dependencies and no I/O, clock, env or randomness; driven ports are
   traits declared in core and implemented by adapters; driving adapters call core use
   cases directly (no single-implementation traits for them); adapters do not depend on
   each other; only composition roots wire adapters; the HTTP and script vocabulary
   is domain, client libraries and JS engines are adapters.
6. Code rules that differ from model defaults: no blob comments, explanations in doc
   comments; public items documented, ports documenting inputs, outputs, errors, side
   effects; suppress lints only with `#[expect(…, reason)]`; no `unwrap`/`expect` outside
   tests; expected failures as `Result` with an error enum, bugs panic; parameter and
   length ceilings; no speculative traits, options or crates.
7. Tests: failing test first and observed red; never delete, ignore, weaken or edit an
   existing test, or change an expected value to match output (stop and report);
   a new test that passes first time must be shown able to fail; fakes over mocks.
8. Scope: name the placement step before editing (skill), stay in the named layers,
   cleanup only in touched files and at most about 10% of the diff (D16), new
   dependencies only with a stated reason and a version verified with `cargo add`.
9. Stop-and-report triggers (report G7, Jukino wording).
10. Commits and PRs: Conventional Commits, body says why; the PR description fills
    every section of the template in detail; no AI attribution.
11. Definition of done: a short checklist pointing at `cargo xtask ci` and
    `cargo xtask mutants`, mirrored in the PR template.

**`CLAUDE.md`**: `@AGENTS.md`, then two or three Claude-specific lines: which hooks run,
and that the placement skill exists.

**Path-scoped rules** in `.claude/rules/` with `paths:` front matter (the builder checks
the current Claude Code syntax):

- `core.md` (`crates/jukino-core/**`): error convention, `RequestPath`-style value
  objects enforce invariants in constructors (V3), pass time and IDs in (X1), no
  hybrids of data and behaviour (Q13), one term per concept with a glossary note (Q1),
  sum types and exhaustive matching versus traits (Q14).
- `adapters.md` (`crates/adapters/**`, `crates/jukino/**`, `crates/jukino-ui/**`):
  adapters translate only
  (A1), map vendor types to core types (A4), translate errors keeping the cause (E6),
  driving adapters validate shape and call one use case (A2, V1), retries and timeouts
  in driven adapters (A3), CLI maps core errors to exit codes (E7).
- `tests.md` (`**/tests/**`): T-rules and the guard workflow.

**Skill** `.claude/skills/placing-a-change/SKILL.md`: the report's placement procedure
(steps 0–7 and the escape hatch) rewritten for Jukino's crates, plus two workflows:
"add a driven port and its adapter" (contract suite first, then fake, then real
adapter) and "swap a technology" (adapter and composition root only; any core change
means stop and report). Its description must make it load when an agent plans a change.

**Reference material**: the clean-hexagon doc, "Clean ports and adapters agent rules"
(<https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof>), holds the research: the
claim-by-claim audit, the reasons and sources behind every rule, and the sizing
evidence. It is linked from `CONVENTIONS.md` and `AGENTS.md`, never copied or imported.
**My call**: no copy in the repository, so there is one live version. The doc is
private until shared; anyone who can't open it relies on `CONVENTIONS.md` alone, so
`CONVENTIONS.md` must stand on its own.

### `CONVENTIONS.md`

The project's complete conventions, written for every contributor, human or agent, and
self-contained. `AGENTS.md`, the path-scoped rules and the skill are compressed
extracts of it. Tool configuration is the source of truth for anything a tool checks;
`CONVENTIONS.md` describes those checks and never contradicts them. It is linked, not
imported, so its length does not count against the agent-file budget.

It keeps the clean-hexagon doc's rule IDs (B1, P4, Q7, …) so every rule traces back to
its reasoning and sources, but rewrites each rule for Rust and Jukino, with Rust
examples in place of the doc's TypeScript, Python and Java ones. Sections:

1. **How to read this.** Who reads which file; absolute versus judgment rules; how a
   departure from a judgment rule is recorded (in the PR description); how to change a
   convention (a PR touching `CONVENTIONS.md`, which needs `guard-approved`).
2. **Vocabulary**, in Rust terms: a driven port is a trait in `jukino-core`; a driving
   port is a concrete core type or function; an adapter is a crate or module that
   translates; a composition root is a binary's `main`.
3. **Jukino's hexagon**: the HTTP-is-domain reading and the crate map above.
4. **Rules by area**, each with ID, Rust statement, type and the check that enforces it:
   boundary (B), ports (P), adapters (A), core code quality (Q), comments and
   documentation (the no-blob-comments rule, doc-comment requirements), errors (E),
   tests (T), cross-cutting concerns (X), when not to apply the pattern (W), agent
   process (G). Transactions, validation, mapping and events (TX, V, M, EV) are listed
   as dormant until a transactional store or message broker exists, so they are ready
   when that feature arrives.
5. **Placement procedure**: steps 0–7 and the escape hatch. The skill adds worked
   Jukino examples and the two workflows.
6. **Commits, PRs and review**: Conventional Commits rules, the PR template sections,
   the review gate with its direct-instruction rule, the guard label, no AI attribution.
7. **Enforcement map**: each rule ID to its check (clippy lint, xtask check, hook, CI
   job, or review).
8. **Resolved decisions**: the table below, plus the clippy threshold calibration.

Resolved decisions, one row per decision:

| #        | Resolved as                                                                                                                       |
| -------- | --------------------------------------------------------------------------------------------------------------------------------- |
| D1       | 30 lines, cognitive complexity 15, nesting 3, 3 params excluding `self`; fail in CI                                               |
| D2       | Core types stay free of storage concerns; on-disk formats are adapter DTOs mapped to core types (also applies to `serde` derives) |
| D3       | Each adapter owns its DTOs and maps both ways at its boundary                                                                     |
| D4       | `Result` with per-port or per-use-case error enums; panics for bugs only                                                          |
| D5       | Role nouns in domain language (`CollectionStore`, `ScriptRuntime`), never library names                                           |
| D6       | Driving ports are concrete core types; no single-implementation traits                                                            |
| D7       | Canon TDD: test list, red before green, observed; commit-order enforcement impossible under the review gate                       |
| D8       | No logging port; adapters and decorators log; business observations as domain events if needed                                    |
| D9       | No prescribed internal structure; feature-first modules                                                                           |
| D10      | Crates by role, modules feature-first inside core                                                                                 |
| D11, D12 | Not applicable until a transactional store or message broker exists                                                               |
| D13      | Interface docs mandatory on public items and ports, plus the no-blob-comments rule                                                |
| D14      | Hand-written fakes; spies for fire-and-forget; no mocking crate without approval                                                  |
| D15      | Agents may add tests, not change existing ones; `guard` warns locally and blocks in CI without the `guard-approved` label         |
| D16      | About 10% of the diff, touched files only                                                                                         |
| D17      | Agent decides per feature and states it in the PR                                                                                 |
| D18      | Plain imperative; one `IMPORTANT`, on the review gate                                                                             |

### Claude Code settings (`.claude/settings.json`)

- Permissions: `ask` for `git commit`, `git push` and `gh pr create` (and `gh pr merge`),
  so the review gate holds even if an agent forgets the prose rule.
- `PreToolUse` on `Bash`: a script (jq-based) blocks `--no-verify`, `-n` on commit,
  `LEFTHOOK=0`, force pushes, and any command containing `guard-approved` (so the agent
  cannot label its own PR), returning the reason with exit code 2.
- `PostToolUse` on `Edit|Write`: for `.rs` files, run `rustfmt` on the file and
  `cargo xtask comments <file>`; findings go back to the agent.
- `Stop`: if tracked Rust or TOML files differ from `HEAD`, run quick checks (fmt,
  clippy, comments, arch, nextest) and block finishing on failure, once. It does not
  run `guard`: a Stop hook can only show output by blocking, and guard findings must
  never block; `guard` runs in `cargo xtask ci`, pre-push and CI instead.
  When the agent tries to stop again in the same turn (`stop_hook_active` is true), the
  hook lets it stop, so stop-and-report stays possible; the agent's one-line message
  must then name the failing checks. **My call**; the contributor can remove the hook if the
  latency annoys them.

### Tusk (agent memory)

Tusk turns the repository's markdown into an indexed graph and serves it over MCP, so
an agent finds the relevant convention, spec or vision section with one query instead
of reading whole files. It is a tool for developing Jukino; it is not Jukino's own
future Claude Code mod, which stays a non-goal.

- **Workspace.** `tusk init --name jukino`, which adopts untyped markdown as `note`
  nodes through the built-in `vault` pack. No custom node types until a real need
  appears.
- **What gets indexed.** `VISION.md`, `CONVENTIONS.md` and `docs/**`, with headings as
  sub-units so a single rule section can be retrieved. Ignored, following the pattern in
  Tusk's own manifest: `target/`, `.claude/`, `.github/`, `xtask/` fixtures,
  `docs/superpowers/plans/` (process artifacts), and the harness or human entry files
  `AGENTS.md`, `CLAUDE.md` and `README.md`, which agents load directly.
- **Embeddings.** `[embeddings]` with Ollama and `nomic-embed-text`, using the query and
  document prefixes and `lexical-weight = 0.5` from Tusk's own manifest, where they
  were measured. **My call.** This needs `ollama pull nomic-embed-text` on each
  machine; the builder verifies what Tusk does without Ollama (structural and lexical
  queries only, or an error) and documents it in the README.
- **Context digest.** `[context]` pins `VISION.md` so every session starts with the
  product picture; the builder sizes `max-bytes` so the digest stays small.
- **The mod.** `tusk claude install` writes the plugin to `.claude/skills/tusk/` (MCP
  server, session-start context hook, after-edit references hook) and approves its MCP
  server in `.claude/settings.local.json`. **My call**: the generated plugin is not
  committed, because it is rewritten on every Tusk upgrade and each contributor must
  run `install` anyway to approve the MCP server. The README documents the command.
- **Agent rule.** One line in `AGENTS.md`: look up project knowledge with the Tusk MCP
  tools (`tusk_query`) before reading whole documents.
- **Checks.** `tusk doctor` runs locally as part of the setup check. **My call**: it is
  not in CI for now, since CI would need Tusk and Ollama installed.
- **Hooks.** Tusk's session-start and after-edit hooks run alongside the project's own
  hooks; the builder confirms they don't conflict.

### PR template (`.github/pull_request_template.md`)

Sections the agent must fill in detail: Summary; Motivation; Placement step and layers
touched; Implementation details (per crate and module: what and how); Design decisions
and any judgment-rule departures with reasons; Tests added (behaviours) and evidence a
new test could fail; Evidence (commands run and their result lines); Definition of done
checklist; Follow-ups.

## Definition of done for this foundation

- `cargo xtask ci` passes on a clean checkout, and `cargo xtask mutants` reports no
  missed mutants.
- Each gate has been shown to fail on a deliberate violation: blob comment, block
  comment, commented-out code, `std::fs` in core, a new core dependency, adapter to
  adapter dependency, unclassified crate, deleted or modified test, protected gate file
  changed, new `#[expect]`, guard in CI mode without the label, coverage below the
  threshold, bad commit message, AI attribution line, `--no-verify` and the
  `guard-approved` label through the hook.
  The xtask checks prove this with fixture tests; the rest are demonstrated once.
- The Claude Code hooks fire as described.
- `tusk doctor` reports no errors, `tusk claude status` shows the mod installed, and a
  `tusk query` for a convention returns the matching `CONVENTIONS.md` section.
- All files are written and the contributor is told, in one line, that the work is ready
  for review. Nothing is committed until the contributor directly instructs it; then the
  commits go on a branch such as `chore/repository-foundation`. Pushing and opening the PR
  each wait for their own direct instruction.

## Left to the builder

- Exact clippy lint list beyond the ones named, and the clippy threshold calibration.
- Exact `disallowed-types`/`disallowed-methods` entries.
- Install method for local tools (Homebrew, `cargo install`, or `cargo binstall`).
- nextest profile details, CI caching details, action SHAs.
- Wording of every rule file and of `CONVENTIONS.md`, within the size and content
  limits above.
- Module structure inside `xtask`.
