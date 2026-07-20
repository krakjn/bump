#!/usr/bin/env bash

set -euo pipefail

# Behavior: major | minor | patch | phase | calendar mutations.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git

# ---------------------------------------------------------------------------
section "SemVer formal bumps"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"
bump phase beta >/dev/null
assert_eq "mutate/pre-patch-has-phase" "${PREFIX}0.1.0-beta.1" p

out="$(bump patch)"
refresh_metadata
echo "[mutate/patch/message]"
echo "out: $out"
if [[ "$out" != *"bumped"*bump.toml*"to ${PREFIX}0.1.1  ${TIMESTAMP}"* ]]; then
    # path may be absolute; require bumped + version + timestamp
    if [[ "$out" != *"bumped"* ]] || [[ "$out" != *"${PREFIX}0.1.1  ${TIMESTAMP}"* ]]; then
        echo "unexpected patch message"
        exit 1
    fi
fi
echo "ok"
echo

assert_eq "mutate/patch/clears-phase" "${PREFIX}0.1.1" p

bump minor >/dev/null
refresh_metadata
assert_eq "mutate/minor" "${PREFIX}0.2.0" p

bump major >/dev/null
refresh_metadata
assert_eq "mutate/major" "${PREFIX}1.0.0" p

# ---------------------------------------------------------------------------
section "Phase bumps"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"

bump phase rc >/dev/null
assert_eq "mutate/phase/named" "${PREFIX}0.1.0-rc.1" p

bump phase >/dev/null
assert_eq "mutate/phase/increment" "${PREFIX}0.1.0-rc.2" p

bump phase alpha >/dev/null
assert_eq "mutate/phase/switch" "${PREFIX}0.1.0-alpha.1" p

# ---------------------------------------------------------------------------
section "Calendar bumps"
# ---------------------------------------------------------------------------

CALVER_TODAY="$(today_calver_base)"

setup_calver
bump calendar >/dev/null
assert_eq "mutate/calendar/date" "$CALVER_TODAY" p

bump cal >/dev/null
assert_eq "mutate/calendar/alias-cal-same-day" "${CALVER_TODAY}-1" p

# ---------------------------------------------------------------------------
section "Wrong-mode errors"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"
assert_fails \
    "mutate/calendar-on-semver" \
    "version.type = 'calver'" \
    calendar

setup_calver
assert_fails \
    "mutate/major-on-calver" \
    "version.type = 'semver'" \
    major

assert_fails \
    "mutate/patch-on-calver" \
    "version.type = 'semver'" \
    patch

echo "All mutate tests passed."
