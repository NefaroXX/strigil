# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-08-09

### Added
- `-i` and `-V` short flags, aliasing `--ignore-case` and `--version`
  respectively (grep-style); accepted anywhere alongside the long forms.
- `-c` / `--count` prints only the number of matching lines, and
  `-v` / `--invert-match` prints only the lines that do NOT contain the
  pattern; both combine with `-i` and with each other.
- Multiple `<file>` arguments: every printed line (or `-c` count) is
  prefixed with its file name, and a missing file is reported to stderr
  without hiding matches from the other files.
- `-r` / `--recursive` directory search: directories are walked in sorted
  order and symlinks are skipped, so recursion is deterministic and cannot
  loop. A directory without `-r` is now an I/O error instead of silently
  matching nothing.
- `--` ends option parsing, so filenames beginning with `-` can be searched.
- Terminal-aware highlighting: color turns on automatically when stdout is
  a terminal. `COLOR=always` forces it, `COLOR=never` disables it, and a
  present `NO_COLOR` variable disables it even when `COLOR=always` is set.
- Tag-triggered `release.yml`: pushing a `v*` tag builds `--release`
  binaries for Linux, Windows, and macOS and attaches them to the matching
  GitHub Release (`workflow_dispatch` builds a "continuous" release).
  `Cargo.lock` is now tracked so release builds are reproducible.

### Changed
- Any number of `<file>` arguments are now accepted (previously at most
  one), and unknown dash-prefixed options are rejected as usage errors.
- Rust MSRV raised from 1.61 to 1.70 for `std::io::IsTerminal`.
- Standard input: `strigil <pattern>` reads from stdin when no file is given,
  and a `-` file argument also means stdin.
- `--help` and `--version` flags, both printing to stdout and exiting `0`.
- Binary input detection: when the first chunk of input contains a NUL byte,
  strigil searches the raw bytes and prints a single "binary file matches"
  line (exit `0` on a match, `1` otherwise).
- CI now runs the full gate on Windows and macOS in addition to Linux.

### Changed
- The `<file>` argument is now optional — it falls back to standard input.
- `--ignore-case` is now accepted anywhere on the command line, not only as
  the third argument.
- Usage errors now report the number of positional arguments that were given.

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
