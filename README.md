# Jukino

A modern API interaction tool for humans and agents, written in Rust. Requests live in folders
and collections, JavaScript graph-middleware keeps state across a request's execution, and chain
runs execute requests in order like a graph. See [VISION.md](VISION.md).

Jukino is at its foundation stage: the repository has its checks and conventions, and product
features come next.

## Prerequisites

- Rust through [rustup](https://rustup.rs); `rust-toolchain.toml` pins the version and components.
- The cargo tools the checks use:

  ```sh
  cargo install --locked cargo-nextest cargo-llvm-cov cargo-deny cargo-machete cargo-mutants committed
  ```

- [lefthook](https://github.com/evilmartians/lefthook) for git hooks, and `jq` for the Claude
  Code hooks.
- Optional, for agents: [Tusk](https://github.com/germanamz/tusk) and [Ollama](https://ollama.com)
  with the embedding model (`ollama pull nomic-embed-text`). Without Ollama running, Tusk still
  indexes and answers structural queries; semantic queries fail with a connection error, and the
  embeddings stay queued until Ollama is back.

## Setup

```sh
lefthook install          # pre-commit, commit-msg and pre-push hooks
tusk claude install       # optional: the Tusk mod for Claude Code sessions
```

## Commands

| Command | What it does |
| --- | --- |
| `cargo xtask ci` | Every check, in the order CI runs them |
| `cargo nextest run --workspace` | The tests |
| `cargo xtask coverage` | Tests under coverage, with the minimums enforced |
| `cargo xtask mutants` | Mutation testing of the code changed since `origin/main` |
| `cargo xtask comments [paths]` | The no-comment-blocks check |
| `cargo xtask arch` | Crate dependency and core import rules |
| `cargo xtask guard` | Changes to existing tests and gates that need a contributor's approval |
| `cargo xtask attribution --file <msg>` | Rejects AI attribution in a commit message |

## Contributing

Read [CONVENTIONS.md](CONVENTIONS.md). Agents follow [AGENTS.md](AGENTS.md), its short form.
Commits follow Conventional Commits, and pull requests fill in
`.github/pull_request_template.md`. When a pull request changes existing tests or gate
configuration, CI waits for a contributor to add the `guard-approved` label.

Recommended repository settings, for a contributor with admin access:

- Squash merging only, with "Pull request title and description" as the default message, so
  the checked title and description become the commit on `main`.
- Branch protection on `main` requiring these checks: `lint`, `test (ubuntu-latest)`,
  `test (macos-latest)`, `test (windows-latest)`, `coverage` and `mutants` from CI, and `guard`
  and `commits` from PR checks.
- Update pull-request branches by rebase, not merge: merge commits fail the `commits` check.

## License

[Apache-2.0](LICENSE)
