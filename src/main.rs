//! strigil — a minimal, dependency-free grep clone.
//!
//! Usage: `strigil <pattern> [<file>...] [-i] [-c] [-v] [-r] [-V] [--help]`
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
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "Usage: strigil <pattern> [<file>] [options]";

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
    files: Vec<&'a str>,
    ignore_case: bool,
    invert_match: bool,
    count: bool,
    recursive: bool,
}

impl<'a> Invocation<'a> {
    /// Parses the CLI arguments. The pattern is required; any following
    /// positional arguments are files (or directories with `-r`), and an
    /// empty file list falls back to standard input (a literal `-` also
    /// names standard input). Flags are accepted in any position, `--` ends
    /// option parsing, and `--help` / `--version` (or `-V`) short-circuit
    /// to informational output.
    fn parse(args: &'a [String]) -> Result<Parsed<'a>, String> {
        let mut positional: Vec<&'a str> = Vec::new();
        let mut ignore_case = false;
        let mut invert_match = false;
        let mut count = false;
        let mut recursive = false;
        let mut help = false;
        let mut version = false;
        let mut after_dashes = false;

        for arg in args {
            if after_dashes {
                positional.push(arg);
                continue;
            }
            match arg.as_str() {
                "--" => after_dashes = true,
                "--ignore-case" | "-i" => ignore_case = true,
                "--invert-match" | "-v" => invert_match = true,
                "--count" | "-c" => count = true,
                "--recursive" | "-r" => recursive = true,
                "--help" => help = true,
                "--version" | "-V" => version = true,
                other if other.starts_with('-') && other != "-" => {
                    return Err(format!("unrecognized option '{other}'"));
                }
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
                files: Vec::new(),
                ignore_case,
                invert_match,
                count,
                recursive,
            })),
            [pattern, files @ ..] => Ok(Parsed::Run(Invocation {
                pattern,
                files: files.to_vec(),
                ignore_case,
                invert_match,
                count,
                recursive,
            })),
            _ => Err(format!(
                "expected a pattern followed by files (<pattern> [<file>...]), got {}",
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

    match run_all(&invocation) {
        RunOutcome::Matches | RunOutcome::Empty => ExitCode::SUCCESS,
        RunOutcome::NoMatches => ExitCode::from(1),
        RunOutcome::Error => ExitCode::from(3),
    }
}

/// Prints the full help text describing the CLI contract.
fn print_help() {
    println!(
        "strigil — a minimal, dependency-free grep clone

{USAGE}

Arguments:
  <pattern>   The literal substring to search for.
  <file>      One or more files to read line by line; standard input when
              omitted or `-`. Directories are searched recursively with `-r`.

Options:
  -i, --ignore-case      Match case-insensitively (accepted in any position).
  -v, --invert-match     Print lines that do NOT contain the pattern.
  -c, --count            Print only the number of matching lines.
  -r, --recursive        Search directories recursively.
  --help                 Print this help and exit.
  -V, --version          Print the version and exit.

Exit codes:
  0  match found, or the input was empty
  1  no match, but the input was read successfully
  2  usage error
  3  I/O error

Environment:
  COLOR=always|never|auto   Force, disable, or auto-detect highlighting.
  NO_COLOR                  When present (any value), disable highlighting."
    );
}

/// How the whole search over all inputs ended.
enum RunOutcome {
    Matches,
    NoMatches,
    Empty,
    Error,
}

enum SearchOutcome {
    MatchFound,
    NoMatch,
    EmptyFile,
}

/// One searchable input: a regular file or standard input.
enum Source {
    File(PathBuf),
    Stdin,
}

/// Runs the search over every resolved input. Per-source I/O errors are
/// reported to stderr without stopping the remaining sources.
fn run_all(invocation: &Invocation) -> RunOutcome {
    let files: Vec<&str> = if invocation.files.is_empty() {
        vec!["-"]
    } else {
        invocation.files.clone()
    };

    let mut sources: Vec<(String, Source)> = Vec::new();
    let mut any_error = false;
    for raw in &files {
        match expand(raw, invocation.recursive) {
            Ok(found) => sources.extend(found),
            Err(error) => {
                eprintln!("strigil: {raw}: {error}");
                any_error = true;
            }
        }
    }

    // Filenames are prefixed when searching several inputs — or whenever a
    // directory is being walked, matching `grep -r`'s always-prefix rule.
    let prefix = sources.len() > 1 || invocation.recursive;
    let mut any_match = false;
    let mut empty_seen = false;
    for (name, source) in &sources {
        match run_source(invocation, source, name, prefix) {
            Ok(SearchOutcome::MatchFound) => any_match = true,
            Ok(SearchOutcome::EmptyFile) => empty_seen = true,
            Ok(SearchOutcome::NoMatch) => {}
            Err(error) => {
                eprintln!("strigil: {name}: {error}");
                any_error = true;
            }
        }
    }

    if any_match {
        RunOutcome::Matches
    } else if any_error {
        RunOutcome::Error
    } else if empty_seen && sources.len() == 1 {
        RunOutcome::Empty
    } else {
        RunOutcome::NoMatches
    }
}

/// Expands one command-line input into concrete sources: a literal `-` names
/// standard input, a file is used as-is, and with `--recursive` a directory
/// is walked in sorted order. Without `--recursive`, a directory argument is
/// rejected so it fails loudly instead of silently matching nothing.
fn expand(raw: &str, recursive: bool) -> io::Result<Vec<(String, Source)>> {
    if raw == "-" {
        return Ok(vec![("<standard input>".to_string(), Source::Stdin)]);
    }

    let path = Path::new(raw);
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        if !recursive {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "is a directory (use -r to search recursively)",
            ));
        }
        let mut files: Vec<PathBuf> = Vec::new();
        walk_dir(path, &mut files)?;
        return Ok(files
            .into_iter()
            .map(|file| (file.display().to_string(), Source::File(file)))
            .collect());
    }
    Ok(vec![(raw.to_string(), Source::File(PathBuf::from(raw)))])
}

/// Collects regular files under `dir`, sorted by name for deterministic
/// output. Symlinks are skipped so recursion can never loop through them.
fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_dir(&path, out)?;
        } else if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Decides whether matching lines are highlighted. A present `NO_COLOR`
/// variable always wins (per no-color.org); otherwise `COLOR=always`,
/// `COLOR=never`, or `COLOR=auto` force, forbid, or auto-detect. The default
/// is to highlight only when stdout is a terminal, like grep.
fn want_highlight() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }
    match env::var("COLOR").as_deref() {
        Ok("always") => return true,
        Ok("never") => return false,
        _ => {}
    }
    io::stdout().is_terminal()
}

/// Scans one input line by line and prints every line containing `pattern`.
/// When `prefix` is set, each printed line (or count) is tagged with the
/// input's display name, grep-style.
fn run_source(
    invocation: &Invocation,
    source: &Source,
    name: &str,
    prefix: bool,
) -> io::Result<SearchOutcome> {
    let mut input: Box<dyn BufRead> = match source {
        Source::File(path) => Box::new(BufReader::new(File::open(path)?)),
        Source::Stdin => Box::new(io::stdin().lock()),
    };

    // Binary heuristic, in the spirit of grep: an input whose first chunk
    // contains a NUL byte is treated as binary and searched as raw bytes.
    let is_binary = {
        let head = input.fill_buf()?;
        head.contains(&0)
    };

    if is_binary {
        return run_binary(&mut *input, invocation, name);
    }

    let needle = if invocation.ignore_case {
        invocation.pattern.to_lowercase()
    } else {
        invocation.pattern.to_string()
    };
    let highlight = want_highlight();

    let mut matched = false;
    let mut count = 0usize;
    let mut lines_read = 0;

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        lines_read += 1;

        let haystack = if invocation.ignore_case {
            line.to_lowercase()
        } else {
            line.clone()
        };

        let hit = haystack.find(&needle);
        if hit.is_some() != invocation.invert_match {
            matched = true;
            count += 1;
            if !invocation.count {
                if let Some(position) = hit {
                    print_match(
                        prefix,
                        name,
                        index + 1,
                        &line,
                        position,
                        needle.len(),
                        highlight,
                    );
                } else {
                    print_match(prefix, name, index + 1, &line, 0, 0, false);
                }
            }
        }
    }

    if invocation.count {
        if prefix {
            println!("{name}:{count}");
        } else {
            println!("{count}");
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
fn run_binary(
    input: &mut dyn BufRead,
    invocation: &Invocation,
    name: &str,
) -> io::Result<SearchOutcome> {
    let needle = if invocation.ignore_case {
        invocation.pattern.to_lowercase().into_bytes()
    } else {
        invocation.pattern.as_bytes().to_vec()
    };

    if needle.is_empty() {
        // An empty pattern matches anything — even binary input.
        println!("strigil: {name}: binary file matches");
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
            println!("strigil: {name}: binary file matches");
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

/// Prints `{line_number}:{line}` — prefixed with the source name when
/// multiple inputs are searched — wrapping the first occurrence of the match
/// in ANSI red when `highlight` is set.
fn print_match(
    prefix: bool,
    name: &str,
    line_number: usize,
    line: &str,
    position: usize,
    length: usize,
    highlight: bool,
) {
    let head = if prefix {
        format!("{name}:{line_number}:")
    } else {
        format!("{line_number}:")
    };
    if highlight {
        let end = position.saturating_add(length);
        // The match position comes from the case-folded haystack. Unicode case
        // folding can change a string's length, so the boundaries may not map
        // onto the original line; fall back to plain output rather than
        // slicing mid-character.
        if line.is_char_boundary(position) && line.is_char_boundary(end) {
            println!(
                "{head}{}\x1b[31m{}\x1b[0m{}",
                &line[..position],
                &line[position..end],
                &line[end..]
            );
            return;
        }
    }
    println!("{head}{line}");
}
