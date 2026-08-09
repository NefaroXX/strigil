//! Integration tests that spawn the compiled `strigil` binary.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const THREE_LINES: &str = "The quick brown fox\njumps over the lazy dog\nthe end\n";

fn temp_file() -> PathBuf {
    let path = std::env::temp_dir().join(format!("strigil_cli_test_{}.txt", std::process::id()));
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
fn usage_error_reports_the_argument_count() {
    let file = temp_file();
    let out = run(&["THE", file.to_str().unwrap(), "extra", "more"]);
    assert_eq!(exit_code(&out), 2);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("got 4"), "stderr was: {stderr}");
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
fn binary_input_with_a_match_prints_binary_file_matches() {
    let path = std::env::temp_dir().join(format!("strigil_cli_test_{}.bin", std::process::id()));
    fs::write(&path, b"hello\x00world\n").expect("writes a binary temp file");
    let out = run(&["hello", path.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("binary file matches"));
}

#[test]
fn binary_input_without_a_match_exits_one() {
    let path = std::env::temp_dir().join(format!("strigil_cli_test_{}.bin", std::process::id()));
    fs::write(&path, b"hello\x00world\n").expect("writes a binary temp file");
    let out = run(&["zebra", path.to_str().unwrap()]);
    assert_eq!(exit_code(&out), 1);
    assert!(out.stdout.is_empty());
}
