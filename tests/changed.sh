#!/usr/bin/env bash

set -euo pipefail

# Behavior: --if-changed and --if-changed-since gate semver bumps.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

setup_monorepo_lib() {
    mkdir -p lib other
    bump init lib/bump.toml >/dev/null
    bump meta --prefix "$PREFIX" lib/bump.toml >/dev/null
    echo "lib source" > lib/src.txt
    echo "other source" > other/src.txt
    git add lib other
    git commit -qm "add lib and other modules"
}

section "Skip when latest commit did not touch module"

enter_workspace --git
setup_monorepo_lib
echo "other only" > other/change.txt
git add other/change.txt
git commit -qm "change other only"

ver_before="$(bump p lib/bump.toml)"
out="$(bump patch --if-changed lib/bump.toml 2>&1)"
echo "[changed/skip-unchanged]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped warning"
    exit 1
fi
ver_after="$(bump p lib/bump.toml)"
if [[ "$ver_before" != "$ver_after" ]]; then
    echo "expected version unchanged: $ver_before vs $ver_after"
    exit 1
fi
echo "ok"
echo

section "Bump when latest commit touched module"

enter_workspace --git
setup_monorepo_lib
echo "other only" > other/change.txt
git add other/change.txt
git commit -qm "change other only"

echo "lib change" > lib/change.txt
git add lib/change.txt
git commit -qm "change lib"

ver_before="$(bump p lib/bump.toml)"
out="$(bump patch --if-changed lib/bump.toml 2>&1)"
echo "[changed/bump-changed]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bumped"
    exit 1
fi
ver_after="$(bump p lib/bump.toml)"
if [[ "$ver_before" == "$ver_after" ]]; then
    echo "expected version to change"
    exit 1
fi
echo "ok"
echo

section "Custom since ref"

enter_workspace --git
setup_monorepo_lib
echo "lib change" > lib/change.txt
git add lib/change.txt
git commit -qm "change lib"
LIB_CHANGE_SHA="$(git rev-parse HEAD)"
echo "other only" > other/change.txt
git add other/change.txt
git commit -qm "change other only"

ver_before="$(bump p lib/bump.toml)"
out="$(bump patch --if-changed-since "$LIB_CHANGE_SHA" lib/bump.toml 2>&1)"
echo "[changed/since-lib-change]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped (no lib changes since lib commit)"
    exit 1
fi

out="$(bump patch --if-changed-since HEAD~1 lib/bump.toml 2>&1)"
echo "[changed/since-head-1]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped on latest commit (other only)"
    exit 1
fi

out="$(bump patch --if-changed-since HEAD~2 lib/bump.toml 2>&1)"
echo "[changed/since-head-2]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bump (lib changed in range)"
    exit 1
fi
ver_after="$(bump p lib/bump.toml)"
if [[ "$ver_before" == "$ver_after" ]]; then
    echo "expected version to change"
    exit 1
fi
echo "ok"
echo

section "Mutually exclusive flags"

enter_workspace --git
setup_monorepo_lib

assert_fails \
    "changed/both-flags" \
    "if-changed" \
    patch --if-changed --if-changed-since HEAD lib/bump.toml

section "Root bumpfile watches repo"

enter_workspace --git
bump init >/dev/null
bump meta --prefix "$PREFIX" >/dev/null
git add bump.toml
git commit -qm "add root bumpfile"
echo "root change" > root.txt
git add root.txt
git commit -qm "change root"

out="$(bump patch --if-changed 2>&1)"
echo "[changed/root-bumpfile]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bump for root bumpfile"
    exit 1
fi
echo "ok"
echo

section "Root commit warns and bumps"

enter_workspace
bump init >/dev/null
bump meta --prefix "$PREFIX" >/dev/null
git init -q
git config user.email "bump-test@example.com"
git config user.name "bump-test"
git add bump.toml
git commit -qm "init with bumpfile"

ver_before="$(bump p)"
out="$(bump patch --if-changed 2>&1)"
echo "[changed/root-commit]"
echo "out: $out"
if [[ "$out" != *"no parent commit"* ]]; then
    echo "expected no parent commit warning"
    exit 1
fi
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bump despite no parent"
    exit 1
fi
ver_after="$(bump p)"
if [[ "$ver_before" == "$ver_after" ]]; then
    echo "expected version to change"
    exit 1
fi
echo "ok"
echo

section "Requires git repository"

enter_workspace
bump init >/dev/null

assert_fails \
    "changed/not-git" \
    "Not a git repository" \
    patch --if-changed

echo "changed.sh: all tests passed."
