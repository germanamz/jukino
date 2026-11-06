## Summary

<!-- What this PR changes, in two or three sentences. -->

## Motivation

<!-- Why the change is needed: the problem, the behaviour from VISION.md, or the ticket in tickets/. -->

## Placement step and layers touched

<!-- The placement step from the placing-a-change skill (0–7), and every crate or layer the diff touches. -->

## Implementation details

<!-- Per crate and module: what changed and how it works. Name new types, ports, adapters and their responsibilities. -->

## Design decisions and rule departures

<!-- Choices made and the alternatives rejected. Every departure from a judgment rule in CONVENTIONS.md, with its reason. -->

## Tests added

<!-- The behaviours the new tests pin down, and how each new test was shown to fail before the fix. -->

## Evidence

<!-- The commands run and their result lines: cargo xtask ci, cargo xtask mutants, and anything else relevant. -->

## Definition of done

- [ ] Placement step named, and the diff stays within those layers.
- [ ] Each new behaviour has a test that was seen failing first.
- [ ] `cargo xtask ci` passes.
- [ ] `cargo xtask mutants` reports no missed mutants.
- [ ] No existing test deleted, ignored, weakened or edited to match output.
- [ ] Every new port or trait is justified, documented and has a test actor.
- [ ] Every new dependency has a stated reason and a verified version.
- [ ] Cleanup stays within touched files and about 10% of the diff.
- [ ] Judgment-rule departures are listed above.
- [ ] The design documents in `docs/` describe the change, and the planned markers it fulfils are removed.
- [ ] Tickets this PR finishes are `completed`.

## Follow-ups

<!-- Work deliberately left for later, each as a ticket in tickets/. -->
