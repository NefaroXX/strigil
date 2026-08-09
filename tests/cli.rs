//! Integration tests that spawn the compiled `strigil` binary.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

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
fn missing_file_is_an_io_error_exit_three() {
    let out = run(&["fox", "definitely-not-a-real-file.txt"]);
    assert_eq!(exit_code(&out), 3);
}

#[test]
fn wrong_argument_count_is_a_usage_error_exit_two() {
    let out = run(&["only-a-pattern"]);
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
