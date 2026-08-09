//! strigil — a minimal, dependency-free grep clone.
//!
//! Usage: `strigil <pattern> <file> [--ignore-case]`
//!
//! Exit codes: `0` match found (or empty file), `1` no match,
//! `2` usage error, `3` I/O error.

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;

const USAGE: &str = "Usage: strigil <pattern> <file> [--ignore-case]";

/// One parsed invocation: what to look for, where, and how.
struct Invocation<'a> {
    pattern: &'a str,
    file: &'a str,
    ignore_case: bool,
}

impl<'a> Invocation<'a> {
    /// Parses the CLI arguments. Exactly 2 positional arguments are required;
    /// a third positional `--ignore-case` flag is optional.
    fn parse(args: &'a [String]) -> Result<Self, String> {
        match args {
            [pattern, file] => Ok(Invocation { pattern, file, ignore_case: false }),
            [pattern, file, flag] if flag == "--ignore-case" => {
                Ok(Invocation { pattern, file, ignore_case: true })
            }
            _ => Err("expected <pattern> <file> [--ignore-case]".to_string()),
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let invocation = match Invocation::parse(&args) {
        Ok(invocation) => invocation,
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

enum SearchOutcome {
    MatchFound,
    NoMatch,
    EmptyFile,
}

/// Reads `file` line by line and prints every line containing `pattern`.
fn run(invocation: &Invocation) -> io::Result<SearchOutcome> {
    let file = File::open(invocation.file)?;
    let reader = BufReader::new(file);

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

    for (index, line) in reader.lines().enumerate() {
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
