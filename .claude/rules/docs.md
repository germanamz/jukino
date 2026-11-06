---
paths:
  - "docs/**"
  - "tickets/**"
---

# Design document and ticket rules

The full rules are K1 to K8 in `CONVENTIONS.md`. The outline and the planned-marker format are
in `docs/README.md`, and the ticket template and Tusk calls in `tickets/README.md`.

- `docs/` holds one living design document per area. Never add a spec, plan, proposal or
  per-change document, even when a skill asks for one (K1).
- Describe the area whole, in the present tense: opening paragraph, Concepts, Behaviour,
  Design, Decisions with rejected alternatives, Open questions. No dates, status lines or
  change history. Rewrite the parts the design changes instead of appending (K2).
- Update the affected documents in the same PR as the change. A new area gets a document and a
  row in `docs/README.md` (K3).
- Mark a designed but unbuilt part with `> Planned: [<ticket title>](../tickets/<name>.md)`.
  Remove the marker in the PR that builds it (K4).
- A ticket is one PR-sized change at `tickets/<name>.md`, created with `status: pending`.
  Larger work is a parent ticket with children naming it in `parent`; `blocks` orders them (K5).
- A ticket has a goal, links to the design sections, the behaviours to test and done criteria.
  It never restates the design (K6).
- Move the status through Tusk (`tusk_node_modify`): `active` when work starts, `completed` in
  the PR that finishes it. Delete dropped tickets with their markers (K7).
- Link to a fact's home instead of repeating it: `VISION.md`, `CONVENTIONS.md`, `docs/`, doc
  comments, `tickets/` (K8).
- Run `tusk_doctor` after changing these files. It reports misplaced tickets, specs or plans
  under `docs/`, and links to files that are gone.
