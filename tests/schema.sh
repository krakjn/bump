#!/usr/bin/env bash

set -euo pipefail

# Behavior: bumpfile schema validation (error strings).

source "$(dirname "$0")/lib.sh"

MALFORMED="$FIXTURES/malformed"

section "Malformed bumpfiles"

assert_fails \
    "schema/invalid-toml" \
    "Failed to parse TOML document" \
    p "$MALFORMED/invalid-toml.toml"

assert_fails \
    "schema/missing-base" \
    "'base' table not found" \
    p "$MALFORMED/missing-base.toml"

assert_fails \
    "schema/base-not-table" \
    "'base' table not found" \
    p "$MALFORMED/base-not-table.toml"

assert_fails \
    "schema/missing-file" \
    "Configuration file not found" \
    p "$MALFORMED/does-not-exist.toml"

assert_fails \
    "schema/missing-prefix" \
    "Expected key 'prefix' not found in [(root)]" \
    p "$MALFORMED/missing-prefix.toml"

assert_fails \
    "schema/missing-phase" \
    "'phase' table not found" \
    p "$MALFORMED/missing-phase.toml"

assert_fails \
    "schema/bad-label-position" \
    "Invalid label position" \
    p "$MALFORMED/bad-label-position.toml"

assert_prints \
    "schema/valid" \
    "v0.1.0" \
    p "$MALFORMED/valid.toml"

echo "All schema tests passed."
