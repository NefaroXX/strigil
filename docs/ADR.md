# Architecture Decision Record — strigil

> Persisted architectural context for `strigil`. This file is the
> git-shareable copy of the design decisions that keep the project minimal.

## PURPOSE
strigil is a minimal, dependency-free clone of `grep`. Given a literal pattern
and an optional file path, it prints every line containing the pattern as
`{line_number}:{line}`, exiting with a deterministic status code. When no file
is given (or the file argument is `-`), standard input is searched instead.

## STACK
- Language: Rust, edition 2021; no external dependencies — the standard library
  only (`std::env`, `std::fs`, `std::io`, `std::process`).
- Build/test: cargo; CI via `.github/workflows/ci.yml`.
- Packaging: single binary crate with an explicit `[[bin]]` target in
  `src/main.rs`. No library surface.

## ARCHITECTURE
- Binary crate with a thin `main` and a small set of plain functions:
  `Invocation::parse` (CLI), `run` (I/O + matching), `run_binary` (raw-byte
  search), `print_match` (rendering). Boundary: `main -> run` (single call site).
- `Parsed::{Run(Invocation), Help, Version}` is the parse result; `Invocation
  { pattern, file: Option<&str>, ignore_case }` is the parsed-enough-to-run
  description; everything else is derived inside `run`.
- `run` returns `io::Result<SearchOutcome>` where
  `SearchOutcome::{MatchFound, NoMatch, EmptyFile}` maps to exit codes
  `0`, `1`, `0` respectively (per the mission spec: an empty file with no
  matches is success). I/O errors map to `3`, usage errors to `2`.
- Rendering is inline via `println!`; no abstraction over output sinks is
  warranted for a tool this small.

## PATTERNS
- Argument parsing by hand from `env::args().skip(1)` — no clap, no derive
  macros. Positionals are collected in order; the literal flag
  `--ignore-case` is recognized in any position (prototype parity).
- Buffered, line-at-a-time reading with `BufReader::lines()`; the full file is
  never loaded into memory.
- Substring matching with `str::find`. For `--ignore-case`, both the pattern
  and each line are folded with `to_lowercase()` before matching.
- Highlighting: when `COLOR=always`, the first match per line is wrapped in
  `\x1b[31m...\x1b[0m`. The match position comes from the case-folded haystack,
  so the slice boundaries are validated against the original line with
  `is_char_boundary` before slicing; a non-boundary case falls back to plain
  output instead of panicking.
- Input sources: a file argument opens `File`; otherwise `io::stdin().lock()`
  is used. The first chunk is probed with `fill_buf` for a NUL byte — binary
  input is searched as raw bytes with an overlapping window (`run_binary`) and
  reported with a single "binary file matches" line, in the spirit of grep.
- `--help` and `--version` short-circuit in `parse` before positional
  validation, print to stdout, and exit `0`.
- Exit codes are a contract: `0` match / empty input, `1` no match,
  `2` usage error, `3` I/O error. Documented in README and enforced by
  `verify.sh` and `tests/cli.rs`.

## TRADEOFFS
- **Zero dependencies is the mission** — deliberately and permanently. Every
  candidate feature is filtered through "does this still work with no crates?".
  Regex, globbing, and recursive search are all possible in stdlib but are
  consciously deferred (see ROADMAP.md).
- Case folding with `to_lowercase()` is simple and correct for the vast
  majority of input, but length-changing Unicode folds (e.g. `İ`) can make the
  match position drift relative to the original line; the boundary check turns
  a potential panic into a graceful fallback.
- `COLOR=always` is an explicit opt-in via environment variable — there is no
  auto-detection and no `NO_COLOR` handling. Chosen for deterministic,
  testable behaviour over convenience.
- `--ignore-case` is deliberately accepted in any position rather than only
  as the third argument: a friendlier grammar (and prototype parity) wins
  over strict flag position, and the scan cost is negligible for a
  two-argument CLI.
- Binary detection is a heuristic: a NUL byte in the first chunk classifies
  the input as binary. This mirrors grep's own behaviour closely enough for a
  tool of this size, and trades a rare false positive (valid UTF-8 containing
  NULs) for never choking on arbitrary bytes.
- The file argument is optional (stdin fallback) because streaming
  compatibility is the cheapest way to compose with other tools (`ps aux |
  strigil python`); `-` keeps the grammar explicit for scripts.

## PHILOSOPHY
- Dependencies are the enemy. If a feature cannot be built with `std`, it does
  not belong in this crate.
- Behaviour must be deterministic and testable: fixed exit codes, explicit
  opt-in colour, no ambient terminal detection.
- Fail loud, fail fast: I/O and usage errors are reported with an
  `eprintln!` and a documented exit code — never silently ignored.
