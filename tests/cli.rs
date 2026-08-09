//! Integration tests that spawn the compiled `strigil` binary.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const THREE_LINES: &str = "The quick brown fox\njumps over the lazy dog\nthe end\n";

fn temp_file() -> PathBuf {
    // Thread id keeps parallel tests from racing on a shared fixture file.
    let path = std::env::temp_dir().join(format!(
        "strigil_cli_test_{}_{:?}.txt",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::write(&path, THREE_LINES).expect("writes a temp test file");
    path
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_strigil"))
        .args(args)
        .output()
        .expect("spawns strigil")
}

/// Runs strigil with the given arguments and a piped standard input.
fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_strigil"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns strigil");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes())
        .expect("writes test input");
    child.wait_with_output().expect("waits for strigil")
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().expect("process exits normally")
}

#[test]
fn prints_matching_lines_with_numbers_and_exits_zero() {
    let file = temp_file();
    let out = run(&["fox", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(!stdout.contains("2:"));
}

#[test]
fn exits_one_when_no_lines_match() {
    let file = temp_file();
    let out = run(&["zebra", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 1);
    assert!(out.stdout.is_empty());
}

#[test]
fn matching_is_case_sensitive_by_default() {
    let file = temp_file();
    let out = run(&["the", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("3:the end"));
}

#[test]
fn ignore_case_flag_matches_case_insensitively() {
    let file = temp_file();
    let out = run(&["THE", file.to_str().unwrap(), "--ignore-case"]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(stdout.contains("3:the end"));
}

#[test]
fn ignore_case_flag_accepted_before_the_positionals() {
    let file = temp_file();
    let out = run(&["--ignore-case", "THE", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(stdout.contains("3:the end"));
}

#[test]
fn ignore_case_flag_accepted_between_the_positionals() {
    let file = temp_file();
    let out = run(&["THE", "--ignore-case", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("1:The quick brown fox"));
}

#[test]
fn multiple_files_are_searched_with_a_filename_prefix() {
    let dir = std::env::temp_dir().join(format!("strigil_cli_multi_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("creates a temp directory");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    fs::write(&a, THREE_LINES).expect("writes a.txt");
    fs::write(&b, "nothing here\n").expect("writes b.txt");
    let out = run(&["fox", a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("a.txt:1:The quick brown fox"),
        "stdout was: {stdout}"
    );
    assert!(!stdout.contains("b.txt:"), "stdout was: {stdout}");
}

#[test]
fn a_missing_file_among_others_does_not_hide_matches() {
    let file = temp_file();
    let out = run(&["fox", file.to_str().unwrap(), "definitely-missing.txt"]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("1:The quick brown fox"),
        "stdout was: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("definitely-missing.txt"),
        "stderr was: {stderr}"
    );
}

#[test]
fn every_file_missing_exits_three() {
    let out = run(&["fox", "missing-a.txt", "missing-b.txt"]);
    assert_eq!(exit_code(&out), 3);
}

#[test]
fn count_with_multiple_files_prefixes_each_count() {
    let dir = std::env::temp_dir().join(format!("strigil_cli_count_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("creates a temp directory");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    fs::write(&a, THREE_LINES).expect("writes a.txt");
    fs::write(&b, "nothing here\n").expect("writes b.txt");
    let out = run(&["-c", "fox", a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a.txt:1"), "stdout was: {stdout}");
    assert!(stdout.contains("b.txt:0"), "stdout was: {stdout}");
}

#[test]
fn recursive_flag_searches_an_entire_directory_tree() {
    let dir = std::env::temp_dir().join(format!("strigil_cli_tree_{}", std::process::id()));
    let nested = dir.join("sub");
    fs::create_dir_all(&nested).expect("creates a temp tree");
    fs::write(nested.join("nested.txt"), THREE_LINES).expect("writes nested.txt");
    let out = run(&["-r", "fox", dir.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("nested.txt:1:The quick brown fox"),
        "stdout was: {stdout}"
    );
}

#[test]
fn double_dash_allows_filenames_starting_with_a_dash() {
    let dir = std::env::temp_dir().join(format!("strigil_cli_dash_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("creates a temp directory");
    let dashed = dir.join("-dashed.txt");
    fs::write(&dashed, THREE_LINES).expect("writes -dashed.txt");
    let out = run(&["fox", "--", dashed.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("1:The quick brown fox"));
}

#[test]
fn directory_without_recursive_is_an_io_error() {
    let dir = std::env::temp_dir().join(format!("strigil_cli_dir_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("creates a temp directory");
    let out = run(&["fox", dir.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 3);
}

#[test]
fn missing_file_is_an_io_error_exit_three() {
    let out = run(&["fox", "definitely-not-a-real-file.txt"]);
    assert_eq!(exit_code(&out), 3);
}

#[test]
fn wrong_argument_count_is_a_usage_error_exit_two() {
    let out = run(&[]);
    assert_eq!(exit_code(&out), 2);
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}

#[test]
fn unknown_third_flag_is_a_usage_error_exit_two() {
    let file = temp_file();
    let out = run(&["fox", file.to_str().unwrap(), "--wat"]);
    assert_eq!(exit_code(&out), 2);
}

#[test]
fn color_always_highlights_the_first_match_in_ansi_red() {
    let file = temp_file();
    let out = Command::new(env!("CARGO_BIN_EXE_strigil"))
        .args(["fox", file.to_str().unwrap()])
        .env("COLOR", "always")
        .output()
        .expect("spawns strigil");
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("\x1b[31mfox\x1b[0m"));
}

#[test]
fn plain_output_contains_no_ansi_escape_codes() {
    let file = temp_file();
    let out = run(&["fox", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(!String::from_utf8_lossy(&out.stdout).contains('\u{1b}'));
}

#[test]
fn reads_pattern_from_standard_input_when_no_file_is_given() {
    let out = run_with_stdin(&["fox"], THREE_LINES);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(!stdout.contains("2:"));
}

#[test]
fn dash_as_the_file_reads_standard_input() {
    let out = run_with_stdin(&["fox", "-"], THREE_LINES);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("1:The quick brown fox"));
}

#[test]
fn empty_standard_input_exits_zero() {
    let out = run_with_stdin(&["fox"], "");
    assert_eq!(exit_code(&out), 0);
    assert!(out.stdout.is_empty());
}

#[test]
fn help_flag_prints_usage_and_exits_zero() {
    let out = run(&["--help"]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Usage: strigil <pattern> [<file>]"));
    assert!(stdout.contains("--ignore-case"));

    // --help wins even when positionals are present.
    let out = run(&["fox", "ignored.txt", "--help"]);
    assert_eq!(exit_code(&out), 0);
}

#[test]
fn version_flag_prints_the_version_and_exits_zero() {
    let out = run(&["--version"]);
    assert_eq!(exit_code(&out), 0);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("strigil {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn short_i_flag_aliases_ignore_case() {
    let file = temp_file();
    let out = run(&["THE", file.to_str().unwrap(), "-i"]);
    assert_eq!(exit_code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(stdout.contains("3:the end"));
}

#[test]
fn short_i_flag_accepted_before_the_positionals() {
    let file = temp_file();
    let out = run(&["-i", "THE", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("1:The quick brown fox"));
}

#[test]
fn capital_v_flag_prints_the_version() {
    let out = run(&["-V"]);
    assert_eq!(exit_code(&out), 0);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("strigil {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn count_flag_prints_only_the_number_of_matching_lines() {
    let file = temp_file();
    let out = run(&["the", file.to_str().unwrap(), "-c"]);
    assert_eq!(exit_code(&out), 0);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}

#[test]
fn count_flag_combines_with_ignore_case() {
    let file = temp_file();
    let out = run(&["-i", "the", "-c", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

#[test]
fn count_on_empty_input_prints_zero_and_exits_zero() {
    let out = run_with_stdin(&["fox", "-c"], "");
    assert_eq!(exit_code(&out), 0);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "0");
}

#[test]
fn invert_flag_prints_only_non_matching_lines() {
    let file = temp_file();
    let out = run(&["the", file.to_str().unwrap(), "-v"]);
    assert_eq!(exit_code(&out), 0);
    // Only line 1 ("The quick brown fox") lacks a lowercase "the".
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1:The quick brown fox"));
    assert!(!stdout.contains("2:"));
    assert!(!stdout.contains("3:"));
}

#[test]
fn invert_flag_exits_one_when_every_line_matches() {
    let file = temp_file();
    let out = run(&["e", file.to_str().unwrap(), "-v"]);
    assert_eq!(exit_code(&out), 1);
    assert!(out.stdout.is_empty());
}

#[test]
fn count_and_invert_combine() {
    let file = temp_file();
    let out = run(&["-v", "-c", "e", file.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 1); // every line contains 'e', so none inverted
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "0");
}

#[test]
fn binary_input_with_a_match_prints_binary_file_matches() {
    let path = std::env::temp_dir().join(format!(
        "strigil_cli_test_{}_{:?}.bin",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::write(&path, b"hello\x00world\n").expect("writes a binary temp file");
    let out = run(&["hello", path.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("binary file matches"));
}

#[test]
fn binary_input_without_a_match_exits_one() {
    let path = std::env::temp_dir().join(format!(
        "strigil_cli_test_{}_{:?}_nope.bin",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::write(&path, b"hello\x00world\n").expect("writes a binary temp file");
    let out = run(&["zebra", path.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 1);
    assert!(out.stdout.is_empty());
}
