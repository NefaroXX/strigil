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
  `Invocation::parse` (CLI), `run_all` / `run_source` (per-input I/O +
  matching), `expand` / `walk_dir` (input resolution), `run_binary`
  (raw-byte search), `print_match` (rendering). Boundary:
  `main -> run_all` (single call site).
- `Parsed::{Run(Invocation), Help, Version}` is the parse result; `Invocation
  { pattern, files: Vec<&str>, ignore_case, invert_match, count, recursive }`
  is the parsed-enough-to-run description; everything else is derived inside
  `run_all`.
- `run_source` returns `io::Result<SearchOutcome>` where
  `SearchOutcome::{MatchFound, NoMatch, EmptyFile}` feeds `RunOutcome`:
  any match wins (`0`), otherwise any I/O error (`3`), otherwise no match
  (`1`) — with a single empty input still mapping to success (`0`), per the
  mission spec. Usage errors map to `2`.
- Rendering is inline via `println!`; no abstraction over output sinks is
  warranted for a tool this small.

## PATTERNS
- Argument parsing by hand from `env::args().skip(1)` — no clap, no derive
  macros. Positionals are collected in order; flags (`-i`, `-c`, `-v`, `-r`
  and their long forms) are recognized in any position, `--` ends option
  parsing, and anything else starting with `-` is a usage error.
- Buffered, line-at-a-time reading with `BufReader::lines()`; the full file is
  never loaded into memory, and inputs are searched one at a time.
- Substring matching with `str::find`. For `--ignore-case`, both the pattern
  and each line are folded with `to_lowercase()` before matching.
- Highlighting: on by default when stdout is a terminal (`IsTerminal`);
  `COLOR=always|never|auto` forces, forbids, or auto-detects, and a present
  `NO_COLOR` variable always wins. The first match per line is wrapped in
  `\x1b[31m...\x1b[0m`. The match position comes from the case-folded
  haystack, so the slice boundaries are validated against the original line
  with `is_char_boundary` before slicing; a non-boundary case falls back to
  plain output instead of panicking.
- Input sources: file arguments open `File` one at a time; a literal `-` or
  an empty file list locks `io::stdin()`. With `-r`, directories are walked
  in sorted order (deterministic output) and symlinks are skipped so
  recursion cannot loop. Per-source I/O errors go to stderr without stopping
  the remaining sources. When more than one source is searched — or any
  directory is walked — output lines and `-c` counts carry a `file:` prefix,
  matching `grep -r`.
- The first chunk of each input is probed with `fill_buf` for a NUL byte —
  binary input is searched as raw bytes with an overlapping window
  (`run_binary`) and reported with a single "binary file matches" line, in
  the spirit of grep.
- `--help` and `--version` short-circuit in `parse` before positional
  validation, print to stdout, and exit `0`.
- Exit codes are a contract: `0` match / empty input, `1` no match,
  `2` usage error, `3` I/O error. Documented in README and enforced by
  `verify.sh` and `tests/cli.rs`.

## TRADEOFFS
- **Zero dependencies is the mission** — deliberately and permanently. Every
  candidate feature is filtered through "does this still work with no crates?".
  Regex and globbing are still deliberately deferred — a literal substring
  match is the whole point. This constraint is why `-r` walks directories
  with `std::fs` rather than a globbing crate, and why colour detection uses
  `std::io::IsTerminal` (which set the MSRV at 1.70).
- Case folding with `to_lowercase()` is simple and correct for the vast
  majority of input, but length-changing Unicode folds (e.g. `İ`) can make the
  match position drift relative to the original line; the boundary check turns
  a potential panic into a graceful fallback.
- Ambient colour is deliberately scoped: auto-detection only fires for real
  terminals, and `COLOR` / `NO_COLOR` make every pipeline deterministic —
  `verify.sh` and the integration tests run with piped stdout, so they never
  depend on terminal state.
- `--ignore-case` is deliberately accepted in any position rather than only
  as the third argument: a friendlier grammar (and prototype parity) wins
  over strict flag position, and the scan cost is negligible for a
  small CLI.
- Binary detection is a heuristic: a NUL byte in the first chunk classifies
  the input as binary. This mirrors grep's own behaviour closely enough for a
  tool of this size, and trades a rare false positive (valid UTF-8 containing
  NULs) for never choking on arbitrary bytes.
- Multi-file aggregation keeps the exit-code contract simple: with no match
  anywhere, a missing file is still `3`; with any match, the run succeeds
  (`0`) even if another file failed. Grep's "error override" nuance is
  deliberately not reproduced.
- The file argument is optional (stdin fallback) because streaming
  compatibility is the cheapest way to compose with other tools (`ps aux |
  strigil python`); `-` keeps the grammar explicit for scripts.

## PHILOSOPHY
- Dependencies are the enemy. If a feature cannot be built with `std`, it does
  not belong in this crate.
- Behaviour must be deterministic and testable: fixed exit codes, colour that
  is either explicitly configured or terminal-scoped, sorted recursion.
- Fail loud, fail fast: I/O and usage errors are reported with an
  `eprintln!` and a documented exit code — never silently ignored.
