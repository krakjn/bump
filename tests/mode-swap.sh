#!/usr/bin/env bash

set -euo pipefail

# Integration tests for semver/calver mode key rewriting on formal bumps.

source "$(dirname "$0")/lib.sh"

FIXTURES="$ROOT/tests/fixtures/mode-swap"

# print never warns — mismatched keys are read via serde aliases
assert_print_silent \
    "semver-mode-calver-keys/print" \
    "v2020.1.1" \
    "$FIXTURES/semver-mode-calver-keys.toml"

assert_print_silent \
    "calver-mode-semver-keys/print" \
    "v2020.01.01" \
    "$FIXTURES/calver-mode-semver-keys.toml"

assert_print_silent \
    "calver-mode-year-only/print" \
    "2020" \
    "$FIXTURES/calver-mode-year-only.toml"

# formal bump warns and rewrites semver mode + calver keys → major/minor/patch
assert_bump_rewrites_keys \
    "semver-mode-calver-keys/bump-patch" \
    "$FIXTURES/semver-mode-calver-keys.toml" \
    --patch \
    "keys will be rewritten" \
    major minor patch

# formal bump warns and rewrites calver mode + semver keys → year/month/day
assert_bump_rewrites_keys \
    "calver-mode-semver-keys/bump-calendar" \
    "$FIXTURES/calver-mode-semver-keys.toml" \
    --calendar \
    "keys will be rewritten" \
    year month day

# year-only calver: semver keys removed, only year written (month/day optional)
tmp="$(mktemp)"
cp "$FIXTURES/calver-mode-year-only.toml" "$tmp"
echo "[calver-mode-year-only/bump-calendar]"
set +e
bump "$tmp" --calendar >/dev/null 2>"${tmp}.err"
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

echo "All mode-swap tests passed."
