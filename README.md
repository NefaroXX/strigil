# strigil

[![CI](https://github.com/NefaroXX/strigil/actions/workflows/ci.yml/badge.svg)](https://github.com/NefaroXX/strigil/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A minimal, dependency-free clone of `grep`, written in Rust using only the
standard library. Point it at a pattern and a file and it prints every matching
line, with optional case-insensitive matching and ANSI highlighting.

```
$ strigil the README.md
3:the end
```

## Mission

`strigil` is built to be:

- **Dependency-free** — zero external crates. Only `std::fs`, `std::io`, and
  `std::env`.
- **Publishable** — a single binary crate, `cargo publish --dry-run` ready.
- **Runnable** — `cargo run -- <pattern> <file>` from anywhere in the repo.

## Features

- **Manual CLI parsing** — no clap, no derive macros:
  `strigil <pattern> <file> [--ignore-case]`.
- **Buffered line-by-line reading** via `std::io::BufReader`.
- **Substring matching** — a plain `find()`, not a regex engine.
- **`--ignore-case`** — folds both the pattern and each line to lowercase
  before matching. The flag is accepted anywhere on the command line —
  before, between, or after the two positional arguments.
- **ANSI highlighting** — set `COLOR=always` and the first match on each line
  is wrapped in red (`\x1b[31m...\x1b[0m`).
- **Predictable exit codes** — `0` match, `1` no match, `2` usage error,
  `3` I/O error.

## Installation

### From source (recommended)

```bash
git clone https://github.com/NefaroXX/strigil.git
cd strigil
cargo install --path .
```

### From crates.io

```bash
cargo install strigil
```

### Build locally

```bash
cargo build --release
./target/release/strigil
```

## Usage

```bash
strigil <pattern> <file> [--ignore-case]
```

| Argument | Description |
| --- | --- |
| `<pattern>` | The literal substring to search for. |
| `<file>` | The file to read line by line. |
| `--ignore-case` | Optional; match case-insensitively. |

> **Note:** matching is case-sensitive by default. There is no `-i` short flag.

### Examples

```bash
# Find every line containing "fn main" in main.rs
strigil "fn main" src/main.rs

# Case-insensitive search
strigil ERROR server.log --ignore-case

# Flag order is flexible — before the positionals works too
strigil --ignore-case ERROR server.log

# Highlight matches in red (single match per line only)
COLOR=always strigil TODO src/main.rs
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Matches found (or the file was empty). |
| `1` | No matches found (but the file was read successfully). |
| `2` | Usage error (wrong number of arguments or unknown flag). |
| `3` | I/O error (file not found, permission denied, etc.). |

## How it works

`strigil` is intentionally boring. `main` parses the arguments by hand, opens
the file with `std::fs::File`, wraps it in `std::io::BufReader`, and iterates
`lines()` one at a time. For each line it searches for the pattern with a
substring `find()`; when `--ignore-case` is given, both sides are folded to
lowercase first. On a match it prints `{line_number}:{line}` — with the first
occurrence wrapped in ANSI red when `COLOR=always` is set. Errors bubble up
with the `?` operator and are mapped to the exit codes above.

There is no regex engine, no glob handling, and no memory beyond one line at a
time — that is the entire point of the project.

## Development

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
bash verify.sh
```

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and please
follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under the [MIT License](LICENSE).
