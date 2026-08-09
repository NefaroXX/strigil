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
assert_exit "missing file error" 3 "$BIN" fox "$TMP/does-not-exist.txt"
assert_exit "wrong arg count" 2 "$BIN" fox

# COLOR=always must wrap the first match per line in ANSI red.
OUT="$(COLOR=always "$BIN" fox "$TEST_FILE")"
if printf '%s' "$OUT" | grep -q $'\x1b\[31m'; then
    echo "PASS: COLOR=always ANSI highlight (exit 0)"
    pass=$((pass + 1))
else
    echo "FAIL: COLOR=always ANSI highlight"
    fail=$((fail + 1))
fi

echo "----"
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
