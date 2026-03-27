# Contributing to why

Thanks for helping make error messages less painful. Every contribution, a single YAML file, a better explanation, a typo fix helps developers who are stuck.

## How `why` works

`why` has a database of error explanations stored as YAML files under `db/`. Each tool (compiler, CLI, etc.) gets its own directory:

```
db/
  rust/          # rustc errors (E0499.yaml, E0308.yaml, ...)
  python/        # Python exceptions (TypeError.yaml, ImportError.yaml, ...)
  c_cpp/         # gcc/clang errors (undefined-reference.yaml, ...)
  go/            # Go compiler errors (undefined.yaml, unused-variable.yaml, ...)
  git/           # git errors (diverged-branches.yaml, ...)
```

When a command fails, `why` reads the stderr output and tries to match it against database entries. For tools like `rustc` and `python`, it matches structured error codes (e.g., `E0499`, `TypeError`). For everything else, it uses the `patterns` field in each YAML file to find a match.

## Adding a new error entry

1. Fork the repo and clone it locally
2. Copy the template:
   ```sh
   cp db/TEMPLATE.yaml db/<tool>/my-error.yaml
   ```
3. Fill in the fields (see below)
4. Validate:
   ```sh
   python scripts/validate.py
   ```
5. Open a pull request

### Adding a new tool

If the tool directory doesn't exist yet, create it:

```sh
mkdir db/docker
cp db/TEMPLATE.yaml db/docker/no-such-image.yaml
```

That's it. No code changes needed, `why` picks up new directories automatically.

## YAML fields

| Field           | Required | Description                                                                 |
| --------------- | -------- | --------------------------------------------------------------------------- |
| `id`            | Yes      | Unique identifier, must match the filename (without `.yaml`)                |
| `tool`          | Yes      | The tool that produces this error (`rustc`, `gcc/clang`, `go`, `git`, etc.) |
| `language`      | Yes      | Directory name under `db/` (`rust`, `c_cpp`, `go`, `git`, etc.)             |
| `title`         | Yes      | Short description, under 60 characters                                      |
| `tags`          | No       | Keywords for filtering (`[memory, pointers, segfault]`)                     |
| `patterns`      | \*       | How `why` detects this error from stderr (see below)                        |
| `exclude`       | No       | Strings that must NOT appear, to avoid false matches                        |
| `explain`       | Yes      | Plain English explanation of what went wrong                                |
| `fix`           | Yes      | Concrete steps to fix it                                                    |
| `example_error` | No       | The actual error message, copy-pasted from a terminal                       |
| `example_code`  | No       | Minimal code that reproduces the error                                      |
| `links`         | No       | URLs to official docs or further reading                                    |

\* `patterns` is required for auto-detection unless the tool has structured error codes (like `rustc` or `python`).

## How patterns work

The `patterns` field tells `why` how to recognize an error from stderr output. Each pattern is a list of strings that must **all** appear in the same line:

```yaml
# Matches any line containing BOTH "undefined reference"
patterns:
  - ["undefined reference"]
```

Multiple patterns give you OR logic, if **any** pattern matches, the error is detected:

```yaml
# Matches "Segmentation fault" OR "segmentation fault"
patterns:
  - ["Segmentation fault"]
  - ["segmentation fault"]
```

A pattern with multiple strings means all of them must appear in the same line (AND logic):

```yaml
# Matches a line containing BOTH "fatal error" AND "No such file or directory"
patterns:
  - ["No such file or directory", "fatal error"]
```

Use `exclude` when two tools have similar error messages:

```yaml
# "too many arguments" appears in both C and Go errors
patterns:
  - ["too many arguments"]
exclude:
  - "go"
```

### Tips for writing patterns

- Use the most specific text you can, prefer `"undefined reference"` over `"undefined"`
- Copy-paste directly from the error message in your terminal
- Test against the `example_error` in your YAML: does the pattern match?
- You don't need regex, simple substrings work

## Example entry

```yaml
id: diverged-branches
tool: git
language: git
title: Local and remote branches have diverged
tags: [branches, merge, rebase, remote]
patterns:
  - ["have diverged"]
  - ["divergent branches"]

explain: |
  Your local branch and the remote branch both have new commits that
  the other doesn't. This usually happens when someone else pushed to
  the remote after your last pull, and you also made local commits.

fix: |
  Pull with rebase to replay your commits on top of the remote:
    git pull --rebase origin main
  If there are conflicts, resolve them and run:
    git rebase --continue
  If you want to merge instead of rebase:
    git pull origin main

example_error: |
  hint: Your branch and 'origin/main' have diverged,
  hint: and have 2 and 3 different commits each, respectively.

links:
  - https://git-scm.com/docs/git-pull
```

## Writing good explanations

**Would a beginner understand this without extra context?** If your explanation uses jargon without defining it, or skips the "why," revise it.

- **Use plain English.** Write like you're explaining to a colleague, not writing a textbook.
- **Avoid jargon.** If you must use a technical term, briefly define it.
- **Be specific.** "Add `-lm` to link the math library" is better than "link the correct libraries."
- **Give actionable fixes.** Tell the reader exactly what to change and how.
- **Keep it concise.** 3-6 sentences for `explain`, 2-4 steps for `fix`.
- **Use second person.** "You are trying to..." not "The programmer is trying to..."

## Validating locally

Run the validation script before opening a PR:

```sh
python scripts/validate.py
```

This runs the same checks as CI. Fix any errors it reports before submitting.

## Code of conduct

Be kind. Be helpful. We're all here to make error messages less miserable.

- Treat every contributor with respect, regardless of experience level.
- Give constructive feedback on pull requests.
- Remember that someone stuck on an error is already having a bad day, your explanation should make it better, not worse.

Thank you for contributing!
