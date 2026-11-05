---
name: placing-a-change
description: Use before editing code in Jukino, when planning any change. Decides where the change belongs (core, port, adapter, composition root), which tests prove it, and when to stop and report. Includes the workflows for adding a driven port with its adapter and for swapping a technology.
---

# Placing a change

Name the step you chose, and the crates it lets you touch, before the first edit (G2). Keep the
diff inside them (G3). The full rules are in `CONVENTIONS.md`; IDs below point there.

## Steps (the first match wins)

**0. Complexity gate.** Does the feature have a real domain rule: an invariant, a state change
or a calculation beyond copying fields? If not, use the simplest structure that keeps technology
out of whatever rules exist (W1), and say so in the PR.

**1. A business rule, invariant, calculation or flow changes.** Edit `crates/jukino-core` only.
Invariants go in the value object's constructor; orchestration goes in a use-case function or
type. Test through the public API in `crates/jukino-core/tests/` with fakes.

> Example: "a request path may not contain `..`". Add the case to `RequestPath::parse` and a
> test such as `dot_segments_are_rejected`. No adapter changes.

**2. The core needs something new from outside** (storage, the network, a script engine, the
time). Look for a driven port whose conversation already covers it and add an operation there.
Otherwise create one: a trait in the core named for the purpose (`CollectionStore`, not
`TomlFileStore`), documented in full (P5), justified by an external actor (P4). Then follow
"Add a driven port and its adapter" below.

**3. A new entry point for existing behaviour** (a CLI subcommand, an MCP tool, a UI action).
Edit only the driving adapter: parse and check the input's shape, build the core input, call one
core operation, map the result and every error to the transport (exit code, MCP error) (A2, E7,
V1). Test it by running the real binary (T7).

**4. A technology swap or upgrade** (HTTP client, file format, JavaScript engine). Follow "Swap
a technology" below. Only the adapter and the composition root change.

**5. The core wants a third-party library.** A pure-computation library (no I/O, no runtime)
can go on `arch.toml`'s core allow-list with a reason, after the contributor approves. Anything
else goes behind a port.

**6. A cross-cutting concern.** Time and IDs: pass them in or use a port (X1). Logging: adapters
and composition roots (X2). Configuration: read in the composition root, passed as typed values
(X4). Retries and timeouts: the driven adapter (A3).

**7. Wiring only.** Edit the composition root (`main.rs`), with a smoke test if the path is new
(T8).

**Escape hatch.** If nothing fits, or two steps conflict, write down the ambiguity and your
choice. If the choice would add a port or cross a boundary, stop and report instead (G7).

## Workflow: add a driven port and its adapter

1. Write the trait in the core, documenting inputs, outputs, `# Errors`, side effects and
   invariants. Its error type is a core enum (E2).
2. Write the contract suite first (T6). It lives in `jukino-core`'s `testing` module, compiled
   only with the crate's `testing` feature (`#[cfg(feature = "testing")] pub mod testing;`), so
   both the core's tests and every adapter's tests can call it:

   ```rust
   /// Behaviour every `CollectionStore` must have.
   pub fn collection_store_contract(make: impl Fn() -> Box<dyn CollectionStore>) {
       let store = make();
       assert_eq!(store.save(&a_collection("orders")), Ok(()));
       assert_eq!(store.load(&name("orders")), Ok(a_collection("orders")));
   }
   ```

   The core's own tests enable the feature through a dev-dependency on the crate itself,
   `jukino-core = { path = ".", features = ["testing"] }`; `cargo xtask arch` allows a crate's
   dependency on itself.
3. Write the in-memory fake in the same `testing` module until the suite passes, called from a
   test in `crates/jukino-core/tests/`. The fake has no behaviour the suite does not pin down.
4. Create the adapter crate under `crates/adapters/<name>/`, implementing the trait, with
   `[lints] workspace = true`. It maps vendor types to core types and translates errors, keeping
   the source (A4, E6). Its tests enable the suite with
   `jukino-core = { path = "../../jukino-core", features = ["testing"] }` in
   `[dev-dependencies]` and run it against the real adapter.
5. Wire it in the composition root: driven adapters first, then the core, then driving adapters
   (B5).
6. Run `cargo xtask arch`: it confirms the new crate is classified and depends only on the core.

## Workflow: swap a technology

1. Write the new adapter beside the old one, in its own crate under `crates/adapters/`.
2. Run the port's existing contract suite against it unchanged.
3. Switch the composition root to the new adapter and delete the old one.
4. If a core file, a port or a core test needs to change, stop and report: the old adapter leaked
   technology into the core (P9, G7).
