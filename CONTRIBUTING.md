# Contributing to why

Welcome! Thanks for your interest in making compiler errors less painful for everyone.

`why` is a community-maintained database of plain English error explanations. Every contribution — whether it is a single YAML file or a typo fix — makes a real difference for developers who are stuck and frustrated.

## Quick Start

1. **Fork** the repo and clone it locally.
2. **Copy the template** for the language you want to contribute to:
   ```sh
   cp db/rust/TEMPLATE.yaml db/rust/E0XXX.yaml
   ```
3. **Fill in every field** with a clear, helpful explanation.
4. **Validate locally** to make sure everything looks right:
   ```sh
   python scripts/validate.py
   ```
5. **Open a pull request.** That's it!

## YAML Schema Reference

Each error entry is a single YAML file. Here is every field:

| Field           | Required | Description                                                                                       |
| --------------- | -------- | ------------------------------------------------------------------------------------------------- |
| `id`            | Yes      | The error code exactly as the tool prints it (e.g., `E0499`).                                     |
| `tool`          | Yes      | The tool that produces this error (e.g., `rustc`, `gcc`, `python`).                               |
| `language`      | Yes      | The programming language (e.g., `rust`, `python`, `cpp`).                                         |
| `title`         | Yes      | A short, descriptive title (e.g., "Multiple mutable borrows"). Keep it under 60 characters.       |
| `tags`          | No       | A list of keywords for search and filtering (e.g., `[borrowing, mutability, references]`).        |
| `explain`       | Yes      | A plain English explanation of what the error means and why it happens. This is the core content. |
| `fix`           | Yes      | Concrete, actionable steps to resolve the error. Be specific.                                     |
| `example_error` | No       | The error message as the compiler actually prints it. Copy-paste from a real terminal.            |
| `example_code`  | No       | Minimal code that reproduces the error. Keep it as short as possible.                             |
| `links`         | No       | A list of URLs to official docs, relevant blog posts, or further reading.                         |

### Example

```yaml
id: E0499
tool: rustc
language: rust
title: Multiple mutable borrows
tags: [borrowing, mutability, references]

explain: |
  You are trying to create a second mutable reference to a value while
  the first mutable reference is still active. Rust enforces a strict
  rule: only one mutable reference to a given value can exist at a time.
  This prevents data races at compile time.

fix: |
  Make sure the first mutable borrow is no longer in use before creating
  the second one. You can do this by:
  - Limiting the scope of the first borrow with a block: { let r = &mut x; ... }
  - Dropping the first reference explicitly before borrowing again.
  - Using RefCell<T> if you need multiple mutable accesses at runtime.

example_error: |
  error[E0499]: cannot borrow `x` as mutable more than once at a time

example_code: |
  fn main() {
      let mut x = String::from("hello");
      let r1 = &mut x;
      let r2 = &mut x; // Error: second mutable borrow
      println!("{}, {}", r1, r2);
  }

links:
  - https://doc.rust-lang.org/error_codes/E0499.html
  - https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
```

## Quality Bar

Before submitting, ask yourself:

> **Would a junior developer understand this without prior context?**

If your explanation uses jargon, assumes background knowledge, or skips the "why," revise it until it does not.

## Style Guide

- **Use plain English.** Write the way you would explain something to a colleague over coffee.
- **Avoid jargon.** If you must use a technical term, briefly define it.
- **Be specific.** "Use a block to limit the borrow scope" is better than "restructure your code."
- **Give actionable fixes.** Tell the reader exactly what to change, not just what is wrong.
- **Keep it concise.** Aim for 3-6 sentences in `explain` and 2-4 concrete steps in `fix`.
- **Use second person.** Say "you are trying to..." not "the programmer is trying to..."

## How CI Validation Works

Every pull request is automatically checked by CI:

- **Schema validation** ensures your YAML file has all required fields and correct types.
- **Format checks** catch common issues like missing titles or empty explanations.

If CI fails, read the error output — it will tell you exactly which field has a problem.

## Testing Locally

Before opening a PR, run the validation script to catch issues early:

```sh
python scripts/validate.py
```

This runs the same checks that CI does. Fix any errors it reports before submitting.

## Code of Conduct

Be kind. Be helpful. We are all here to make error messages less miserable.

- Treat every contributor with respect, regardless of experience level.
- Give constructive feedback on pull requests.
- Remember that someone stuck on a compiler error is already having a bad day — your explanation should make it better, not worse.

Thank you for contributing!
