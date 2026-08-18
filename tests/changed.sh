#!/usr/bin/env bash

set -euo pipefail

# Behavior: --if-changed-from TREEISH gates semver bumps.

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
out="$(bump patch --if-changed-from HEAD~1 lib/bump.toml 2>&1)"
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
out="$(bump patch --if-changed-from HEAD~1 lib/bump.toml 2>&1)"
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

section "Custom from ref"

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
out="$(bump patch --if-changed-from "$LIB_CHANGE_SHA" lib/bump.toml 2>&1)"
echo "[changed/from-lib-change]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped (no lib changes since lib commit)"
    exit 1
fi

out="$(bump patch --if-changed-from HEAD~1 lib/bump.toml 2>&1)"
echo "[changed/from-head-1]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped on latest commit (other only)"
    exit 1
fi

out="$(bump patch --if-changed-from HEAD~2 lib/bump.toml 2>&1)"
echo "[changed/from-head-2]"
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

section "Compare from branch"

enter_workspace --git
setup_monorepo_lib
git branch -m main
git checkout -qb feature
echo "other only" > other/branch.txt
git add other/branch.txt
git commit -qm "change other on feature"

ver_before="$(bump p lib/bump.toml)"
out="$(bump patch --if-changed-from main lib/bump.toml 2>&1)"
echo "[changed/from-main]"
echo "out: $out"
if [[ "$out" != *"skipped"* ]]; then
    echo "expected skipped (lib unchanged since main)"
    exit 1
fi

echo "lib feature" > lib/feature.txt
git add lib/feature.txt
git commit -qm "change lib on feature"

out="$(bump patch --if-changed-from main lib/bump.toml 2>&1)"
echo "[changed/from-main-after-lib-change]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bump (lib changed since main)"
    exit 1
fi
ver_after="$(bump p lib/bump.toml)"
if [[ "$ver_before" == "$ver_after" ]]; then
    echo "expected version to change"
    exit 1
fi
echo "ok"
echo

section "Root bumpfile watches repo"

enter_workspace --git
bump init >/dev/null
bump meta --prefix "$PREFIX" >/dev/null
git add bump.toml
git commit -qm "add root bumpfile"
echo "root change" > root.txt
git add root.txt
git commit -qm "change root"

out="$(bump patch --if-changed-from HEAD~1 2>&1)"
echo "[changed/root-bumpfile]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]]; then
    echo "expected bump for root bumpfile"
    exit 1
fi
echo "ok"
echo

section "Unknown ref fails"

enter_workspace
bump init >/dev/null
git init -q
git config user.email "bump-test@example.com"
git config user.name "bump-test"
git add bump.toml
git commit -qm "init with bumpfile"

assert_fails \
    "changed/unknown-ref" \
    "unknown git ref" \
    patch --if-changed-from HEAD~1

section "Requires git repository"

enter_workspace
bump init >/dev/null

assert_fails \
    "changed/not-git" \
    "Not a git repository" \
    patch --if-changed-from HEAD

echo "changed.sh: all tests passed."
