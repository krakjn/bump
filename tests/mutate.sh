#!/usr/bin/env bash

set -euo pipefail

# Behavior: base-key cascade, phase, mixed date+custom bumps.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"

enter_workspace --git

section "SemVer formal bumps"

setup_semver "$PREFIX"

out="$(bump patch)"
refresh_metadata
echo "[mutate/patch/message]"
echo "out: $out"
if [[ "$out" != *"bumped"* ]] || [[ "$out" != *"${PREFIX}0.1.1  ${TIMESTAMP}"* ]]; then
    echo "unexpected patch message"
    exit 1
fi
echo "ok"
echo

assert_eq "mutate/patch" "${PREFIX}0.1.1" p

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

bump patch >/dev/null
assert_eq "mutate/patch/clears-phase" "${PREFIX}0.1.1" p

section "Calendar keys sync to UTC"

setup_calver
bump date >/dev/null
assert_eq "mutate/calver/date-syncs-today" "$(today_calver_base)" p

bump date >/dev/null
assert_eq "mutate/calver/same-day-phase" "$(today_calver_base)-1" p

section "Custom key cascade"

setup_custom "$PREFIX"
assert_eq "mutate/custom/print" "${PREFIX}2.9" p
bump alpha >/dev/null
assert_eq "mutate/custom/alpha-zeros-beta" "${PREFIX}3.0" p

section "Mixed: bump custom refreshes dates, zeros later custom"

setup_mixed
today_year="$(date -u +"%Y")"
today_month="$(date -u +"%m")"
bump alpha >/dev/null
assert_eq "mutate/mixed/bump-alpha" "v${today_year}.3.${today_month}.0" p
assert_base_has_key bump.toml year
assert_base_has_key bump.toml alpha
assert_base_has_key bump.toml month
assert_base_has_key bump.toml beta

echo "All mutate tests passed."
