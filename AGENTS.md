# Agent Guide for `why` Database Entries

This file helps coding agents (Claude, Copilot, Cursor, etc.) create correct `why` database entries. Read this before generating YAML files.

## Project context

`why` is a CLI tool that captures stderr from failed commands and explains the error in plain English. The error database lives in `db/` as YAML files — one file per error, organized by tool.

## Directory structure

```
db/
  TEMPLATE.yaml       # universal template — copy this
  rust/               # rustc compiler errors (E0499.yaml)
  python/             # Python exceptions (TypeError.yaml)
  c_cpp/              # gcc/clang errors (undefined-reference.yaml)
  go/                 # Go compiler errors (undefined.yaml)
  git/                # git CLI errors (merge-conflict.yaml)
  docker/             # Docker CLI errors (daemon-not-running.yaml)
  npm/                # npm CLI errors (eresolve.yaml)
  cargo/              # Cargo build errors (could-not-compile.yaml)
  why/                # errors from the why tool itself
```

To add a new tool, just create the directory. No code changes needed.

## Creating an entry

### Filename

- Must match the `id` field exactly: `id: merge-conflict` → `merge-conflict.yaml`
- Rust errors use the error code: `E0499.yaml`
- Python errors use the exception name: `TypeError.yaml`
- Everything else uses lowercase slugs: `daemon-not-running.yaml`

### Required fields

```yaml
id: merge-conflict # matches filename without .yaml
tool: git # the CLI/compiler that produces this error
language: git # matches the parent directory name
title: Merge conflict # under 60 characters
explain: | # plain English, 3-6 sentences
  ...
fix: | # concrete steps, 2-4 actions
  ...
```

### Patterns (required for auto-detection)

The `patterns` field is how `why` matches stderr output to this entry. Each pattern is a list of substrings that must ALL appear in the same stderr line (AND logic). Multiple patterns give OR logic.

```yaml
patterns:
  - ["Automatic merge failed"] # matches this exact substring
  - ["CONFLICT", "Merge conflict"] # both must appear in same line
  - ["fix conflicts and then commit"] # OR this matches
```

Rules:

- Use the most specific substring possible — avoid single common words
- Copy-paste from real error output when possible
- No regex — plain substring matching only
- Patterns are case-sensitive
- Each pattern group is checked against each line independently
- First match wins across the entire database, so be specific enough to avoid collisions

When two tools produce similar errors, use `exclude`:

```yaml
# C/C++ "too many arguments" would also match Go errors
patterns:
  - ["too many arguments"]
exclude:
  - "go"
```

Tools with structured error codes (`rustc`, `python`) don't need patterns — detection works via regex on the error code / exception name.

### Optional fields

```yaml
tags: [merge, conflict, branches] # for search/filtering
exclude: ["text that must NOT appear"] # avoid false matches
example_error: | # real error output
  Auto-merging src/main.rs
  CONFLICT (content): Merge conflict in src/main.rs
example_code: | # minimal reproduction
  // code that triggers the error
links: # official docs
  - https://git-scm.com/docs/git-merge
```

## Style rules

These are strict — entries that violate them need revision:

1. **`explain` uses second person.** "You tried to..." not "The user tried to..."
2. **`explain` is plain English.** If you use a technical term, define it briefly.
3. **`fix` gives concrete commands/steps.** "Run `git pull --rebase`" not "update your branch."
4. **No filler.** Don't start with "This error occurs when..." — just explain what happened.
5. **No emoji.** Anywhere.
6. **`language` must match the directory name exactly.** `db/c_cpp/` → `language: c_cpp`, not `language: cpp`.
7. **`id` must match the filename exactly.** No exceptions.

## Validation

After creating entries, run:

```sh
python scripts/validate.py
```

This checks: required fields, id/filename match, language/directory match, patterns structure, duplicate IDs, unknown fields, and empty values.

Also run the Rust tests to make sure detection still works:

```sh
cargo test
```

## Common mistakes to avoid

- Setting `language: cpp` when the directory is `c_cpp` — must match exactly
- Writing patterns that are too generic (e.g., `["error"]`) — will match everything
- Forgetting the `|` after `explain:` and `fix:` — these must be YAML block scalars
- Adding fields not in the schema — the validator rejects unknown fields
- Putting patterns on entries in `db/rust/` or `db/python/` — not needed, detection uses regex for those

## Full example

```yaml
id: port-already-in-use
tool: docker
language: docker
title: Port is already allocated
tags: [port, networking, bind]
patterns:
  - ["port is already allocated"]
  - ["address already in use"]
  - ["Bind for", "failed"]

explain: |
  Docker tried to bind a container port to a host port that is already in use by
  another container or a process running on your machine.

fix: |
  1. Find what is using the port: lsof -i :<port> or ss -tlnp | grep <port>
  2. Stop the conflicting container: docker stop <container>
  3. Or use a different host port: docker run -p 8081:80 instead of -p 8080:80
  4. To see all running containers: docker ps

example_error: |
  docker: Error response from daemon: driver failed programming external connectivity
  on endpoint mycontainer: Bind for 0.0.0.0:8080 failed: port is already allocated.

links:
  - https://docs.docker.com/engine/reference/run/#expose-incoming-ports
```
