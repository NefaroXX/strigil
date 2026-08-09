# Contributing to strigil

Thanks for your interest in improving `strigil`! This project aims to stay
small, fast, and **dependency-free**. A few guidelines keep it that way.

## Getting started

1. Fork the repository and clone your fork.
2. Make sure you have a recent stable Rust toolchain
   (`rustup toolchain install stable`).
3. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Before opening a pull request

Run the same checks CI runs, and make sure they pass locally:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
bash verify.sh
```

- **Formatting:** code must be `rustfmt`-clean.
- **Lints:** `clippy` runs with `-D warnings` — no warnings allowed.
- **Tests:** add or update integration tests in `tests/` for any behaviour
  change, and extend `verify.sh` where the behaviour is user-visible.

## Design principles

- **Standard library only.** Do not add external crate dependencies — the
  whole point of `strigil` is being a tiny, dependency-free grep clone. If a
  feature genuinely requires a crate, it needs a written-up trade-off in
  `docs/ADR.md` and discussion before any change.
- **Never panicking.** Malformed input, missing files, and odd encodings
  should produce a clear error message and the documented exit code, not a
  crash.
- **Small, focused changes.** Keep PRs focused on one improvement. Update
  `CHANGELOG.md` under the `[Unreleased]` heading for user-facing changes.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) where
practical, e.g. `feat:`, `fix:`, `docs:`, `test:`, `refactor:`.

## Code of Conduct

By participating, you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).
