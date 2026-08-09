# Roadmap

`strigil` is intentionally minimal and dependency-free. This is the current
thinking — nothing here is a promise.

## 0.1.0 — core grep (current)
- [x] Zero-dependency grep clone over a single file.
- [x] `--ignore-case` flag.
- [x] `COLOR=always` ANSI highlighting.
- [x] Deterministic exit codes (`0/1/2/3`).
- [x] `verify.sh` and `tests/cli.rs` integration tests.
- [ ] First crates.io publish (`cargo publish --dry-run` ready).

## 0.2.0 — ideas that earn their place
- Multiple input files with `{file}:{line}:{content}` output.
- Recursive directory search (`-r`) — would reuse the current buffer/slice
  logic per file.
- `-n` / `-i` short flags for ergonomics.
- Reading from stdin when no file is given (`-`).

## Deferred / likely out of scope
- Regex support — impossible without an external crate, which contradicts the
  zero-dependency mission. If it ever happens, it will follow the ADR process
  in `docs/ADR.md`.
- Context lines (`-A`/`-B`/`-C`), colour auto-detection, `NO_COLOR` support —
  these require bigger decisions than a grep clone needs.
