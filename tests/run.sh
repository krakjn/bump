#!/usr/bin/env bash

set -euo pipefail

# Single entrypoint for bump behavior tests.
# Requires a release binary (set BUMP_BIN, or use target/release/bump).

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Ensure integration tests run against a release binary built from this tree.
if [[ -z "${BUMP_BIN:-}" ]]; then
    if [[ ! -x "$ROOT/target/release/bump" ]] \
        || find "$ROOT/src" "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" \
            -newer "$ROOT/target/release/bump" -print -quit 2>/dev/null | grep -q .; then
        echo "Building release binary for behavior tests..."
        cargo build --release
    fi
fi

# Resolve relative BUMP_BIN against the repo root before suites cd away.
if [[ -n "${BUMP_BIN:-}" && "$BUMP_BIN" != /* ]]; then
    export BUMP_BIN="$ROOT/$BUMP_BIN"
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
    completion
)

for suite in "${SUITES[@]}"; do
    echo "======== ${suite} ========"
    "$(dirname "$0")/${suite}.sh"
    echo
done

echo "All behavior tests passed."
