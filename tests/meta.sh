#!/usr/bin/env bash

set -euo pipefail

# Behavior: meta --prefix / --suffix.

source "$(dirname "$0")/lib.sh"

enter_workspace --git

section "meta --prefix"

bump init >/dev/null
refresh_metadata
assert_bumpfile_report "meta/prefix/sets" "updated" meta --prefix "v-"
assert_eq "meta/prefix/persists" "v-0.1.0" p

assert_bumpfile_report "meta/prefix/change" "updated" meta --prefix "rel-"
assert_eq "meta/prefix/change-persists" "rel-0.1.0" p

section "meta --suffix"

setup_semver "v-"
assert_bumpfile_report "meta/suffix/git-sha" "updated" meta --suffix git_sha
refresh_metadata
assert_eq "meta/suffix/git-sha-with-suffix" "v-0.1.0+${GIT_SHA}" p --with-suffix

branch="$(git rev-parse --abbrev-ref HEAD)"
assert_bumpfile_report "meta/suffix/branch" "updated" meta --suffix branch
assert_eq "meta/suffix/branch-with-suffix" "v-0.1.0+${branch}" p --with-suffix

section "meta combined flags"

assert_bumpfile_report "meta/combined" "updated" meta --prefix "pre-" --suffix git_sha
assert_eq "meta/combined/persists-prefix" "pre-0.1.0" p

section "meta requires a flag"

assert_fails \
    "meta/no-flags" \
    "bump error >> meta requires at least one of --prefix or --suffix" \
    meta

assert_fails \
    "meta/invalid-suffix" \
    "invalid value 'nonsense' for '--suffix" \
    meta --suffix nonsense

echo "All meta tests passed."
