# Jukino conventions

The rules every contributor follows, human or agent. `AGENTS.md`, the path-scoped rules in
`.claude/rules/` and the `placing-a-change` skill are short extracts of this file. Where a
tool checks a rule, the tool's configuration is the source of truth and this file describes
it.

The research behind these rules (the audit of the hexagonal pattern, the reasons and the
sources for each rule, and the evidence on rule-file size) is the clean-hexagon doc,
"Clean ports and adapters agent rules":
<https://claude.ai/artifact/Ks8VYF57ui3Buoy9nvaMof>. Rule IDs here match that doc. Rules with
IDs the doc does not have (C, R) are specific to Jukino. This file stands on its own for anyone
who cannot open the doc.

## 1. How to read this

- **Absolute (A)** rules have no exceptions unless a contributor records a waiver.
- **Judgment (J)** rules are the default. Depart from one only by writing the reason in the
  PR description, under "Design decisions and rule departures".
- **Dormant** rules apply once the feature they govern exists, for example a transactional
  store. They are listed now so they are ready when it arrives.
- To change a convention, open a PR that edits this file. `cargo xtask guard` treats this file
  as a gate, so CI needs the `guard-approved` label from a contributor.

## 2. Vocabulary

| Term | Meaning in Jukino |
| --- | --- |
| Core | `crates/jukino-core`: every business rule and no technology. No I/O, clock, environment or randomness. |
| Port | A conversation between the core and one external actor, named for its purpose in domain language. |
| Driven port | What the core needs from outside, such as storing a collection or sending a request. A trait declared in the core and implemented by an adapter. |
| Driving port | What the outside asks the core to do. A concrete core type or function that driving adapters call directly. |
| Adapter | A crate under `crates/adapters/`, or the CLI and UI modules, that translates between a port and one technology. |
| Composition root | A binary's `main.rs` (`jukino`, `jukino-ui`): the only code that builds adapters and hands them to the core. |
| Actor | Anything outside the core: a person, an agent, the file system, a remote API, a test. |
| Fake | A working, simplified implementation of a driven port, such as an in-memory store. |
| Contract suite | One generic test suite for a driven port, run against the fake and every real adapter. |
| Interactor | Not used here; the word means different things in Cockburn's and Martin's writing. |

### Domain glossary

One term per concept (Q1). Reuse these words in code, tests and docs.

| Term | Meaning |
| --- | --- |
| Request | An HTTP request Jukino stores and can send. |
| Folder | A named container of requests and other folders, like a file-system directory. |
| Request path | A request's location in the folder tree: folder names and the request name, separated by `/` (`RequestPath`). |
| Collection | A group of folders and requests. |
| Graph-middleware | JavaScript attached to folders and collections that keeps state across a request's execution. |
| Chain run | An execution of configured requests in a specified order, like a graph. |
| Script | JavaScript source text; domain data. The engine that runs it is an adapter. |

## 3. Jukino's hexagon

For most applications HTTP is technology. For Jukino it is the domain. HTTP methods, URLs,
headers, bodies, status codes, folders, collections, middleware graphs and chain runs are
core concepts, written as plain Rust types in `jukino-core`. What stays outside is the
machinery.

| Concern | Lives in | Port (when it arrives) |
| --- | --- | --- |
| Sending a request over the network | driven adapter wrapping an HTTP client library | core trait, e.g. `RequestSender` |
| Reading and writing folders and collections on disk | driven adapter | core trait, e.g. `CollectionStore` |
| Running JavaScript middleware | driven adapter wrapping a JS engine | core trait, e.g. `ScriptRuntime` |
| Clock, randomness, IDs, environment | driven adapter, or values passed in | `Clock` trait or plain arguments |
| CLI | driving adapter in `crates/jukino` | calls core types directly |
| User interface | driving adapter in `crates/jukino-ui` | calls core types directly |
| MCP server | driving adapter (future crate) | calls core types directly |

Crates by role:

- `crates/jukino-core`: the core. Modules are organised by feature inside it.
- `crates/jukino`, `crates/jukino-ui`: composition roots, one per deployable.
- `crates/adapters/<name>`: adapters (none yet).
- `xtask`: repository tooling. Product crates never depend on it.

## 4. Rules

### Boundary and dependencies (B)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| B1 | The core does not depend on adapter crates, composition roots, or any HTTP client, storage driver, JavaScript engine, async runtime, framework or vendor SDK. | A | `cargo xtask arch` (empty allow-list) |
| B2 | Driven ports are traits in the core, implemented by adapters. The core never names a concrete adapter type. | A | `cargo xtask arch` |
| B3 | Driving adapters may call core types and functions directly. The core never depends on them. | A | `cargo xtask arch` |
| B4 | Adapters do not depend on each other, except couplings declared in `arch.toml` with a reason. | A | `cargo xtask arch` |
| B5 | Each deployable has one composition root, the only code that constructs driven adapters (tests excepted). It builds driven adapters, then the core, then driving adapters. | A | `cargo xtask arch` (only roots depend on adapters) |
| B6 | Everything between the core and the outside passes through a port: files, environment, network, processes, threads, clock and randomness. | A | core `clippy.toml` disallowed lists; `cargo xtask arch` source scan |
| B7 | Port signatures use core types only. No client-library, storage, engine or framework types. | A | follows from B1 |
| B8 | The core does not reach through a port's return value into technology objects. | A | follows from B7; review |
| B9 | Every new boundary gets an `arch.toml` entry. The arch check fails when a configured crate is missing, so it cannot pass vacuously. | A | `cargo xtask arch` |
| B10 | Existing violations, if there ever are any, are frozen in `arch.toml` with a reason, never hidden by disabling a rule. No new ones. | A | review; `cargo xtask guard` |

### Ports (P)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| P1 | Port and operation names use domain language and never name a library: `CollectionStore::save`, not `SqliteCollectionDao`. HTTP words (method, header, status) are domain language here; a client library's name is not. | A | review |
| P2 | Design ports from the external actors: start with one port per actor and split only with a stated reason. A driven port maps to an external system, not to a domain noun. | J | review |
| P3 | One port may hold several traits. Do not create a port per entity. | J | review |
| P4 | A new driven port needs an external actor (I/O, time, randomness, a third party) or a second implementation that exists now; a test fake counts. | A | review |
| P5 | Every port trait and method documents inputs and constraints, outputs, failures (`# Errors`), side effects and invariants, without restating the name. | A | `missing_docs`, `missing_errors_doc`; review |
| P6 | Every port has a test actor: a fake for a driven port, a test driver for a driving port. | A | review; contract suite (T6) |
| P7 | Ports live in `jukino-core`. | A | `cargo xtask arch` |
| P8 | Driving ports are concrete core types or functions, not single-implementation traits. | A | review |
| P9 | A domain change may change ports; a technology change must not. If one seems to, stop and report (G7). | A | review |

### Adapters (A)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| A1 | Adapters only translate: parsing, serialisation, mapping, protocol details, error translation and resilience. No business decisions. | A | review |
| A2 | A driving adapter turns input into a core input, calls one core operation per request or command, and maps the result and errors to its transport. | A | review; adapter tests (T7) |
| A3 | Timeouts, retries, authentication headers, pagination and rate limiting live in the driven adapter. | A | review |
| A4 | Vendor types and field names stay inside the adapter and never cross a port. | A | follows from B1, B7 |
| A5 | When a foreign model already matches the core's language, keep the adapter thin. | J | review |
| A6 | Long field-by-field mapping functions may exceed the length limit under `#[expect(clippy::too_many_lines, reason = "field-by-field mapping")]`. | J | clippy |
| A7 | Similar code in two adapters, or in an adapter and a fake, is not deduplicated through the core. | A | review |

### Core code quality (Q)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| Q1 | One term per concept across core, ports and tests. Check the glossary and the code before naming, and reuse the existing term. | A | review |
| Q2 | Follow Rust naming (RFC 430): types are noun phrases in `UpperCamelCase`, functions with effects are verb phrases, booleans read as predicates (`is_`, `has_`, `can_`). | A | rustc lints; review |
| Q3 | Names used beyond one function are searchable: no single letters, no bare `data`, `info`, `item`, `manager`, `util`. Loop indices and short closures are exempt. | A | review |
| Q4 | Core logic has no unexplained numeric literals other than 0, 1 and -1; name them as constants. | A | review |
| Q5 | Functions stay within 30 lines, cognitive complexity 15 and three nested blocks. Exceed a limit only with `#[expect(…, reason = "…")]`. | A | clippy (`too_many_lines`, `cognitive_complexity`, `excessive_nesting`) |
| Q6 | Aim for 20 lines or fewer, but never split a function only to meet a line count. | J | review |
| Q7 | Extract a block only if it can be named without "and/or/then", understood without its caller, and needs 3 parameters or fewer and no new fields. Inline a single-use helper of 2 lines or fewer whose name adds nothing. | J | review |
| Q8 | At most `self` plus 3 parameters; more need a parameter struct named for a domain concept. No boolean flag parameters in the core, no output parameters. | A | clippy (`too_many_arguments`, `fn_params_excessive_bools` with `max-fn-params-bools = 0` in the core) |
| Q9 | A function either changes state or returns information, not both. Idiomatic exceptions (`pop`, `save` returning an ID) are named for it. | A | review |
| Q10 | A function that calls a port or domain operation does not also do raw parsing, formatting or index arithmetic. | J | review |
| Q11 | In a file, public items come first, then helpers in the order they are first called. | A | review |
| Q12 | Each core module serves one actor or business capability, stated in its `//!` doc in one sentence. | J | review |
| Q13 | A core type is either a value object or entity (private fields, behaviour that enforces invariants) or a plain data record (public fields, no business behaviour). Never both. | A | review |
| Q14 | Choose deliberately between an enum with exhaustive `match` (new operations expected) and a trait (new variants or plugins expected), and say which in the PR. No `_ =>` arms on enums. | J | clippy (`wildcard_enum_match_arm`); review |
| Q15 | Do not call methods on an entity or service obtained from a collaborator (`a.b().c()`). Same-type builders, value objects and iterator chains are fine. | J | review |
| Q16 | Each domain fact (rule, constant, validation, mapping) has one authoritative definition in the core. | A | review |
| Q17 | Do not abstract structurally similar code before its third occurrence, unless it is the same knowledge (Q16). | J | review |
| Q18 | If a change would add a mode flag or a caller-specific branch to a shared helper, inline the helper back into its callers instead. | J | review |
| Q19 | Add no trait, parameter, configuration option or extension point that no current use or test exercises. Driven ports are exempt. | A | review |
| Q20 | Comments say only why, or warn. No comments that paraphrase code, no commented-out code, no change logs or authorship notes. Update or delete a comment when the behaviour it describes changes. | A | `cargo xtask comments`; review |
| Q21 | All code passes `cargo fmt` and clippy with zero warnings. The workspace is always fully formatted. | A | `cargo xtask ci`; hooks |
| Q22 | Where a local convention conflicts with this file, follow the local convention and report the conflict. B rules always win. | J | review |

### Comments and documentation (C)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| C1 | No comment blocks: two or more consecutive lines holding only a `//` comment fail. | A | `cargo xtask comments` |
| C2 | No block comments (`/* */`, `/** */`, `/*! */`). Use `///` and `//!`. | A | `cargo xtask comments` |
| C3 | No commented-out code (a `//` comment ending in `;`, `{` or `}`). | A | `cargo xtask comments` |
| C4 | Every public item has a doc comment. Fallible functions have `# Errors`, panicking ones `# Panics`. Intra-doc links resolve. | A | `missing_docs`, clippy pedantic, `cargo doc` with `-D warnings` |
| C5 | An explanation longer than one line goes in the doc comment of the item it explains. | J | review |

### Errors (E)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| E1 | Core functions never signal failure with sentinels (-1, empty strings, magic codes). | A | review |
| E2 | Expected outcomes the caller must handle are in the signature as `Result<T, E>` with an error enum per port or use case, implementing `Display` and `core::error::Error`. The failure set is readable from the enum and its `# Errors` docs. | A | review; `missing_errors_doc` |
| E3 | Bugs and broken invariants panic and are documented under `# Panics`; they are never domain results. | A | `missing_panics_doc`; review |
| E4 | No `unwrap` or `expect` outside tests, no ignored `Result`s, no empty error arms without a comment saying why. | A | clippy (`unwrap_used`, `expect_used`), `unused_must_use` |
| E5 | The core never names technology error types. | A | follows from B1 |
| E6 | Driven adapters translate technology failures with domain meaning into core errors, keeping the original as the source. Unexpected failures surface as one technology-neutral error. | A | adapter contract tests (T6) |
| E7 | Driving adapters map core errors to their transport: CLI exit codes, MCP error responses. Transport codes never appear in the core. | A | review; adapter tests |
| E8 | Use `Option` for legitimate absence. Use a special-case type only when "nothing" has real behaviour. | J | type checker; review |
| E9 | When validation must report several problems, collect them instead of failing on the first. | J | review |
| E10 | Before adding an error case, check whether the operation can be defined so the error cannot happen. | J | review |

### Tests (T)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| T1 | New behaviour starts with a test that fails for the right reason; a compile failure counts. Write it against the driving port. | A | review of the red run (D7) |
| T2 | Before coding, write the list of behaviours to test, including existing ones that must keep working. | J | plan or PR |
| T3 | Core tests run without files, network, processes or the real clock. Driven ports get in-memory fakes. | A | core dependency allow-list and clippy lists also apply to tests |
| T4 | Replace stateful driven ports with hand-written fakes and assert outcomes. Spies may record fire-and-forget calls. | J | review |
| T5 | Never mock types you do not own; wrap the library behind a port and fake the port. No mocking crate without approval. | A | review; `arch.toml` and dependency review |
| T6 | Each driven port has one contract suite, a generic test function run against the fake on every run and against each real adapter in CI. Suites and fakes live in `jukino-core`'s `testing` module behind the `testing` feature, which the core's tests (through a dev-dependency on the crate itself) and adapter tests enable. Change the suite before changing the port. A fake has no behaviour the suite does not pin. | A | review; CI |
| T7 | Driving-adapter tests go through the real binary or framework and assert translation only. | A | review |
| T8 | One or two smoke tests boot each real composition root. | J | `crates/*/tests/cli.rs` |
| T9 | Test through the driving port; core tests live in `crates/jukino-core/tests/`. Unit tests inside modules are for dense pure logic only. No test file per type by default. | J | review |
| T10 | Test names describe behaviour in domain language (`empty_segments_are_rejected`). | A | review |
| T11 | One concept per test, arranged, acted and asserted; readability over reuse. | J | review |
| T12 | A new test that passes on its first run is suspect: show it can fail by breaking the code once. | A | `cargo xtask mutants`; PR evidence |
| T13 | Refactor only on green, separately from behaviour changes. | A | review |

Coverage minimums (`cargo xtask coverage`): workspace lines 90%, workspace functions 90%,
`jukino-core` lines 95%. Only `xtask/src/runner.rs`, which launches external tools and makes
no decisions, is excluded. Mutation testing (`cargo xtask mutants`) must report no missed
mutants in the diff.

### Cross-cutting concerns (X)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| X1 | Core logic never calls `now()`, random sources or ID generators directly. Pass the value in, or inject a `Clock` or ID port. | A | core `clippy.toml` (`SystemTime::now`, `Instant::now`) |
| X2 | Technical logging, metrics and tracing live in adapters and composition roots, not behind a logging port. The core logs nothing for now; adding a logging facade to it needs an allow-list entry. | J | `arch.toml` |
| X3 | Business-meaningful observations are domain events or results, never metric names or analytics calls in the core. | A | `cargo xtask arch` |
| X4 | Configuration is read only in composition roots or adapters and reaches the core as typed values. | A | core `clippy.toml` (`std::env`) |
| X5 | Adapters add correlation IDs and structured logs at translation points. | J | review |

Authorisation has no rule yet; it is decided when the first feature needs it.

### Validation and mapping (V, M)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| V1 | Driving adapters reject malformed input (types, required fields, formats) and then build a core input. | A | adapter tests |
| V2 | Core input types may validate their own structure on construction. | J | review |
| V3 | Rules that need state live in the core. Value objects and entities enforce their invariants in their constructors (`RequestPath::parse`) and never trust an adapter's validation. | A | core tests |
| V4 | The same check at the edge (for good messages) and in the core (as an invariant) is not duplication. | J | review |
| M1 | External contracts (the MCP server's API, on-disk file formats) never expose core types directly; adapters own their DTOs. | A | review |
| M2 | State-changing operations take a dedicated input type; read-only queries may return a read model. | J | review |
| M3 | Each adapter owns its DTOs and maps both ways at its boundary (D3). Do not add a mapping layer in an unrelated change. | A | review |
| M4 | Core types carry no framework, storage or serialisation attributes (no `serde` derives); formats are adapter DTOs (D2). | A | `cargo xtask arch` (no `serde` in the core) |
| M5 | Entities never call ports to complete a rule; the use case loads the data and passes it in. | A | review |

### Transactions and events (TX, EV): dormant

These apply once a transactional store or a message broker exists.

| ID | Rule | Type |
| --- | --- | --- |
| TX1 | The transaction boundary is the use case. | A |
| TX2 | Store adapters do not commit when a use case may combine their writes. | A |
| TX3 | Prefer one aggregate per transaction; record why when a use case changes two. | J |
| TX4 | External side effects never share a database transaction's atomicity; use an outbox or compensating steps. | A |
| EV1 | Domain events are plain data in the core; aggregates record them and do not publish them. | A |
| EV2 | Events other systems rely on go through an outbox in the same transaction, unless a loss window is accepted and recorded. | A |
| EV3 | Message consumers are driving adapters and are idempotent. | A |
| EV4 | In-process event dispatch within one transaction is allowed for side effects in the same context. | J |

### When not to apply the full pattern (W)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| W1 | Before adding ports for a feature, check that it has real domain rules. A feature that only moves fields gets the simplest structure that keeps technology out of whatever rules exist. Say which in the PR. | J | PR (D17) |
| W2 | No trait with a single implementation unless it is a driven port (P4). | A | review |
| W3 | No layer or type whose methods only delegate. | A | review |
| W4 | Scripts and prototypes skip the hexagon; extract a port when the second adapter or the first fake appears. | J | review |
| W5 | No hexagons inside hexagons. | A | review |
| W7 | Do not impose global layers; feature modules inside the core share only genuine domain rules. | A | review |
| W8 | Crates by role; modules by feature inside the core. Cross-feature calls go through the other feature's public API. | J | review |

W6 in the doc is specific to Go and does not apply.

### Agent and contributor process (G)

| ID | Rule | Type | Check |
| --- | --- | --- | --- |
| G1 | Read a port, adapter or core module before using or changing it. Never guess an API. | A | review |
| G2 | Before editing, name the placement step (section 5) and the layers you will touch. | A | PR |
| G3 | Keep the diff within those layers; stop and report before going outside them. | A | review |
| G4 | Clean up only files the change already touches, preserving behaviour, within about 10% of the diff or one separate commit (D16). | A | review |
| G5 | Never delete, skip, ignore or weaken a test or assertion, or edit an expected value to match output. If a test looks wrong, stop and report with evidence. | A | `cargo xtask guard` |
| G6 | Never special-case test inputs or the test environment. | A | review; `cargo xtask mutants` |
| G7 | Stop and report instead of working around: a technology change needs core or port changes; a test conflicts with the requirement; a check fails because of something outside the change; a fix needs a test changed; a new driven port has no external actor; it is unclear which actor owns a rule; a dependency cannot be verified. | A | review |
| G8 | Add a dependency only with a stated reason and a version checked with `cargo add`. | A | `cargo deny`, `cargo machete`, review |
| G9 | Never bypass checks: no `--no-verify`, no disabled hooks, no `#[allow]`, no loosened thresholds without approval. | A | Claude Code hooks; `allow_attributes`; `cargo xtask guard` |
| G10 | Do not add docs or comments to code you did not change, or helpers and flexibility the task does not need. | A | review |
| G11 | Finish with evidence: the commands run and their results, in the PR description. | A | PR template |
| G12 | Act on review findings about correctness, requirements and rules, not on every stylistic suggestion. | J | review |

## 5. Placement procedure

Run this before any edit. The first step that matches wins. The `placing-a-change` skill adds
worked examples and the "add a driven port" and "swap a technology" workflows.

0. **Complexity gate.** If the feature only moves fields with no domain rule, use the simplest
   structure that keeps technology out of the rules that exist (W1), then go to the definition
   of done.
1. **A business rule, invariant, calculation or flow changes.** It goes in `jukino-core`:
   value objects for invariants, use cases for orchestration. Test it through the core's
   public API with fakes (T1, T3). Ports change only if the conversation changes (P9).
2. **The core needs something new from outside** (store, send, run a script, the time). If an
   existing driven port's conversation covers it, add an operation to that port; otherwise
   create a port named for the purpose (P1, P4) with full docs (P5). Then change the contract
   suite first, then the fake, then the real adapter (T6).
3. **A new entry point uses existing behaviour** (CLI command, MCP tool, UI action). It goes
   only in a driving adapter: build a core input, call one core operation, map results and
   errors (A2, E7, V1). Test it through the real binary (T7).
4. **A technology swap or upgrade** (HTTP client, storage format, JS engine). It goes only in
   the adapter and the composition root, and the contract suite must pass unchanged. Any core,
   port or core-test change means stop and report (G7).
5. **The core wants a third-party library.** A pure-computation library may go on `arch.toml`'s
   allow-list with a reason and a contributor's approval. Anything with I/O goes behind a port.
6. **A cross-cutting concern.** Time and randomness: X1. Logging: X2. Business observations:
   X3. Configuration: X4. Retries and timeouts: A3.
7. **Wiring only.** It goes in a composition root, with a smoke test if the path is new (T8).

**Escape hatch.** If the change fits no step, or two in conflict, state the ambiguity and the
choice you made, or stop and report when the choice would add a port or cross a boundary.

## 6. Commits, PRs and review

| ID | Rule |
| --- | --- |
| R1 | **Review gate.** When a change is ready, the agent says so in one line and stops. It never pastes diffs into chat. The contributor reviews in their editor; comments arrive as normal input, or on the PR once it exists. Review comments are not approval. The agent commits, pushes or opens a PR only on a direct instruction naming that step; "commit" does not cover pushing. This holds for every commit. Claude Code asks permission before every pull-request creation or merge, and before the first commit or push after each contributor message, whatever the command's form (`git -C`, a full path, a subshell): the `guard-bash.sh` hook asks, and permission rules in `.claude/settings.json` also ask for the plain pull-request forms. Approving that first commit or push opens a wave, recorded by `review-wave.sh`, in which later commands made only of `cd`, `git add`, `git commit` and `git push` run without a prompt until the contributor's next message. The guard blocks the command if the hook itself fails. |
| R2 | **Guard label.** `cargo xtask guard` reports changes to existing test code (`#[test]` functions anywhere; in `tests/` files and `#[cfg(test)]` modules also helpers and file and module attributes), gate files, rule texts, lint tables and new `#[expect]` attributes. Locally it warns and the agent stops and reports. In CI it fails until a contributor adds the `guard-approved` label. Agents never add that label. |
| R3 | **Conventional Commits.** Types `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`; optional scope; subject in the imperative, at most 72 characters, lowercase, no trailing period; no `fixup!` or WIP commits; no merge commits. The body explains why. Checked by `committed`. |
| R4 | **PR descriptions** fill every section of `.github/pull_request_template.md` in detail, including evidence and rule departures. The PR title follows R3; squash merging makes it the commit. |
| R5 | **No AI attribution** in commits, PRs, code or docs: no AI co-author trailers, session trailers or links, or "generated with" lines. Checked by `cargo xtask attribution`. |

## 7. Enforcement map

| Check | Command | Rules |
| --- | --- | --- |
| Formatting | `cargo fmt --all --check` | Q21 |
| Workspace lints and limits | `cargo clippy --workspace --all-targets -- -D warnings` | Q5, Q8, Q14, C4, E2–E4, G9 |
| Core restrictions | `crates/jukino-core/clippy.toml` (same clippy run) | B6, X1, X4, Q8 |
| Docs | `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` | C4 |
| Comments | `cargo xtask comments` | C1–C3, Q20 |
| Architecture | `cargo xtask arch` (also: every crate sets `[lints] workspace = true`) | B1–B7, B9, P7, X3, M4, Q21 |
| Guard | `cargo xtask guard` (CI: the PR checks workflow, `--ci --labels …`) | G5, G9, B10, R2 |
| Tests | `cargo nextest run --workspace`, `cargo test --workspace --doc` | T rules |
| Coverage | `cargo xtask coverage` | section 4, Tests |
| Mutation | `cargo xtask mutants` | T12, G6 |
| Supply chain | `cargo deny check`, `cargo machete` | G8 |
| Commit messages | `committed`, `cargo xtask attribution` (commits, PR title and description) | R3, R5 |
| Line endings | `.gitattributes` (LF on every platform) | Q21 |
| Git hooks | `lefthook.yml` | Q21, C1–C3, R3, R5 |
| Claude Code hooks | `.claude/settings.json` | R1, G9, C1–C3, Q21 |
| Everything, in order | `cargo xtask ci`; GitHub Actions `.github/workflows/ci.yml` and `.github/workflows/pr-checks.yml` | all of the above |

## 8. Resolved decisions

The research left eighteen decisions open. Each takes the doc's default, translated to Rust.
Change one by PR (section 1).

| # | Decision | Resolved as |
| --- | --- | --- |
| D1 | Numeric limits | 30 lines, cognitive complexity 15, three nested blocks, `self` plus 3 parameters; violations fail the build. Clippy counts `self` as a parameter and counts the function body plus any enclosing `impl` or inline `mod` as nesting levels, so `clippy.toml` sets `too-many-arguments-threshold = 4` and `excessive-nesting-threshold = 5`. Free functions therefore get one extra parameter and one extra nesting level. |
| D2 | Framework attributes on domain types | Core types stay free of storage and serialisation concerns; on-disk formats are adapter DTOs mapped to core types. This includes `serde` derives. |
| D3 | Mapping strategy | Each adapter owns its DTOs and maps both ways at its boundary. |
| D4 | Error mechanism | `Result` with per-port or per-use-case error enums; panics for bugs only. |
| D5 | Port naming | Role nouns in domain language (`CollectionStore`, `ScriptRuntime`), never library names. |
| D6 | Driving-port form | Concrete core types; no single-implementation traits. |
| D7 | TDD strictness | Canon TDD: a test list, red before green, the red run observed. Commit order cannot be enforced because commits wait for the review gate. |
| D8 | Logging | No logging port. Adapters and composition roots log; business observations are domain events if needed. |
| D9 | Internal core structure | None prescribed beyond feature-first modules. |
| D10 | Package layout | Crates by role; modules by feature inside the core. |
| D11 | Cross-aggregate transactions | Not applicable until a transactional store exists. |
| D12 | Event reliability | Not applicable until a message broker exists. |
| D13 | Comment policy | Interface docs on all public items and ports, plus the no-comment-blocks rule (C1–C5). |
| D14 | Mocks | Hand-written fakes; spies for fire-and-forget calls; no mocking crate without approval. |
| D15 | Test changes during implementation | Agents may add tests but not change existing ones. `cargo xtask guard` warns locally and fails in CI without the `guard-approved` label. |
| D16 | Cleanup budget | About 10% of the diff, in files the change touches. |
| D17 | Complexity gate | The agent decides per feature and states it in the PR. |
| D18 | Emphasis in agent files | Plain imperative wording; one `IMPORTANT`, on the review gate. |
