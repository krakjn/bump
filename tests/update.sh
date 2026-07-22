#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump update Cargo.toml / pyproject.toml.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git
setup_semver "$PREFIX"

section "update Cargo.toml"

cat > Cargo.toml <<'EOF'
[package]
name = "demo"
version = "0.0.0"
edition = "2021"
EOF

echo "[update/cargo/strips-prefix]"
err="$(mktemp)"
out="$(bump update Cargo.toml 2>"$err")"
stderr="$(cat "$err")"
rm -f "$err"

if [[ "$stderr" != *"stripping prefix"* && "$out" != *"stripping prefix"* ]]; then
    # message goes to stdout via println
    if [[ "$out" != *"stripping prefix"* ]]; then
        echo "expected stripping prefix message"
        echo "stdout: $out"
        echo "stderr: $stderr"
        exit 1
    fi
fi
if [[ "$out" != *"Cargo.toml updated to version 0.1.0"* ]]; then
    echo "unexpected update message: $out"
    exit 1
fi
ver="$(grep '^version = ' Cargo.toml | head -1)"
if [[ "$ver" != 'version = "0.1.0"' ]]; then
    echo "expected version without prefix, got: $ver"
    cat Cargo.toml
    exit 1
fi
echo "ok"
echo

section "update pyproject.toml"

cat > pyproject.toml <<'EOF'
[project]
name = "demo"
version = "0.0.0"
EOF

echo "[update/pyproject/keeps-prefix]"
out="$(bump update pyproject.toml)"
if [[ "$out" != *"PEP"* && "$out" != *"Public version identifiers"* && "$out" != *"N!"* ]]; then
    echo "expected PEP 440 warning in output"
    echo "$out"
    exit 1
fi
if [[ "$out" != *"pyproject.toml updated to version ${PREFIX}0.1.0"* ]]; then
    echo "unexpected update message: $out"
    exit 1
fi
ver="$(grep '^version = ' pyproject.toml | head -1)"
if [[ "$ver" != "version = \"${PREFIX}0.1.0\"" ]]; then
    echo "expected version with prefix, got: $ver"
    cat pyproject.toml
    exit 1
fi
echo "ok"
echo

echo "[update/pyproject/no-project]"
cat > pyproject.toml <<'EOF'
[tool.poetry]
name = "demo"
version = "9.9.9"
EOF
assert_fails \
    "update/pyproject/no-project" \
    "bump error >> no [project] section found" \
    update pyproject.toml
if [[ "$(grep '^version = ' pyproject.toml)" != 'version = "9.9.9"' ]]; then
    echo "poetry version should be unchanged"
    cat pyproject.toml
    exit 1
fi
echo "ok"
echo

assert_fails \
    "update/unsupported" \
    "invalid value 'package.json' for '<PATH>'" \
    update package.json

echo "All update tests passed."
