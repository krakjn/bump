#!/usr/bin/env bash

set -euo pipefail

# Behavior: bumpfile schema validation and semver/calver key rewriting.

source "$(dirname "$0")/lib.sh"

MALFORMED="$FIXTURES/malformed"
MODE_SWAP="$FIXTURES/mode-swap"

# ---------------------------------------------------------------------------
section "Malformed / legacy bumpfiles"
# ---------------------------------------------------------------------------

assert_fails \
    "schema/v6-semver" \
    "missing field \`prefix\`" \
    "$MALFORMED/v6-semver.toml"

assert_fails \
    "schema/v6-calver" \
    "missing field \`prefix\`" \
    "$MALFORMED/v6-calver.toml"

assert_fails \
    "schema/invalid-toml" \
    "Failed to parse TOML document" \
    "$MALFORMED/invalid-toml.toml"

assert_fails \
    "schema/missing-base" \
    "missing field \`base\`" \
    "$MALFORMED/missing-base.toml"

assert_fails \
    "schema/base-not-table" \
    "unknown variant \`mode\`" \
    "$MALFORMED/base-not-table.toml"

assert_fails \
    "schema/missing-file" \
    "Configuration file not found" \
    "$MALFORMED/does-not-exist.toml"

assert_fails \
    "schema/missing-prefix" \
    "missing field \`prefix\`" \
    "$MALFORMED/missing-prefix.toml"

assert_fails \
    "schema/missing-major" \
    "missing field \`major\`" \
    "$MALFORMED/missing-major.toml"

assert_fails \
    "schema/missing-phase" \
    "missing field \`phase\`" \
    "$MALFORMED/missing-phase.toml"

assert_fails \
    "schema/bad-label-position" \
    "unknown variant \`middle\`" \
    "$MALFORMED/bad-label-position.toml"

assert_prints \
    "schema/semver-with-calver-keys" \
    "v2020.1.1" \
    "$MALFORMED/semver-with-calver-keys.toml"

assert_warns_on_bump \
    "schema/semver-with-calver-keys-on-bump" \
    "keys will be rewritten" \
    "$MALFORMED/semver-with-calver-keys.toml"

assert_prints \
    "schema/valid" \
    "v0.1.0" \
    "$MALFORMED/valid.toml"

# ---------------------------------------------------------------------------
section "Mode key rewrite"
# ---------------------------------------------------------------------------

assert_print_silent \
    "schema/semver-mode-calver-keys/print" \
    "v2020.1.1" \
    "$MODE_SWAP/semver-mode-calver-keys.toml"

assert_print_silent \
    "schema/calver-mode-semver-keys/print" \
    "v2020.01.01" \
    "$MODE_SWAP/calver-mode-semver-keys.toml"

assert_print_silent \
    "schema/calver-mode-year-only/print" \
    "2020" \
    "$MODE_SWAP/calver-mode-year-only.toml"

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
