#!/usr/bin/env bash

# Shared helpers for bump integration tests.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ -n "${BUMP_BIN:-}" ]]; then
    :
elif [[ -n "${CARGO_TARGET_DIR:-}" && -x "${CARGO_TARGET_DIR}/release/bump" ]]; then
    BUMP_BIN="${CARGO_TARGET_DIR}/release/bump"
else
    BUMP_BIN="$ROOT/target/release/bump"
fi

bump() {
    "$BUMP_BIN" "$@"
}

assert_fails() {
    local name="$1"
    local pattern="$2"
    local bumpfile="$3"
    local output
    local status=0

    echo "[$name]"
    set +e
    output="$(bump print "$bumpfile" 2>&1)"
    status=$?
    set -e

    if [[ "$status" -eq 0 ]]; then
        echo "expected failure, but command succeeded"
        echo "output: $output"
        exit 1
    fi

    if [[ "$output" != *"$pattern"* ]]; then
        echo "expected stderr/stdout to contain: $pattern"
        echo "got: $output"
        exit 1
    fi

    echo "ok"
    echo
}

assert_prints() {
    local name="$1"
    local expected="$2"
    local bumpfile="$3"
    local actual

    echo "[$name]"
    actual="$(bump print "$bumpfile")"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_warns_on_bump() {
    local name="$1"
    local warn_pattern="$2"
    local bumpfile="$3"
    local tmp
    local output
    local stderr
    local status=0

    tmp="$(mktemp)"
    cp "$bumpfile" "$tmp"

    echo "[$name]"
    set +e
    output="$(bump "$tmp" --patch 2>"${tmp}.err")"
    stderr="$(cat "${tmp}.err")"
    status=$?
    set -e

    rm -f "${tmp}.err" "$tmp"

    if [[ "$status" -ne 0 ]]; then
        echo "expected success with warning, but command failed"
        echo "stdout: $output"
        echo "stderr: $stderr"
        exit 1
    fi

    if [[ "$stderr" != *"$warn_pattern"* ]]; then
        echo "expected warning containing: $warn_pattern"
        echo "stderr: $stderr"
        exit 1
    fi

    echo "ok"
    echo
}

base_section() {
    awk '/^\[base\]/{flag=1; next} /^\[/{flag=0} flag' "$1"
}

assert_base_has_key() {
    local file="$1"
    local key="$2"
    if ! base_section "$file" | grep -q "^${key} ="; then
        echo "expected [base] to contain key: $key"
        echo "base section:"
        base_section "$file"
        exit 1
    fi
}

assert_base_lacks_key() {
    local file="$1"
    local key="$2"
    if base_section "$file" | grep -q "^${key} ="; then
        echo "expected [base] to lack key: $key"
        echo "base section:"
        base_section "$file"
        exit 1
    fi
}

assert_print_silent() {
    local name="$1"
    local expected="$2"
    local bumpfile="$3"
    local actual
    local stderr
    local tmp

    tmp="$(mktemp)"

    echo "[$name]"
    set +e
    actual="$(bump print "$bumpfile" 2>"$tmp")"
    stderr="$(cat "$tmp")"
    set -e
    rm -f "$tmp"

    if [[ -n "$stderr" ]]; then
        echo "expected no stderr on print, got: $stderr"
        exit 1
    fi

    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_bump_rewrites_keys() {
    local name="$1"
    local bumpfile="$2"
    local bump_flag="$3"
    local warn_pattern="$4"
    shift 4
    local want_keys=("$@")
    local tmp
    local stderr
    local status=0
    local key

    tmp="$(mktemp)"
    cp "$bumpfile" "$tmp"

    echo "[$name]"
    set +e
    bump "$tmp" "$bump_flag" >/dev/null 2>"${tmp}.err"
    stderr="$(cat "${tmp}.err")"
    status=$?
    set -e
    rm -f "${tmp}.err"

    if [[ "$status" -ne 0 ]]; then
        echo "bump failed"
        cat "$tmp"
        exit 1
    fi

    if [[ "$stderr" != *"$warn_pattern"* ]]; then
        echo "expected warning containing: $warn_pattern"
        echo "stderr: $stderr"
        exit 1
    fi

    for key in "${want_keys[@]}"; do
        assert_base_has_key "$tmp" "$key"
    done

    case "${want_keys[0]}" in
        major|minor|patch)
            for key in year month day; do
                assert_base_lacks_key "$tmp" "$key"
            done
            ;;
        year|month|day)
            for key in major minor patch; do
                assert_base_lacks_key "$tmp" "$key"
            done
            ;;
    esac

    rm -f "$tmp"
    echo "ok"
    echo
}
