# Security Policy

`strigil` is a local command-line tool. It reads exactly the file you point it
at, performs substring matching in memory, and prints matching lines to
stdout. It makes no network requests and exposes no service.

## Reporting a vulnerability

Please report security concerns privately using GitHub's private vulnerability
reporting on the [repository](https://github.com/NefaroXX/strigil). Do not
open a public issue for security problems.

## Security considerations

- **Zero dependencies, zero supply chain** — no third-party crates are compiled
  into the binary.
- **No unsafe code** — the crate is 100% safe Rust.
- **Input is never executed** — patterns are matched as literal text, not
  evaluated.
- **Panic-free on bad input** — match positions are validated against character
  boundaries before string slicing (see `docs/ADR.md`).
