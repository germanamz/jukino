# Jukino

Jukino is a modern API interaction tool for humans and agents, written in Rust.

## Behaviours

### Requests in folders

Jukino manages HTTP requests in folders, the way a file system holds files.

### Collections

Collections group folders and requests, as in Postman or Bruno.

### Graph-middleware

Folders and collections can carry graph-middleware written in JavaScript. The
middleware maintains state across a request's execution.

### Chain runs

A chain run executes configured requests in a specified order, like a graph. A script
can join responses before the next request runs, using the graph-middleware.

## Interfaces

- **Humans:** the `jukino` command-line interface first; a user interface (`jukino-ui`)
  is planned.
- **Agents:** an MCP server, and mods for specific harnesses, starting with Claude Code.

## Extending this document

Contributors add to this file as Jukino gains behaviours. Describe what Jukino does for
the person or agent using it. How it is built belongs in [CONVENTIONS.md](CONVENTIONS.md)
and in the design documents under `docs/`.
