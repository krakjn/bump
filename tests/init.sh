#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump init.

source "$(dirname "$0")/lib.sh"

enter_workspace

section "init creates bumpfile"

echo "[init/default]"
out="$(bump init)"
refresh_metadata
if [[ "$out" != *"initialized"*bump.toml* ]]; then
    echo "unexpected init message: $out"
    exit 1
fi
if [[ "$out" != *"v0.1.0  ${TIMESTAMP}"* ]]; then
    echo "expected version and timestamp in init report"
    echo "out: $out"
    exit 1
fi
if [[ ! -f bump.toml ]]; then
    echo "bump.toml was not created"
    exit 1
fi
if ! grep -q '# epoch = 0' bump.toml; then
    echo "expected commented epoch hint in default bump.toml"
    exit 1
fi
assert_eq "init/default/print" "v0.1.0" p
echo "ok"
echo

echo "[init/nested-path]"
out="$(bump init nested/dir/bump.toml)"
if [[ ! -f nested/dir/bump.toml ]]; then
    echo "nested bumpfile was not created"
    echo "out: $out"
    exit 1
fi
assert_eq "init/nested/print" "v0.1.0" p nested/dir/bump.toml
echo "ok"
echo

section "init refuses overwrite without --force"

assert_fails \
    "init/overwrite-without-force" \
    "bump error >> bumpfile already exists" \
    init

assert_fails \
    "init/overwrite-without-force/nested" \
    "bump error >> bumpfile already exists" \
    init nested/dir/bump.toml

echo "[init/force-overwrite]"
bump init --force >/dev/null
assert_eq "init/force-overwrite/print" "v0.1.0" p
echo "ok"
echo

section "init --calver"

CALVER_TODAY="$(today_calver_base)"

echo "[init/calver]"
out="$(bump init --calver calver/bump.toml)"
if [[ "$out" != *"initialized"*calver/bump.toml* ]]; then
    echo "unexpected calver init message: $out"
    exit 1
fi
if [[ ! -f calver/bump.toml ]]; then
    echo "calver bumpfile was not created"
    exit 1
fi
assert_eq "init/calver/print" "$CALVER_TODAY" p calver/bump.toml
echo "ok"
echo

echo "All init tests passed."
