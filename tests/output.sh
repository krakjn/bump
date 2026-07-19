#!/usr/bin/env bash

set -euo pipefail

# Integration tests for bump show output.
# Tier 1: workflow sections with full show-flag permutations (single label position).
# Tier 2: all six label slots with focused label-placement assertions.

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

assert_eq() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual
    actual="$(bump "$@")"
    echo "[$name]"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [ "$actual" != "$expected" ]; then
        exit 1
    fi
    echo
}

section_banner() {
    echo "========================================"
    echo "SECTION: $1"
    echo "========================================"
}

set_label_position() {
    local pos="$1"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        sed -i '' "s/^position = .*/position = \"${pos}\"/" bump.toml
    else
        sed -i "s/^position = .*/position = \"${pos}\"/" bump.toml
    fi
}

refresh_metadata() {
    GIT_SHA="$(git rev-parse --short HEAD)"
    TIMESTAMP="$(grep '^last = ' bump.toml | sed 's/^last = "\(.*\)"/\1/')"
}

setup_bumpfile() {
    bump init >/dev/null
    bump meta --prefix "$PREFIX" >/dev/null
    refresh_metadata
}

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

# Mirror show/print.rs label slot assembly.
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
    local section="$1"
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

    assert_eq "${section}/default" "$default" p
    assert_eq "${section}/only-prefix" "$prefix" p --only-prefix
    assert_eq "${section}/only-base" "$base" p --only-base
    assert_eq "${section}/only-base-with-label" "$base" p --only-base --with-label "$LABEL"
    assert_eq "${section}/only-phase" "$phase" p --only-phase
    assert_eq "${section}/no-prefix" "${base}${phase}" p --no-prefix
    assert_eq "${section}/no-phase" "${prefix}${base}" p --no-phase
    assert_eq "${section}/with-label" "$with_label" p --with-label "$LABEL"
    assert_eq "${section}/with-label-no-phase" "$with_label_no_phase" p --with-label "$LABEL" --no-phase
    assert_eq "${section}/with-label-no-prefix" "$with_label_no_prefix" p --with-label "$LABEL" --no-prefix
    assert_eq "${section}/with-suffix" "${default}+${GIT_SHA}" p --with-suffix
    assert_eq "${section}/with-label-with-suffix" "${with_label}+${GIT_SHA}" p --with-label "$LABEL" --with-suffix
    assert_eq "${section}/with-timestamp" "${default}  ${TIMESTAMP}" p --with-timestamp
    assert_eq "${section}/no-prefix-no-phase" "${base}" p --no-prefix --no-phase
    assert_eq "${section}/no-prefix-no-phase-with-label" "$with_label_no_prefix_no_phase" p --no-prefix --no-phase --with-label "$LABEL"
    assert_eq "${section}/no-prefix-no-phase-with-suffix-with-timestamp" "${base}+${GIT_SHA}  ${TIMESTAMP}" p --no-prefix --no-phase --with-suffix --with-timestamp
    assert_eq "${section}/full" "${default}+${GIT_SHA}  ${TIMESTAMP}" p --full
    assert_eq "${section}/full-with-label" "${with_label}+${GIT_SHA}  ${TIMESTAMP}" p --full --with-label "$LABEL"
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

init_calver() {
    cat > bump.toml <<'EOF'
prefix = ""

[base]
mode = "calver"
delimiter = "."
year = 2020
month = 1
day = 1

[phase]
separator = "-"
name = ""
delimiter = "."
distance = 0

[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "1970-01-01 00:00:00 UTC"

[label]
position = "after-base"
EOF
}

today_calver_base() {
    date -u +"%Y.%m.%d"
}

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

GIT_SHA="$(git rev-parse --short HEAD)"

# ---------------------------------------------------------------------------
# Tier 1: Phase bumping (after-base only)
# ---------------------------------------------------------------------------

section_banner "Phase bumping"

setup_bumpfile

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
# Tier 1: Formal bumping (after-base only)
# ---------------------------------------------------------------------------

section_banner "Formal bumping"

setup_bumpfile

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
# Tier 1: Calendar bumping (after-base only)
# ---------------------------------------------------------------------------

section_banner "Calendar bumping"

CALVER_TODAY="$(today_calver_base)"

init_calver
refresh_metadata

bump calendar >/dev/null
refresh_metadata
run_print_permutations "calendar/date" "" "$CALVER_TODAY" "" "0" "$DEFAULT_LABEL_POSITION"

bump calendar >/dev/null
refresh_metadata
run_print_permutations "calendar/same-day" "" "$CALVER_TODAY" "" "1" "$DEFAULT_LABEL_POSITION"

# ---------------------------------------------------------------------------
# Tier 2: Label position slots (all six)
# ---------------------------------------------------------------------------

section_banner "Label positions"

for label_pos in "${LABEL_POSITIONS[@]}"; do
    setup_bumpfile
    set_label_position "$label_pos"
    bump phase "$PHASE_NAMED" >/dev/null
    refresh_metadata
    run_label_slots "$label_pos" "$PREFIX" "0.1.0" "$PHASE_NAMED" "1"
done

# ---------------------------------------------------------------------------
# Emit smoke
# ---------------------------------------------------------------------------

section_banner "Emit"

setup_bumpfile
EMIT_TMP="$(mktemp)"
bump emit raw --prefix "VER=" >"$EMIT_TMP"
actual="$(cat "$EMIT_TMP")"
expected="VER=v-0.1.0"
echo "[emit/raw-prefix]"
echo "expected: $expected"
echo "actual:   $actual"
if [[ "$actual" != "$expected" ]]; then
    exit 1
fi
rm -f "$EMIT_TMP"
echo "ok"
echo

echo "All output tests passed."
