#!/usr/bin/env bash

set -euo pipefail

# Single entrypoint for bump tests: unit tests first, then CLI contracts.
# Default binary is debug. Set BUMP_BIN to skip cargo test (foreign artifact / CI).

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -n "${BUMP_BIN:-}" && "$BUMP_BIN" != /* ]]; then
    export BUMP_BIN="$ROOT/$BUMP_BIN"
fi

if [[ -z "${BUMP_BIN:-}" ]]; then
    cargo test
    cargo build
fi

SUITES=(
    cli
    print
    mutate
    changed
    meta
    emit
    init
    tag
    update
    schema
)

for suite in "${SUITES[@]}"; do
    echo "======== ${suite} ========"
    "$(dirname "$0")/${suite}.sh"
    echo
done

echo "All behavior tests passed."
