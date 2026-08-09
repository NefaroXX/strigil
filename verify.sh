#!/usr/bin/env bash
# verify.sh — exercises strigil's matching, flags, highlighting, and exit codes.
#
# Run from the crate root (requires a local Rust toolchain and cargo):
#   bash verify.sh
#
# Each test asserts the process exit code.

set -u

cargo build --quiet
build_status=$?
if [ "$build_status" -ne 0 ]; then
    echo "FAIL: 'cargo build' exited $build_status"
    exit 1
fi

BIN="target/debug/strigil"
[ -x "$BIN.exe" ] && BIN="$BIN.exe"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

TEST_FILE="$TMP/sample.txt"
printf 'The quick brown fox\njumps over the lazy dog\nthe end\n' > "$TEST_FILE"

pass=0
fail=0

assert_exit() {
    local desc="$1"
    local expected="$2"
    shift 2
    "$@" >/dev/null 2>&1
    local actual=$?
    if [ "$actual" -eq "$expected" ]; then
        echo "PASS: $desc (exit $actual)"
        pass=$((pass + 1))
    else
        echo "FAIL: $desc (expected exit $expected, got $actual)"
        fail=$((fail + 1))
    fi
}

assert_exit "matching pattern in a 3-line file" 0 "$BIN" fox "$TEST_FILE"
assert_exit "no matches but file read successfully" 1 "$BIN" zebra "$TEST_FILE"
assert_exit "--ignore-case matching" 0 "$BIN" THE "$TEST_FILE" --ignore-case
assert_exit "--ignore-case before the positionals" 0 "$BIN" --ignore-case THE "$TEST_FILE"
assert_exit "--ignore-case between the positionals" 0 "$BIN" THE --ignore-case "$TEST_FILE"
assert_exit "missing file error" 3 "$BIN" fox "$TMP/does-not-exist.txt"
assert_exit "wrong arg count" 2 "$BIN"
assert_exit "too many positional arguments" 2 "$BIN" fox "$TEST_FILE" extra

# COLOR=always must wrap the first match per line in ANSI red.
OUT="$(COLOR=always "$BIN" fox "$TEST_FILE")"
if printf '%s' "$OUT" | grep -q $'\x1b\[31m'; then
    echo "PASS: COLOR=always ANSI highlight (exit 0)"
    pass=$((pass + 1))
else
    echo "FAIL: COLOR=always ANSI highlight"
    fail=$((fail + 1))
fi

# Standard input: with no file argument, strigil reads stdin.
if printf '%s\n' 'The quick brown fox' 'jumps over the lazy dog' 'the end' | "$BIN" fox >/dev/null 2>&1; then
    echo "PASS: reads standard input when no file is given (exit 0)"
    pass=$((pass + 1))
else
    echo "FAIL: reading standard input"
    fail=$((fail + 1))
fi

assert_exit "-i matching" 0 "$BIN" THE "$TEST_FILE" -i
assert_exit "-i before the positionals" 0 "$BIN" -i THE "$TEST_FILE"
assert_exit "-c count" 0 "$BIN" the "$TEST_FILE" -c
assert_exit "-c combines with -i" 0 "$BIN" -i -c the "$TEST_FILE"
assert_exit "-v invert" 0 "$BIN" the "$TEST_FILE" -v
assert_exit "-v with every line matching" 1 "$BIN" e "$TEST_FILE" -v
assert_exit "--help prints usage" 0 "$BIN" --help
assert_exit "--version prints version" 0 "$BIN" --version
assert_exit "-V prints version" 0 "$BIN" -V

# -c must print exactly the matching-line count, nothing else.
COUNT_OUT="$("$BIN" -i -c the "$TEST_FILE")"
if [ "$COUNT_OUT" = "3" ]; then
    echo "PASS: -c prints the matching-line count (exit 0)"
    pass=$((pass + 1))
else
    echo "FAIL: -c printed '$COUNT_OUT', expected 3"
    fail=$((fail + 1))
fi

BINARY_FILE="$TMP/binary.dat"
printf 'hello\0world\n' > "$BINARY_FILE"
assert_exit "binary file with a match" 0 "$BIN" hello "$BINARY_FILE"
assert_exit "binary file without a match" 1 "$BIN" nope "$BINARY_FILE"

echo "----"
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
