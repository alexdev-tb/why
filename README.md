<div align="center">

# why

[![Build Status](https://img.shields.io/github/actions/workflow/status/alexdev-tb/why/ci.yml?branch=main&label=Build)](https://github.com/alexdev-tb/why/actions)
[![License](https://img.shields.io/github/license/alexdev-tb/why)](https://github.com/alexdev-tb/why/blob/main/LICENSE)
[![Repo-Stars](https://img.shields.io/github/stars/alexdev-tb/why?style=flat&label=Stars&color=blue)](https://github.com/alexdev-tb/why/stargazers)
[![Repo-Forks](https://img.shields.io/github/forks/alexdev-tb/why?style=flat&label=Forks&color=blue)](https://github.com/alexdev-tb/why/forks)
[![Languages](https://img.shields.io/badge/Languages%20Supported-4-blue?style=flat)](https://github.com/alexdev-tb/why)

**Plain English explanations for compiler errors and CLI failures.**

Stop staring at cryptic error messages. Just type `why`.

</div>

## Demo

<img width="897" height="438" alt="Screenshot_20260324_215001" src="https://github.com/user-attachments/assets/f53649a5-0ea1-4890-b418-e1dba3ca8587" />

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
- **Go** - `go` error codes
- **C/C++** - `gcc`/`g++`/`clang`/`clang++` errors

### Coming Soon

- Node.js errors
- TypeScript

## Contributing

Contributions are encouraged! Adding a new error explanation is as simple as copying a YAML template and writing a few sentences of plain English.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MIT](LICENSE)
