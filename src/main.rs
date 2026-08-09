//! strigil — a minimal, dependency-free grep clone.
//!
//! Usage: `strigil <pattern> [<file>] [--ignore-case] [--help] [--version]`
//!
//! Reads `<file>` — or standard input when no file is given — line by line
//! and prints every line containing `<pattern>` as `{line_number}:{line}`.
//! When the `COLOR` environment variable is set to `always`, the first
//! occurrence of the pattern on each matching line is highlighted in red
//! (`\x1b[31m...\x1b[0m`).
//!
//! Exit codes: `0` match found (or empty input), `1` no match,
//! `2` usage error, `3` I/O error.

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;

const USAGE: &str = "Usage: strigil <pattern> [<file>] [--ignore-case] [--help] [--version]";

/// What the command line asked for, once parsed.
enum Parsed<'a> {
    /// Search `pattern` in `file`, or standard input when `file` is `None`.
    Run(Invocation<'a>),
    /// `--help` was given; print usage and exit successfully.
    Help,
    /// `--version` was given; print the version and exit successfully.
    Version,
}

/// One parsed invocation: what to look for, where, and how.
struct Invocation<'a> {
    pattern: &'a str,
    file: Option<&'a str>,
    ignore_case: bool,
}

impl<'a> Invocation<'a> {
    /// Parses the CLI arguments. The pattern is required; the file is
    /// optional and falls back to standard input (a literal `-` also names
    /// standard input). `--ignore-case` is accepted in any position, and
    /// `--help` / `--version` short-circuit to informational output.
    fn parse(args: &'a [String]) -> Result<Parsed<'a>, String> {
        let mut positional: Vec<&'a str> = Vec::new();
        let mut ignore_case = false;
        let mut help = false;
        let mut version = false;

        for arg in args {
            match arg.as_str() {
                "--ignore-case" => ignore_case = true,
                "--help" => help = true,
                "--version" => version = true,
                other => positional.push(other),
            }
        }

        if help {
            return Ok(Parsed::Help);
        }
        if version {
            return Ok(Parsed::Version);
        }

        match positional.as_slice() {
            [pattern] => Ok(Parsed::Run(Invocation {
                pattern,
                file: None,
                ignore_case,
            })),
            [pattern, file] => {
                let file = if *file == "-" { None } else { Some(*file) };
                Ok(Parsed::Run(Invocation {
                    pattern,
                    file,
                    ignore_case,
                }))
            }
            _ => Err(format!(
                "expected a pattern and at most one file (<pattern> [<file>]), got {}",
                positional.len()
            )),
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let invocation = match Invocation::parse(&args) {
        Ok(Parsed::Help) => {
            print_help();
            return ExitCode::SUCCESS;
        }
        Ok(Parsed::Version) => {
            println!("strigil {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(Parsed::Run(invocation)) => invocation,
        Err(message) => {
            eprintln!("strigil: {message}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match run(&invocation) {
        Ok(SearchOutcome::MatchFound) | Ok(SearchOutcome::EmptyFile) => ExitCode::SUCCESS,
        Ok(SearchOutcome::NoMatch) => ExitCode::from(1),
        Err(error) => {
            eprintln!("strigil: {error}");
            ExitCode::from(3)
        }
    }
}

/// Prints the full help text describing the CLI contract.
fn print_help() {
    println!(
        "strigil — a minimal, dependency-free grep clone

{USAGE}

Arguments:
  <pattern>   The literal substring to search for.
  <file>      The file to read line by line; standard input when omitted or `-`.

Options:
  --ignore-case    Match case-insensitively (accepted in any position).
  --help           Print this help and exit.
  --version        Print the version and exit.

Exit codes:
  0  match found, or the input was empty
  1  no match, but the input was read successfully
  2  usage error
  3  I/O error

Environment:
  COLOR=always     Highlight the first match per line in ANSI red."
    );
}

enum SearchOutcome {
    MatchFound,
    NoMatch,
    EmptyFile,
}

/// Scans the input line by line and prints every line containing `pattern`.
fn run(invocation: &Invocation) -> io::Result<SearchOutcome> {
    let mut input: Box<dyn BufRead> = match invocation.file {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(io::stdin().lock()),
    };

    // Binary heuristic, in the spirit of grep: an input whose first chunk
    // contains a NUL byte is treated as binary and searched as raw bytes.
    let is_binary = {
        let head = input.fill_buf()?;
        head.contains(&0)
    };

    if is_binary {
        return run_binary(&mut *input, invocation);
    }

    let needle = if invocation.ignore_case {
        invocation.pattern.to_lowercase()
    } else {
        invocation.pattern.to_string()
    };
    let highlight = match env::var("COLOR") {
        Ok(value) => value == "always",
        Err(_) => false,
    };

    let mut matched = false;
    let mut lines_read = 0;

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        lines_read += 1;

        let haystack = if invocation.ignore_case {
            line.to_lowercase()
        } else {
            line.clone()
        };

        if let Some(position) = haystack.find(&needle) {
            matched = true;
            print_match(index + 1, &line, position, needle.len(), highlight);
        }
    }

    Ok(if lines_read == 0 {
        SearchOutcome::EmptyFile
    } else if matched {
        SearchOutcome::MatchFound
    } else {
        SearchOutcome::NoMatch
    })
}

/// Scans binary input for the pattern as raw bytes. A small overlap keeps
/// matches that straddle a chunk boundary from being missed. Output is a
/// single grep-style "binary file matches" line; per-line rendering is
/// skipped because binary data has no reliable line structure.
fn run_binary(input: &mut dyn BufRead, invocation: &Invocation) -> io::Result<SearchOutcome> {
    let needle = if invocation.ignore_case {
        invocation.pattern.to_lowercase().into_bytes()
    } else {
        invocation.pattern.as_bytes().to_vec()
    };
    let source = invocation.file.unwrap_or("<standard input>");

    if needle.is_empty() {
        // An empty pattern matches anything — even binary input.
        println!("strigil: {source}: binary file matches");
        return Ok(SearchOutcome::MatchFound);
    }

    let overlap = needle.len() - 1;
    let mut window: Vec<u8> = Vec::new();

    loop {
        let chunk = input.fill_buf()?;
        if chunk.is_empty() {
            break;
        }

        window.extend_from_slice(chunk);
        if window.windows(needle.len()).any(|w| w == needle) {
            println!("strigil: {source}: binary file matches");
            return Ok(SearchOutcome::MatchFound);
        }

        // Keep only the trailing bytes that could still complete a match.
        let keep = overlap.min(window.len());
        window.drain(..window.len() - keep);

        let len = chunk.len();
        input.consume(len);
    }

    Ok(SearchOutcome::NoMatch)
}

/// Prints `line_number:line`, wrapping the first occurrence of the match in
/// ANSI red when `highlight` is set.
fn print_match(line_number: usize, line: &str, position: usize, length: usize, highlight: bool) {
    if highlight {
        let end = position.saturating_add(length);
        // The match position comes from the case-folded haystack. Unicode case
        // folding can change a string's length, so the boundaries may not map
        // onto the original line; fall back to plain output rather than
        // slicing mid-character.
        if line.is_char_boundary(position) && line.is_char_boundary(end) {
            println!(
                "{}:{}\x1b[31m{}\x1b[0m{}",
                line_number,
                &line[..position],
                &line[position..end],
                &line[end..]
            );
            return;
        }
    }
    println!("{line_number}:{line}");
}
