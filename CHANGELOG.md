# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-09

### Added
- Initial release: a minimal, dependency-free `grep` clone.
- Manual CLI parsing (`<pattern> <file> [--ignore-case]`) with usage errors
  exiting `2`.
- Buffered line-by-line reading via `std::io::BufReader`.
- Substring matching with optional `--ignore-case` case folding.
- ANSI red highlighting of the first match per line when `COLOR=always` is set.
- Deterministic exit codes: `0` match / empty file, `1` no match,
  `2` usage error, `3` I/O error.
- `verify.sh` verification script and `tests/cli.rs` integration tests.
- Repository community-health files: `SECURITY.md`, issue templates, and a
  pull request template (mirroring the `ls-tree` repo layout).
