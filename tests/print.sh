#!/usr/bin/env bash

set -euo pipefail

# Behavior: print command (print, p, show) smoke. Compose/labels live in Rust.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git

section "Aliases (print / p / show)"

setup_semver "$PREFIX"
DEFAULT="$(bump print)"
assert_eq "print/alias/print" "$DEFAULT" print
assert_eq "print/alias/p" "$DEFAULT" p
assert_eq "print/alias/show" "$DEFAULT" show

echo "[print/no-trailing-newline]"
raw="$(bump p; printf '|')"
if [[ "$raw" == *$'\n'* ]]; then
    echo "print output unexpectedly contains a newline"
    printf '%q\n' "$raw"
    exit 1
fi
if [[ "$raw" != "${DEFAULT}|" ]]; then
    echo "unexpected print payload: $raw"
    exit 1
fi
echo "ok"
echo

section "Default print and flags"

assert_eq "print/default" "${PREFIX}0.1.0" p
assert_eq "print/only-base" "0.1.0" p --only base
assert_eq "print/with-suffix" "${PREFIX}0.1.0+${GIT_SHA}" p --with suffix
assert_eq "print/with-timestamp" "${PREFIX}0.1.0  ${TIMESTAMP}" p --with timestamp
assert_eq "print/full" "${PREFIX}0.1.0+${GIT_SHA}  ${TIMESTAMP}" p --full

set_label_position "after-base"
assert_eq "print/label" "${PREFIX}0.1.0-tiger" p --label "-tiger"

section "Mixed compose (date keys padded)"

setup_mixed
assert_eq "print/mixed" "v2020.2.01.9" p
assert_eq "print/mixed/semver" "v2020.2.01" p --semver
assert_eq "print/mixed/semver-only-base" "2020.2.01" p --semver --only base

section "Print suffix requires git"

NOGIT_DIR="$(mktemp -d)"
cp bump.toml "$NOGIT_DIR/"
(
    cd "$NOGIT_DIR"
    set +e
    output="$(bump p --with suffix 2>&1)"
    status=$?
    set -e
    if [[ "$status" -eq 0 ]]; then
        echo "expected failure for --with suffix outside git repo"
        exit 1
    fi
    if [[ "$output" != *"Not a git repository"* ]]; then
        echo "unexpected output: $output"
        exit 1
    fi
)
rm -rf "$NOGIT_DIR"
echo "[print/with-suffix/no-git]"
echo "ok"
echo

echo "All print tests passed."
