#!/usr/bin/env bash

set -euo pipefail

# Behavior: bumpfile schema validation and semver/calver key rewriting.

source "$(dirname "$0")/lib.sh"

MALFORMED="$FIXTURES/malformed"
MODE_SWAP="$FIXTURES/mode-swap"

section "Malformed / legacy bumpfiles"

assert_fails \
    "schema/v6-semver" \
    "missing field \`prefix\`" \
    p "$MALFORMED/v6-semver.toml"

assert_fails \
    "schema/v6-calver" \
    "missing field \`prefix\`" \
    p "$MALFORMED/v6-calver.toml"

assert_fails \
    "schema/invalid-toml" \
    "Failed to parse TOML document" \
    p "$MALFORMED/invalid-toml.toml"

assert_fails \
    "schema/missing-base" \
    "missing field \`base\`" \
    p "$MALFORMED/missing-base.toml"

assert_fails \
    "schema/base-not-table" \
    "unknown variant \`mode\`" \
    p "$MALFORMED/base-not-table.toml"

assert_fails \
    "schema/missing-file" \
    "Configuration file not found" \
    p "$MALFORMED/does-not-exist.toml"

assert_fails \
    "schema/missing-prefix" \
    "missing field \`prefix\`" \
    p "$MALFORMED/missing-prefix.toml"

assert_fails \
    "schema/missing-major" \
    "missing field \`major\`" \
    p "$MALFORMED/missing-major.toml"

assert_fails \
    "schema/missing-phase" \
    "missing field \`phase\`" \
    p "$MALFORMED/missing-phase.toml"

assert_fails \
    "schema/bad-label-position" \
    "unknown variant \`middle\`" \
    p "$MALFORMED/bad-label-position.toml"

assert_prints \
    "schema/semver-with-calver-keys" \
    "v2020.1.1" \
    p "$MALFORMED/semver-with-calver-keys.toml"

assert_warns_on_bump \
    "schema/semver-with-calver-keys-on-bump" \
    "keys will be rewritten" \
    "$MALFORMED/semver-with-calver-keys.toml"

assert_prints \
    "schema/valid" \
    "v0.1.0" \
    p "$MALFORMED/valid.toml"

section "Mode key rewrite"

assert_print_silent \
    "schema/semver-mode-calver-keys/print" \
    "v2020.1.1" \
    p "$MODE_SWAP/semver-mode-calver-keys.toml"

assert_print_silent \
    "schema/calver-mode-semver-keys/print" \
    "v2020.01.01" \
    p "$MODE_SWAP/calver-mode-semver-keys.toml"

assert_print_silent \
    "schema/calver-mode-year-only/print" \
    "2020" \
    p "$MODE_SWAP/calver-mode-year-only.toml"

assert_bump_rewrites_keys \
    "schema/semver-mode-calver-keys/bump-patch" \
    "$MODE_SWAP/semver-mode-calver-keys.toml" \
    patch \
    "keys will be rewritten" \
    major minor patch

assert_bump_rewrites_keys \
    "schema/calver-mode-semver-keys/bump-calendar" \
    "$MODE_SWAP/calver-mode-semver-keys.toml" \
    calendar \
    "keys will be rewritten" \
    year month day

tmp="$(mktemp)"
cp "$MODE_SWAP/calver-mode-year-only.toml" "$tmp"
echo "[schema/calver-mode-year-only/bump-calendar]"
set +e
bump calendar "$tmp" >/dev/null 2>"${tmp}.err"
set -e
if [[ "$(cat "${tmp}.err")" != *"keys will be rewritten"* ]]; then
    echo "expected rewrite warning"
    cat "${tmp}.err"
    exit 1
fi
assert_base_has_key "$tmp" year
assert_base_lacks_key "$tmp" month
assert_base_lacks_key "$tmp" day
for key in major minor patch; do
    assert_base_lacks_key "$tmp" "$key"
done
rm -f "${tmp}.err" "$tmp"
echo "ok"
echo

echo "All schema tests passed."
