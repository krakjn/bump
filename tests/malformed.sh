#!/usr/bin/env bash

set -euo pipefail

# Integration tests for malformed and legacy bump.toml files.
# Covers v6 schema rejection, TOML syntax errors, and missing required fields.

source "$(dirname "$0")/lib.sh"

FIXTURES="$ROOT/tests/fixtures/malformed"

# Legacy bump v6 schemas

assert_fails \
    "v6-semver" \
    "missing field \`prefix\`" \
    "$FIXTURES/v6-semver.toml"

assert_fails \
    "v6-calver" \
    "missing field \`prefix\`" \
    "$FIXTURES/v6-calver.toml"

# TOML syntax and structural errors

assert_fails \
    "invalid-toml" \
    "Failed to parse TOML document" \
    "$FIXTURES/invalid-toml.toml"

assert_fails \
    "missing-base" \
    "missing field \`base\`" \
    "$FIXTURES/missing-base.toml"

assert_fails \
    "base-not-table" \
    "unknown variant \`mode\`" \
    "$FIXTURES/base-not-table.toml"

assert_fails \
    "missing-file" \
    "Configuration file not found" \
    "$FIXTURES/does-not-exist.toml"

# v7 schema validation (deserialization)

assert_fails \
    "missing-prefix" \
    "missing field \`prefix\`" \
    "$FIXTURES/missing-prefix.toml"

assert_fails \
    "missing-major" \
    "missing field \`major\`" \
    "$FIXTURES/missing-major.toml"

assert_fails \
    "missing-phase" \
    "missing field \`phase\`" \
    "$FIXTURES/missing-phase.toml"

assert_fails \
    "bad-label-position" \
    "unknown variant \`middle\`" \
    "$FIXTURES/bad-label-position.toml"

# Compatibility warnings and valid input

assert_prints \
    "semver-with-calver-keys" \
    "v2020.1.1" \
    "$FIXTURES/semver-with-calver-keys.toml"

assert_warns_on_bump \
    "semver-with-calver-keys-on-bump" \
    "keys will be rewritten" \
    "$FIXTURES/semver-with-calver-keys.toml"

assert_prints \
    "valid" \
    "v0.1.0" \
    "$FIXTURES/valid.toml"

echo "All malformed bumpfile tests passed."
