# Agent rules for Jukino

Jukino is an API interaction tool for humans and agents, written in Rust ([VISION.md](VISION.md)).
These rules are the short form of [CONVENTIONS.md](CONVENTIONS.md); rule IDs (B1, T6, …) point
there. The research behind them is the clean-hexagon doc, "Clean ports and adapters agent rules"
(<https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof>). Before editing, follow the
`placing-a-change` skill (`.claude/skills/placing-a-change/SKILL.md`).

## Review gate

IMPORTANT: when a change is ready, say so in one line and stop. Do not paste diffs or file
contents into chat. Review comments, from the contributor's editor or on the PR, are not
approval: address them, say so in one line, and stop again. Commit, push or open a PR only on a
direct instruction from the contributor that names that step. Silence, questions and praise are
not instructions, and "commit" does not cover pushing. This applies to every commit, including
fixes for PR comments.

## Commands

- `cargo xtask ci`: every check, in CI order. Run it before saying a change is ready.
- `cargo xtask mutants`: mutation-test the code changed since `origin/main`.
- `cargo nextest run --workspace`: the tests.
- `cargo xtask comments [paths]`, `cargo xtask arch`, `cargo xtask guard`: single checks.
- Look up project knowledge (vision, conventions, specs) with the Tusk MCP tools (`tusk_query`)
  before reading whole documents.

## Map

- `crates/jukino-core/`: the core. Domain types and port traits; no I/O.
- `crates/jukino/`: the CLI binary. `main.rs` is the composition root, `cli.rs` the driving adapter.
- `crates/jukino-ui/`: placeholder UI binary, a second composition root.
- `crates/adapters/<name>/`: adapter crates (none yet).
- `xtask/`: the checks behind `cargo xtask`.
- Core behaviour tests go through the public API in `crates/jukino-core/tests/`.

## Invariants

- The core has no technology dependencies and no I/O, clock, environment, process or
  randomness (B1, B6, X1). Checked by `cargo xtask arch` and the core's `clippy.toml`.
- Driven ports are traits declared in the core and implemented by adapters. Driving adapters
  call core types directly, without single-implementation traits (B2, B3, P8, W2).
- Adapters do not depend on each other except couplings declared in `arch.toml`, and only
  composition roots wire adapters (B4, B5). Checked by `cargo xtask arch`.
- Requests, folders, collections, middleware graphs and chain runs are domain concepts in the
  core. HTTP client libraries, storage and JavaScript engines are adapters (P1).
- A new external dependency of the core goes on `arch.toml`'s allow-list only with a stated
  reason and the contributor's approval.

## Code

- No comment blocks: at most one `//` line in a row, no `/* */`, no commented-out code.
  Explanations go in doc comments (`///`, `//!`). Checked by `cargo xtask comments`.
- Document every public item. Ports document inputs, outputs, errors, side effects and
  invariants (P5).
- Suppress a lint only with `#[expect(lint, reason = "…")]`, never `#[allow]`.
- No `unwrap` or `expect` outside tests. Expected failures are `Result` with an error enum;
  bugs panic (E1–E4).
- Limits: 30 lines per function, cognitive complexity 15, `self` plus 3 parameters, three nested
  blocks (Q5, Q8). Exceed one only with a reasoned `#[expect]`.
- No traits, options, crates or abstractions that no current use needs (Q19, W2, W3).

## Tests

- Write the failing test first and watch it fail. A test that passes on its first run must be
  shown able to fail (T1, T12).
- Never delete, rename, ignore, weaken or edit an existing test, or change an expected value to
  match output. If a test looks wrong, stop and report (G5). `cargo xtask guard` flags these.
- Hand-written fakes over mocks; no mocking crate without approval (T4, T5).

## Scope

- Name the placement step before editing and stay in the layers it names (G2, G3).
- Clean up only files the change already touches, within about 10% of the diff (G4).
- Add a dependency only with a stated reason and a version checked with `cargo add` (G8).

## Stop and report when

- a technology change seems to need core or port changes;
- a test conflicts with the requirement, or a fix needs an existing test changed;
- `cargo xtask guard` reports findings;
- a check fails because of a problem outside the change;
- a new driven port has no external actor, or it is unclear which actor owns a rule;
- a dependency or package cannot be verified.

## Commits and PRs

- Conventional Commits (`feat:`, `fix:`, `docs:`, …); the body says why. `committed` checks them.
- The PR description fills every section of `.github/pull_request_template.md` in detail.
- No AI attribution in commits, PRs, code or docs.

## Definition of done

`cargo xtask ci` passes, `cargo xtask mutants` reports no missed mutants, any `cargo xtask guard`
findings are reported to the contributor, and the PR template's checklist holds.
