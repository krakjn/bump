#!/usr/bin/env bash

set -euo pipefail

# Behavior: no-subcommand UX and help lists bumpfile keys.

source "$(dirname "$0")/lib.sh"

enter_workspace

echo "[cli/no-subcommand]"
set +e
output="$(bump 2>&1)"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
    echo "expected failure without subcommand"
    exit 1
fi
if [[ "$output" != *"bump error >> No command provided"* ]]; then
    echo "expected bump error message"
    echo "got: $output"
    exit 1
fi
if [[ "$output" != *"bump --help"* ]]; then
    echo "expected hint to run bump --help"
    echo "got: $output"
    exit 1
fi
echo "ok"
echo

section "Help lists keys from bumpfile"

setup_semver
assert_contains "cli/help/patch" "patch" --help

setup_calver
assert_contains "cli/help/date" "date" --help

setup_mixed
assert_contains "cli/help/mixed-alpha" "alpha" --help
assert_contains "cli/help/mixed-date" "date" --help

echo "All cli tests passed."
