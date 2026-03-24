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
You tried to create a second mutable reference to a value while the first mutable reference is
still active. Rust's borrow checker enforces that you can have either one mutable reference or
any number of immutable references to a value at a time, but never both. This rule prevents
data races at compile time and ensures memory safety without a garbage collector.

Fix:
Limit the scope of the first mutable borrow so it ends before the second one begins. You can
do this by introducing a block `{ }` around the first borrow, or by restructuring your code
so that both mutations do not overlap. Sometimes using a method that takes `&mut self` once
to perform both operations is the cleanest solution.

Docs: https://doc.rust-lang.org/error_codes/E0499.html
────────────────────────────────────────────────────────────
Was this helpful? Improve this entry → https://github.com/alexdev-tb/why

```

## What it does

`why` reads the last error from your terminal and explains it in plain English, what went wrong, why it happened, and how to fix it. No more context-switching to Google or Stack Overflow.

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
why ErrorCode
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

The shell hook captures the output of failed commands automatically so `why` can read them. Run `why --setup` for automatic setup, or add the hook manually:

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

- **Rust** - `rustc` error codes
- **Python** - `python` tracebacks
- **GO** - `go` error codes

### Coming Soon

- C/C++ errors 
- Node.js errors
- TypeScript

## Contributing

Contributions are encouraged! Adding a new error explanation is as simple as copying a YAML template and writing a few sentences of plain English.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MIT](LICENSE)
