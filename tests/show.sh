#!/usr/bin/env bash

set -euo pipefail

# Behavior: compose / show (bare bump, show, p, print) and label slots.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"
LABEL="-tiger"
PHASE_NAMED=".big.cat"
DEFAULT_LABEL_POSITION="after-base"

LABEL_POSITIONS=(
    before-prefix
    after-prefix
    before-base
    after-base
    before-phase
    after-phase
)

enter_workspace --git

format_phase() {
    local name="$1"
    local distance="$2"
    if [[ -z "$name" && "$distance" == "0" ]]; then
        echo ""
    elif [[ -z "$name" && "$distance" -gt 0 ]]; then
        echo "-${distance}"
    elif [[ "$distance" == "0" ]]; then
        echo "-${name}"
    else
        echo "-${name}.${distance}"
    fi
}

# Mirror compose.rs label slot assembly.
# Args: prefix base phase label_pos no_prefix no_phase with_label
assemble() {
    local prefix="$1"
    local base="$2"
    local phase="$3"
    local label_pos="$4"
    local no_prefix="$5"
    local no_phase="$6"
    local with_label="$7"

    local use_prefix=1
    local use_base=1
    local use_phase=1
    [[ "$no_prefix" == "1" ]] && use_prefix=0
    [[ "$no_phase" == "1" ]] && use_phase=0

    local out=""

    label_visible() {
        local pos="$1"
        local anchor="$2"
        [[ "$with_label" == "1" && "$label_pos" == "$pos" && "$anchor" == "1" ]]
    }

    if label_visible "before-prefix" "$use_prefix"; then
        out+="$LABEL"
    fi
    if [[ "$use_prefix" == "1" ]]; then
        out+="$prefix"
    fi
    if label_visible "after-prefix" "$use_prefix"; then
        out+="$LABEL"
    elif label_visible "before-base" "$use_base"; then
        out+="$LABEL"
    fi
    if [[ "$use_base" == "1" ]]; then
        out+="$base"
    fi
    if label_visible "after-base" "$use_base"; then
        out+="$LABEL"
    elif label_visible "before-phase" "$use_phase"; then
        out+="$LABEL"
    fi
    if [[ "$use_phase" == "1" ]]; then
        out+="$phase"
    fi
    if label_visible "after-phase" "$use_phase"; then
        out+="$LABEL"
    fi

    echo -n "$out"
}

run_print_permutations() {
    local section_name="$1"
    local prefix="$2"
    local base="$3"
    local phase_name="$4"
    local phase_distance="$5"
    local label_pos="$6"

    local phase
    phase="$(format_phase "$phase_name" "$phase_distance")"

    local default
    default="$(assemble "$prefix" "$base" "$phase" "$label_pos" 0 0 0)"
    local with_label
    with_label="$(assemble "$prefix" "$base" "$phase" "$label_pos" 0 0 1)"
    local with_label_no_phase
    with_label_no_phase="$(assemble "$prefix" "$base" "" "$label_pos" 0 1 1)"
    local with_label_no_prefix
    with_label_no_prefix="$(assemble "" "$base" "$phase" "$label_pos" 1 0 1)"
    local with_label_no_prefix_no_phase
    with_label_no_prefix_no_phase="$(assemble "" "$base" "" "$label_pos" 1 1 1)"

    assert_eq "${section_name}/default" "$default" p
    assert_eq "${section_name}/only-prefix" "$prefix" p --only-prefix
    assert_eq "${section_name}/only-base" "$base" p --only-base
    assert_eq "${section_name}/only-base-with-label" "$base" p --only-base --with-label "$LABEL"
    assert_eq "${section_name}/only-phase" "$phase" p --only-phase
    assert_eq "${section_name}/no-prefix" "${base}${phase}" p --no-prefix
    assert_eq "${section_name}/no-phase" "${prefix}${base}" p --no-phase
    assert_eq "${section_name}/with-label" "$with_label" p --with-label "$LABEL"
    assert_eq "${section_name}/with-label-no-phase" "$with_label_no_phase" p --with-label "$LABEL" --no-phase
    assert_eq "${section_name}/with-label-no-prefix" "$with_label_no_prefix" p --with-label "$LABEL" --no-prefix
    assert_eq "${section_name}/with-suffix" "${default}+${GIT_SHA}" p --with-suffix
    assert_eq "${section_name}/with-label-with-suffix" "${with_label}+${GIT_SHA}" p --with-label "$LABEL" --with-suffix
    assert_eq "${section_name}/with-timestamp" "${default}  ${TIMESTAMP}" p --with-timestamp
    assert_eq "${section_name}/no-prefix-no-phase" "${base}" p --no-prefix --no-phase
    assert_eq "${section_name}/no-prefix-no-phase-with-label" "$with_label_no_prefix_no_phase" p --no-prefix --no-phase --with-label "$LABEL"
    assert_eq "${section_name}/no-prefix-no-phase-with-suffix-with-timestamp" "${base}+${GIT_SHA}  ${TIMESTAMP}" p --no-prefix --no-phase --with-suffix --with-timestamp
    assert_eq "${section_name}/full" "${default}+${GIT_SHA}  ${TIMESTAMP}" p --full
    assert_eq "${section_name}/full-with-label" "${with_label}+${GIT_SHA}  ${TIMESTAMP}" p --full --with-label "$LABEL"
}

run_label_slots() {
    local label_pos="$1"
    local prefix="$2"
    local base="$3"
    local phase_name="$4"
    local phase_distance="$5"

    local phase
    phase="$(format_phase "$phase_name" "$phase_distance")"
    local with_label
    with_label="$(assemble "$prefix" "$base" "$phase" "$label_pos" 0 0 1)"
    local with_label_no_phase
    with_label_no_phase="$(assemble "$prefix" "$base" "" "$label_pos" 0 1 1)"
    local with_label_no_prefix
    with_label_no_prefix="$(assemble "" "$base" "$phase" "$label_pos" 1 0 1)"

    assert_eq "label/${label_pos}/with-label" "$with_label" p --with-label "$LABEL"
    assert_eq "label/${label_pos}/with-label-no-phase" "$with_label_no_phase" p --with-label "$LABEL" --no-phase
    assert_eq "label/${label_pos}/with-label-no-prefix" "$with_label_no_prefix" p --with-label "$LABEL" --no-prefix
    assert_eq "label/${label_pos}/only-base-with-label" "$base" p --only-base --with-label "$LABEL"
    assert_eq "label/${label_pos}/with-suffix" "${with_label}+${GIT_SHA}" p --with-label "$LABEL" --with-suffix
    assert_eq "label/${label_pos}/full-with-label" "${with_label}+${GIT_SHA}  ${TIMESTAMP}" p --full --with-label "$LABEL"
}

# ---------------------------------------------------------------------------
section "Aliases (bare / show / p / print)"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"
DEFAULT="$(bump)"
assert_eq "show/alias/bare" "$DEFAULT"
assert_eq "show/alias/show" "$DEFAULT" show
assert_eq "show/alias/p" "$DEFAULT" p
assert_eq "show/alias/print" "$DEFAULT" print

# No trailing newline on show
echo "[show/no-trailing-newline]"
raw="$(bump p; printf '|')"
if [[ "$raw" == *$'\n'* ]]; then
    echo "show output unexpectedly contains a newline"
    printf '%q\n' "$raw"
    exit 1
fi
if [[ "$raw" != "${DEFAULT}|" ]]; then
    echo "unexpected show payload: $raw"
    exit 1
fi
echo "ok"
echo

# ---------------------------------------------------------------------------
section "Compose after phase bumps"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"

bump phase "$PHASE_NAMED" >/dev/null
refresh_metadata
run_print_permutations "phase/named" "$PREFIX" "0.1.0" "$PHASE_NAMED" "1" "$DEFAULT_LABEL_POSITION"

bump phase >/dev/null
refresh_metadata
run_print_permutations "phase/increment" "$PREFIX" "0.1.0" "$PHASE_NAMED" "2" "$DEFAULT_LABEL_POSITION"

bump phase beta >/dev/null
refresh_metadata
run_print_permutations "phase/switch-beta" "$PREFIX" "0.1.0" "beta" "1" "$DEFAULT_LABEL_POSITION"

# ---------------------------------------------------------------------------
section "Compose after formal bumps"
# ---------------------------------------------------------------------------

setup_semver "$PREFIX"

bump patch >/dev/null
refresh_metadata
run_print_permutations "formal/patch" "$PREFIX" "0.1.1" "" "0" "$DEFAULT_LABEL_POSITION"

bump minor >/dev/null
refresh_metadata
run_print_permutations "formal/minor" "$PREFIX" "0.2.0" "" "0" "$DEFAULT_LABEL_POSITION"

bump major >/dev/null
refresh_metadata
run_print_permutations "formal/major" "$PREFIX" "1.0.0" "" "0" "$DEFAULT_LABEL_POSITION"

# ---------------------------------------------------------------------------
section "Compose after calendar bumps"
# ---------------------------------------------------------------------------

CALVER_TODAY="$(today_calver_base)"

setup_calver
bump calendar >/dev/null
refresh_metadata
run_print_permutations "calendar/date" "" "$CALVER_TODAY" "" "0" "$DEFAULT_LABEL_POSITION"

bump calendar >/dev/null
refresh_metadata
run_print_permutations "calendar/same-day" "" "$CALVER_TODAY" "" "1" "$DEFAULT_LABEL_POSITION"

# ---------------------------------------------------------------------------
section "Label positions"
# ---------------------------------------------------------------------------

for label_pos in "${LABEL_POSITIONS[@]}"; do
    setup_semver "$PREFIX"
    set_label_position "$label_pos"
    bump phase "$PHASE_NAMED" >/dev/null
    refresh_metadata
    run_label_slots "$label_pos" "$PREFIX" "0.1.0" "$PHASE_NAMED" "1"
done

echo "All show tests passed."
