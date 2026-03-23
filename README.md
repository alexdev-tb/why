# why

**Plain English explanations for compiler errors and CLI failures.**

Stop staring at cryptic error messages. Just type `why`.

## Demo

```
$ cargo build
error[E0499]: cannot borrow `x` as mutable more than once at a time

$ why

  → E0499 — Multiple mutable borrows
  ────────────────────────────────────────────────────────────
  You are trying to mutably borrow a value that is already
  mutably borrowed. Rust only allows one mutable reference at
  a time to prevent data races.

  Fix: Drop the first borrow before creating the second, or
  use RefCell<T> if you need shared mutation.

  Docs: https://doc.rust-lang.org/error_codes/E0499.html
  ────────────────────────────────────────────────────────────
  Was this helpful? Improve this entry → github.com/TODO/why
```

## What it does

`why` reads the last error from your terminal and explains it in plain English — what went wrong, why it happened, and how to fix it. No more context-switching to Google or Stack Overflow.

## Installation

```sh
cargo install --git https://github.com/alexdev-tb/why
```

## Usage

Run `why` after a failed command to get an explanation of the error:

```sh
why
```

Look up a specific error code directly:

```sh
why E0499
```

Browse all supported error codes:

```sh
why --list
```

Set up the shell hook for automatic error capture:

```sh
why --setup
```

## Shell Hook Setup

The shell hook captures the output of failed commands automatically so `why` can read them. Run `why --setup` for guided setup, or add the hook manually:

### Bash

Add to your `~/.bashrc`:

```bash
eval "$(why --hook bash)"
```

### Zsh

Add to your `~/.zshrc`:

```zsh
eval "$(why --hook zsh)"
```

Then restart your shell or run `source ~/.bashrc` (or `~/.zshrc`).

## Currently Supported

- **Rust** — `rustc` error codes (E0001 through E0999)

### Coming Soon

- Python tracebacks
- GCC/G++ errors
- Node.js errors

## Contributing

Contributions are welcome! Adding a new error explanation is as simple as copying a YAML template and writing a few sentences of plain English.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MIT](LICENSE)
