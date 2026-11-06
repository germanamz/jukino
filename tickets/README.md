# Tickets

Each file here is one unit of planned work: a Tusk `ticket` node. The design the work builds
lives in the [design documents](../docs/README.md), and a ticket links to it instead of
repeating it. The rules are K4 to K7 in
[CONVENTIONS.md](../CONVENTIONS.md#design-documents-and-tickets-k).

## A ticket

One ticket is one change the size of a pull request. Name the file for the outcome in kebab
case, such as `tickets/store-collections-on-disk.md`. Its id is the path without `.md`. Larger
work is a parent ticket whose children name it with `parent`. A ticket that must land first
names the tickets it holds up with `blocks`.

```markdown
---
type: ticket
title: Store collections on disk
status: pending
priority: medium
parent: tickets/collections
blocks:
  - tickets/run-a-collection
---

# Store collections on disk

## Goal

What the change achieves and for whom, in a paragraph, linking to the behaviour in
VISION.md.

## Design

Links to the design-document sections that hold the design, for example
[Collections: storage](../docs/collections.md#storage).

## Behaviours to test

- Each behaviour the change adds, named in domain language.
- Existing behaviour that must keep working.

## Done when

- What a reviewer can check to call the work finished.
- The design documents describe the change, and its planned markers are gone.
```

`priority` (`low`, `medium` or `high`), `parent` and `blocks` are optional. `status` is
required.

## Status

| Status | Meaning |
| --- | --- |
| `pending` | Planned. Every ticket starts here. |
| `active` | Work has started. |
| `completed` | Finished, set in the pull request that finishes it. |

A ticket can move from `pending` to `active`, from `active` to `completed` or back to
`pending`, and from `completed` back to `pending` when it is reopened. Tusk checks each move
when it writes the file, so change the status through Tusk rather than by hand. Dropped work is
deleted, along with its planned markers in the design documents.

## With Tusk

| To | MCP tool | CLI |
| --- | --- | --- |
| Create a ticket | `tusk_node_create` with `type: "ticket"`, a `title`, `properties: {status: "pending"}` and the body | `tusk node create tickets/<name>.md --type ticket --title "…" --prop status=pending` |
| Change its status | `tusk_node_modify` with `status` | `tusk node modify tickets/<name> --prop status=active` |
| Add a parent or a blocked ticket | `tusk_edge_add` with type `parent` or `blocks` | `tusk edge add --type parent --source tickets/<name> --target tickets/<parent>` |
| List the open tickets | `tusk_run` with alias `open-tickets` | `tusk run open-tickets` |
| List a ticket's children | `tusk_query` with filter `type=ticket AND parent-> id=tickets/<name>` | `tusk query 'type=ticket AND parent-> id=tickets/<name>'` |
| Find its planned markers | `tusk_query` with filter `names-path=tickets/<name>.md*` | `tusk query 'names-path=tickets/<name>.md*'` |
| Check the rules | `tusk_doctor` | `tusk doctor` |

The trailing `*` in the marker query makes it a glob, which matches the ticket's own path. A
plain path would also match every page that names the `tickets/` folder.
