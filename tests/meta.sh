#!/usr/bin/env bash

set -euo pipefail

# Behavior: meta --prefix / --suffix.

source "$(dirname "$0")/lib.sh"

enter_workspace --git

# ---------------------------------------------------------------------------
section "meta --prefix"
# ---------------------------------------------------------------------------

bump init >/dev/null
assert_eq "meta/prefix/sets" "v-0.1.0" meta --prefix "v-"
assert_eq "meta/prefix/persists" "v-0.1.0" p

assert_eq "meta/prefix/change" "rel-0.1.0" meta --prefix "rel-"
assert_eq "meta/prefix/change-persists" "rel-0.1.0" p

# ---------------------------------------------------------------------------
section "meta --suffix"
# ---------------------------------------------------------------------------

setup_semver "v-"
assert_eq "meta/suffix/git-sha" "v-0.1.0" meta --suffix git_sha
# with-suffix uses persisted mode
refresh_metadata
assert_eq "meta/suffix/git-sha-with-suffix" "v-0.1.0+${GIT_SHA}" p --with-suffix

branch="$(git rev-parse --abbrev-ref HEAD)"
assert_eq "meta/suffix/branch" "v-0.1.0" meta --suffix branch
assert_eq "meta/suffix/branch-with-suffix" "v-0.1.0+${branch}" p --with-suffix

# ---------------------------------------------------------------------------
section "meta requires a flag"
# ---------------------------------------------------------------------------

assert_fails \
    "meta/no-flags" \
    "meta requires at least one of --prefix or --suffix" \
    meta

assert_fails \
    "meta/invalid-suffix" \
    "invalid value 'nonsense' for '--suffix" \
    meta --suffix nonsense

echo "All meta tests passed."
