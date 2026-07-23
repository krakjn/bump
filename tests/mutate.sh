#!/usr/bin/env bash

set -euo pipefail

# Behavior: major | minor | patch | phase | calendar mutations.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git

section "SemVer formal bumps"

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

section "Phase bumps"

setup_semver "$PREFIX"

bump phase rc >/dev/null
assert_eq "mutate/phase/named" "${PREFIX}0.1.0-rc.1" p

bump phase >/dev/null
assert_eq "mutate/phase/increment" "${PREFIX}0.1.0-rc.2" p

bump phase alpha >/dev/null
assert_eq "mutate/phase/switch" "${PREFIX}0.1.0-alpha.1" p

section "Calendar bumps"

CALVER_TODAY="$(today_calver_base)"

setup_calver
bump calendar >/dev/null
assert_eq "mutate/calendar/date" "$CALVER_TODAY" p

bump cal >/dev/null
assert_eq "mutate/calendar/alias-cal-same-day" "${CALVER_TODAY}-1" p

section "Wrong-mode errors"

setup_semver "$PREFIX"
assert_fails \
    "mutate/calendar-on-semver" \
    "base.mode = 'calver'" \
    calendar

setup_calver
assert_fails \
    "mutate/major-on-calver" \
    "base.mode = 'semver'" \
    major

assert_fails \
    "mutate/patch-on-calver" \
    "base.mode = 'semver'" \
    patch

section "Optional semver base keys — major only"

setup_semver_major_only "$PREFIX"

assert_eq "mutate/optional/semver/major-only/print" "${PREFIX}0" p

bump major >/dev/null
refresh_metadata
assert_eq "mutate/optional/semver/major-only/major" "${PREFIX}1" p
assert_base_lacks_key bump.toml minor
assert_base_lacks_key bump.toml patch

assert_fails \
    "mutate/optional/semver/major-only/minor" \
    "version.minor is set" \
    minor

assert_fails \
    "mutate/optional/semver/major-only/patch" \
    "version.patch is set" \
    patch

bump phase beta >/dev/null
assert_eq "mutate/optional/semver/major-only/phase" "${PREFIX}1-beta.1" p

section "Optional semver base keys — no patch (major + minor)"

setup_semver_no_patch "$PREFIX"

assert_eq "mutate/optional/semver/no-patch/print" "${PREFIX}0.1" p

bump minor >/dev/null
refresh_metadata
assert_eq "mutate/optional/semver/no-patch/minor" "${PREFIX}0.2" p
assert_base_has_key bump.toml minor
assert_base_lacks_key bump.toml patch

assert_fails \
    "mutate/optional/semver/no-patch/patch" \
    "version.patch is set" \
    patch

bump major >/dev/null
refresh_metadata
assert_eq "mutate/optional/semver/no-patch/major" "${PREFIX}1.0" p
assert_base_has_key bump.toml minor
assert_base_lacks_key bump.toml patch

bump phase rc >/dev/null
assert_eq "mutate/optional/semver/no-patch/phase" "${PREFIX}1.0-rc.1" p

section "Optional semver base keys — no minor (major + patch)"

setup_semver_no_minor "$PREFIX"

assert_eq "mutate/optional/semver/no-minor/print" "${PREFIX}0.0" p

assert_fails \
    "mutate/optional/semver/no-minor/minor" \
    "version.minor is set" \
    minor

bump patch >/dev/null
refresh_metadata
assert_eq "mutate/optional/semver/no-minor/patch" "${PREFIX}0.1" p
assert_base_lacks_key bump.toml minor
assert_base_has_key bump.toml patch

bump major >/dev/null
refresh_metadata
assert_eq "mutate/optional/semver/no-minor/major-resets-patch" "${PREFIX}1.0" p
assert_base_lacks_key bump.toml minor
assert_base_has_key bump.toml patch

section "Optional calver base keys — year only"

setup_calver_year_only

assert_eq "mutate/optional/calver/year-only/print" "2020" p

bump calendar >/dev/null
refresh_metadata
CALVER_YEAR="$(date -u +"%Y")"
assert_eq "mutate/optional/calver/year-only/calendar" "$CALVER_YEAR" p
assert_base_has_key bump.toml year
assert_base_lacks_key bump.toml month
assert_base_lacks_key bump.toml day

bump calendar >/dev/null
refresh_metadata
assert_eq "mutate/optional/calver/year-only/calendar-same-year" "${CALVER_YEAR}-1" p
assert_base_lacks_key bump.toml month
assert_base_lacks_key bump.toml day

section "Optional calver base keys — no day (year + month)"

setup_calver_no_day

assert_eq "mutate/optional/calver/no-day/print" "2020.01" p

bump calendar >/dev/null
refresh_metadata
CALVER_MONTH="$(date -u +"%Y.%m")"
assert_eq "mutate/optional/calver/no-day/calendar" "$CALVER_MONTH" p
assert_base_has_key bump.toml year
assert_base_has_key bump.toml month
assert_base_lacks_key bump.toml day

echo "All mutate tests passed."
