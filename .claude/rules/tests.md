---
paths:
  - "**/tests/**"
---

# Test rules

The full rules are in `CONVENTIONS.md`.

- Name tests for the behaviour, in domain language: `empty_segments_are_rejected` (T10).
- Core tests go through the public API, with no files, network, processes or real clock; driven
  ports get hand-written in-memory fakes (T3, T4, T9).
- Each driven port has one contract suite: a generic function such as
  `fn collection_store_contract(make: impl Fn() -> Box<dyn CollectionStore>)` in `jukino-core`'s
  `testing` module (behind the `testing` feature, enabled through dev-dependencies), called from
  one `#[test]` per implementation: the fake on every run, real adapters in CI. Change the suite
  before changing the port (T6). The `placing-a-change` skill shows the wiring.
- Driving-adapter tests run the real binary (`env!("CARGO_BIN_EXE_jukino")`) and assert
  translation only (T7).
- Helpers outside `#[test]` functions may use `expect` under a crate-level
  `#![expect(clippy::expect_used, reason = "…")]`.
- Show every new test failing once before it passes (T12).
- Never delete, rename, ignore or edit an existing test, or change an expected value, without the
  contributor's approval (G5). `cargo xtask guard` reports such changes; in CI a contributor
  approves them with the `guard-approved` label.
