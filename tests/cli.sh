#!/usr/bin/env bash

set -euo pipefail

# Behavior: no subcommand UX.

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
if [[ "$output" != *"Usage:"* ]]; then
    echo "expected help output"
    exit 1
fi
echo "ok"
echo

echo "All cli tests passed."
