#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump emit — one nested json, language smoke, custom/calver/mixed keys, -o.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git

expected_json() {
    cat <<EOF
{
  "version": {
    "prefix": "${PREFIX}",
    "major": "0",
    "minor": "1",
    "patch": "0",
    "phase": "",
    "phase_distance": "0",
    "string": "${PREFIX}0.1.0",
    "timestamp": "${TIMESTAMP}"
  }
}
EOF
}

section "Nested json"

setup_semver "$PREFIX"
assert_eq "emit/json/nested" "$(expected_json)" emit json

section "Language smoke"

assert_contains \
    "emit/c/string" \
    "#define VERSION_STRING \"${PREFIX}0.1.0\"" \
    emit c

assert_contains \
    "emit/c/major" \
    "#define VERSION_MAJOR 0" \
    emit c

section "Format and spacing"

expected_raw() {
    cat <<EOF
VERSION_PREFIX="${PREFIX}"
VERSION_MAJOR=0
VERSION_MINOR=1
VERSION_PATCH=0
VERSION_PHASE=""
VERSION_PHASE_DISTANCE=0
VERSION_STRING="${PREFIX}0.1.0"
VERSION_TIMESTAMP="${TIMESTAMP}"
EOF
}

assert_eq "emit/raw/format-spacing" "$(expected_raw)" emit raw

echo "[emit/go/format-spacing]"
go_out="$(bump emit go)"
if [[ "$go_out" == *$'\t'* ]]; then
    echo "go emit must indent with spaces, not tabs"
    printf '%s\n' "$go_out"
    exit 1
fi
expected_go_body=$(
    cat <<EOF
const (
    VERSION_PREFIX = "${PREFIX}"
    VERSION_MAJOR = 0
    VERSION_MINOR = 1
    VERSION_PATCH = 0
    VERSION_PHASE = ""
    VERSION_PHASE_DISTANCE = 0
    VERSION_STRING = "${PREFIX}0.1.0"
    VERSION_TIMESTAMP = "${TIMESTAMP}"
)
EOF
)
if [[ "$go_out" != *"$expected_go_body"* ]]; then
    echo "go const block spacing mismatch"
    echo "expected body:"
    printf '%s\n' "$expected_go_body"
    echo "actual:"
    printf '%s\n' "$go_out"
    exit 1
fi
if [[ "$go_out" != *"package version"$'\n\n'"const ("* ]]; then
    echo "expected blank line between package and const"
    printf '%s\n' "$go_out"
    exit 1
fi
echo "ok"
echo

echo "[emit/json/format-spacing]"
json_out="$(bump emit json)"
expected_json_spacing=$(
    cat <<EOF
{
  "version": {
    "prefix": "${PREFIX}",
    "major": "0",
    "minor": "1",
    "patch": "0",
    "phase": "",
    "phase_distance": "0",
    "string": "${PREFIX}0.1.0",
    "timestamp": "${TIMESTAMP}"
  }
}
EOF
)
if [[ "$json_out" != "$expected_json_spacing" ]]; then
    echo "json spacing mismatch"
    echo "expected:"
    printf '%s\n' "$expected_json_spacing"
    echo "actual:"
    printf '%s\n' "$json_out"
    exit 1
fi
echo "ok"
echo

section "Custom VERSION_ALPHA"

setup_custom "$PREFIX"
assert_contains \
    "emit/custom/c/alpha" \
    "#define VERSION_ALPHA 2" \
    emit c

section "Calver VERSION_YEAR"

setup_calver
assert_contains \
    "emit/calver/c/year" \
    "#define VERSION_YEAR 2020" \
    emit c

section "Mixed emit preserves file order"

setup_mixed
out="$(bump emit c)"
echo "[emit/mixed/year-then-alpha-order]"
year_line="$(printf '%s\n' "$out" | grep -n '#define VERSION_YEAR' | head -1 | cut -d: -f1)"
alpha_line="$(printf '%s\n' "$out" | grep -n '#define VERSION_ALPHA' | head -1 | cut -d: -f1)"
if [[ -z "$year_line" || -z "$alpha_line" ]]; then
    echo "expected VERSION_YEAR and VERSION_ALPHA in emit c"
    echo "$out"
    exit 1
fi
if [[ "$year_line" -ge "$alpha_line" ]]; then
    echo "expected VERSION_YEAR before VERSION_ALPHA"
    echo "$out"
    exit 1
fi
echo "ok"
echo

section "emit -o writes"

setup_semver "$PREFIX"
echo "[emit/output/single]"
bump emit raw -o version.env >/dev/null
if [[ ! -f version.env ]]; then
    echo "expected version.env to be written"
    exit 1
fi
if ! grep -q "VERSION_STRING=\"${PREFIX}0.1.0\"" version.env; then
    echo "unexpected version.env contents:"
    cat version.env
    exit 1
fi
echo "ok"
echo

echo "[emit/output/no-stdout]"
stdout="$(bump emit json -o out.json 2>/dev/null)"
stderr="$(bump emit json -o out.json 2>&1 >/dev/null)"
if [[ -n "$stdout" ]]; then
    echo "emit -o unexpectedly printed to stdout: $stdout"
    exit 1
fi
if [[ "$stderr" != *"written to"* ]]; then
    echo "expected status on stderr"
    echo "stderr: $stderr"
    exit 1
fi
echo "ok"
echo

section "Invalid case / format rejected"

assert_fails \
    "emit/invalid-case" \
    "invalid value 'kebab' for '--case" \
    emit raw --case kebab

assert_fails \
    "emit/invalid-format" \
    "invalid value 'ruby' for" \
    emit ruby

echo "All emit tests passed."
