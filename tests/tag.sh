#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump tag.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

# ---------------------------------------------------------------------------
section "tag requires git"
# ---------------------------------------------------------------------------

enter_workspace
setup_semver "$PREFIX"

assert_fails \
    "tag/not-a-git-repo" \
    "Not in a git repository" \
    tag

# Replace non-git workspace with a git one for the remaining checks.
rm -rf "$WORKSPACE"
enter_workspace --git
setup_semver "$PREFIX"

# ---------------------------------------------------------------------------
section "tag creates annotated tag"
# ---------------------------------------------------------------------------

tag_name="${PREFIX}0.1.0"

echo "[tag/default-message]"
out="$(bump tag)"
if [[ "$out" != "Created git tag: ${tag_name}" ]]; then
    echo "unexpected: $out"
    exit 1
fi
if ! git rev-parse -q --verify "refs/tags/${tag_name}" >/dev/null; then
    echo "tag was not created"
    exit 1
fi
msg="$(git tag -l --format='%(contents:subject)' "$tag_name")"
if [[ "$msg" != "chore(release): bump version to ${tag_name}" ]]; then
    echo "unexpected tag message: $msg"
    exit 1
fi
echo "ok"
echo

assert_fails \
    "tag/duplicate" \
    "already exists" \
    tag

echo "[tag/custom-message]"
bump patch >/dev/null
tag_name="${PREFIX}0.1.1"
out="$(bump tag -m "release ${tag_name}")"
if [[ "$out" != "Created git tag: ${tag_name}" ]]; then
    echo "unexpected: $out"
    exit 1
fi
msg="$(git tag -l --format='%(contents:subject)' "$tag_name")"
if [[ "$msg" != "release ${tag_name}" ]]; then
    echo "unexpected tag message: $msg"
    exit 1
fi
echo "ok"
echo

echo "All tag tests passed."
