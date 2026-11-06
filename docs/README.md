# Design documents

Each document here describes one area of Jukino as a whole: what the area does, its concepts,
its behaviour, how it is built and why. Together they are the design of the system, both what
is built and what is planned. Planned work is tracked as tickets in [`tickets/`](../tickets/).
The rules for both are K1 to K8 in
[CONVENTIONS.md](../CONVENTIONS.md#design-documents-and-tickets-k).

| Document | Area |
| --- | --- |
| [Workspace](workspace.md) | The crates, their roles, how the architecture maps onto them, and their shared configuration. |
| [Checks](checks.md) | Lints, the `cargo xtask` checks, tests, coverage, mutation testing, supply chain, commit messages, git hooks and CI. |
| [Workflow](workflow.md) | How a change goes from idea to merged pull request: design documents, tickets, agent rules, the review gate, Claude Code hooks and Tusk. |

A new area gets its own document and a row in this table, in the same change that introduces
it. An area is a behaviour from [VISION.md](../VISION.md), such as collections or chain runs,
or a part of how Jukino is built, such as the checks.

## Writing a design document

Every document follows this outline. Leave out a section that has nothing to say.

```markdown
# <Area>

<What the area does and for whom, linking to its behaviour in VISION.md.>

## Concepts

## Behaviour

## Design

## Decisions

## Open questions
```

- **Opening paragraph**: what the area is for, who uses it, and where its edges are.
- **Concepts**: the terms the area uses. Terms from the
  [domain glossary](../CONVENTIONS.md#domain-glossary) link to it instead of being redefined.
- **Behaviour**: what a person or agent observes: rules, limits, edge cases and errors, with
  concrete examples.
- **Design**: how the area is built: the crates, ports, adapters and composition roots it
  touches, the data flow, formats and files, and how its parts fit together.
- **Decisions**: each choice, its reason and the alternatives rejected.
- **Open questions**: what is not decided yet, with the ticket that will decide it when one
  exists.

Write about the system in the present tense. A document has no dates, status lines or change
history, since git and the pull requests keep those. It describes the area whole, so it is
rewritten where the design changes rather than added to at the end.

## Planned parts

A part that is designed but not built yet carries a planned marker where it is described, the
first line of its section or paragraph:

```markdown
> Planned: [Store collections on disk](../tickets/store-collections-on-disk.md)
```

The pull request that builds the part removes the marker and completes the ticket. When the
work is dropped, the same change deletes the ticket and the marker, and rewrites or removes
the part.
