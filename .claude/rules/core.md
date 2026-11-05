---
paths:
  - "crates/jukino-core/**"
---

# Core rules

The full rules are in `CONVENTIONS.md`; these are the ones that differ most from defaults.

- Expected failures are `Result<T, E>` with an error enum per port or use case, implementing
  `Display` and `core::error::Error`, with every variant documented and listed under `# Errors`
  (E2). Bugs panic, documented under `# Panics` (E3). No sentinel values (E1).
- Value objects and entities keep fields private and enforce invariants in their constructor
  (`parse` or `new` returning `Result`), as `RequestPath::parse` does. Never rely on an adapter
  having validated (V3).
- A type is a value object or entity with behaviour, or a plain data record with public fields
  and no behaviour. Never both (Q13).
- Time, IDs and randomness come in as arguments or through a port (X1). Configuration arrives as
  typed values from the composition root (X4).
- One term per concept: check the glossary in `CONVENTIONS.md` and the code before naming (Q1).
- Choose between an enum with exhaustive `match` and a trait by the change you expect, and say
  which in the PR (Q14). No `_ =>` arms on enums.
- Entities never call ports; the use case loads data and passes it in (M5).
- Ports are named for their purpose in domain language (`CollectionStore`, `RequestSender`,
  `ScriptRuntime`): one per external actor, never one per entity (P1–P4).
- No `serde` or other framework attributes on core types (M4). The core's external dependency
  allow-list in `arch.toml` is empty; adding to it needs a reason and a contributor's approval.
