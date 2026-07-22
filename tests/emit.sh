#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump emit — formats, cases, nested markup, -o writes.

source "$(dirname "$0")/lib.sh"

PREFIX="v-"
CASES=(snake camel pascal uppercase)
LANG_FORMATS=(raw c go java csharp python)

enter_workspace --git

case_string() {
    case "$1" in
        snake) echo "version_string" ;;
        camel) echo "versionString" ;;
        pascal) echo "VersionString" ;;
        uppercase) echo "VERSION_STRING" ;;
        *) echo "unknown case: $1" >&2; exit 1 ;;
    esac
}

case_prefix() {
    case "$1" in
        snake) echo "version_prefix" ;;
        camel) echo "versionPrefix" ;;
        pascal) echo "VersionPrefix" ;;
        uppercase) echo "VERSION_PREFIX" ;;
        *) echo "unknown case: $1" >&2; exit 1 ;;
    esac
}

case_phase_distance() {
    case "$1" in
        snake) echo "version_phase_distance" ;;
        camel) echo "versionPhaseDistance" ;;
        pascal) echo "VersionPhaseDistance" ;;
        uppercase) echo "VERSION_PHASE_DISTANCE" ;;
        *) echo "unknown case: $1" >&2; exit 1 ;;
    esac
}

expected_json() {
    local root="$1"
    cat <<EOF
{
  "${root}": {
    "prefix": "${PREFIX}",
    "major": "0",
    "minor": "1",
    "patch": "0",
    "phase": "",
    "phase_distance": "0",
    "string": "${PREFIX}0.1.0",
    "timestamp": "${TIMESTAMP}"
  }
}
EOF
}

expected_toml() {
    local root="$1"
    cat <<EOF
[${root}]
prefix = "${PREFIX}"
major = "0"
minor = "1"
patch = "0"
phase = ""
phase_distance = "0"
string = "${PREFIX}0.1.0"
timestamp = "${TIMESTAMP}"
EOF
}

expected_yaml() {
    local root="$1"
    cat <<EOF
${root}:
  prefix: ${PREFIX}
  major: 0
  minor: 1
  patch: 0
  phase: ""
  phase_distance: 0
  string: ${PREFIX}0.1.0
  timestamp: "${TIMESTAMP}"
EOF
}

section "Markup nested layouts (exact)"

setup_semver "$PREFIX"

assert_eq \
    "emit/json/nested-with-prefix" \
    "$(expected_json "mylib_version")" \
    emit json --prefix "mylib_"

assert_eq \
    "emit/json/nested-no-prefix" \
    "$(expected_json "version")" \
    emit json

assert_eq \
    "emit/toml/nested-with-prefix" \
    "$(expected_toml "mylib_version")" \
    emit toml --prefix "mylib_"

assert_eq \
    "emit/toml/nested-no-prefix" \
    "$(expected_toml "version")" \
    emit toml

assert_eq \
    "emit/yaml/nested-with-prefix" \
    "$(expected_yaml "mylib_version")" \
    emit yaml --prefix "mylib_"

assert_eq \
    "emit/yaml/nested-no-prefix" \
    "$(expected_yaml "version")" \
    emit yaml

section "Markup ignores --case (always snake)"

for case in "${CASES[@]}"; do
    assert_eq \
        "emit/json/ignores-case-${case}" \
        "$(expected_json "mylib_version")" \
        emit json --prefix "mylib_" --case "$case"

    assert_eq \
        "emit/toml/ignores-case-${case}" \
        "$(expected_toml "mylib_version")" \
        emit toml --prefix "mylib_" --case "$case"

    assert_eq \
        "emit/yaml/ignores-case-${case}" \
        "$(expected_yaml "mylib_version")" \
        emit yaml --prefix "mylib_" --case "$case"
done

assert_lacks \
    "emit/json/no-flat-version-string-key" \
    "version_string" \
    emit json --prefix "mylib_" --case snake

assert_lacks \
    "emit/json/no-camel-keys" \
    "versionString" \
    emit json --prefix "mylib_" --case camel

assert_lacks \
    "emit/toml/no-flat-table" \
    "mylib_version_string" \
    emit toml --prefix "mylib_" --case snake

assert_lacks \
    "emit/yaml/no-camel-keys" \
    "versionString" \
    emit yaml --prefix "mylib_" --case camel

section "Language/raw: format × case × prefix"

setup_semver "$PREFIX"

for format in "${LANG_FORMATS[@]}"; do
    for case in "${CASES[@]}"; do
        local_prefix="mylib_"
        id_string="$(case_string "$case")"
        id_prefix="$(case_prefix "$case")"
        id_phase_distance="$(case_phase_distance "$case")"
        name="emit/${format}/${case}"

        case "$format" in
            raw)
                assert_contains \
                    "${name}/string" \
                    "${local_prefix}${id_string}=\"${PREFIX}0.1.0\"" \
                    emit raw --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "${local_prefix}${id_prefix}=\"${PREFIX}\"" \
                    emit raw --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "${local_prefix}${id_phase_distance}=0" \
                    emit raw --prefix "$local_prefix" --case "$case"
                ;;
            c)
                assert_contains \
                    "${name}/string" \
                    "#define ${local_prefix}${id_string} \"${PREFIX}0.1.0\"" \
                    emit c --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "#define ${local_prefix}${id_prefix} \"${PREFIX}\"" \
                    emit c --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "#define ${local_prefix}${id_phase_distance} 0" \
                    emit c --prefix "$local_prefix" --case "$case"
                ;;
            go)
                assert_contains \
                    "${name}/string" \
                    "${local_prefix}${id_string} = \"${PREFIX}0.1.0\"" \
                    emit go --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "${local_prefix}${id_prefix} = \"${PREFIX}\"" \
                    emit go --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "${local_prefix}${id_phase_distance} = 0" \
                    emit go --prefix "$local_prefix" --case "$case"
                ;;
            java)
                assert_contains \
                    "${name}/string" \
                    "public static final String ${local_prefix}${id_string} = \"${PREFIX}0.1.0\";" \
                    emit java --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "public static final String ${local_prefix}${id_prefix} = \"${PREFIX}\";" \
                    emit java --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "public static final int ${local_prefix}${id_phase_distance} = 0;" \
                    emit java --prefix "$local_prefix" --case "$case"
                ;;
            csharp)
                assert_contains \
                    "${name}/string" \
                    "public const string ${local_prefix}${id_string} = \"${PREFIX}0.1.0\";" \
                    emit csharp --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "public const string ${local_prefix}${id_prefix} = \"${PREFIX}\";" \
                    emit csharp --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "public const int ${local_prefix}${id_phase_distance} = 0;" \
                    emit csharp --prefix "$local_prefix" --case "$case"
                ;;
            python)
                assert_contains \
                    "${name}/string" \
                    "${local_prefix}${id_string} = \"${PREFIX}0.1.0\"" \
                    emit python --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/prefix" \
                    "${local_prefix}${id_prefix} = \"${PREFIX}\"" \
                    emit python --prefix "$local_prefix" --case "$case"
                assert_contains \
                    "${name}/phase-distance" \
                    "${local_prefix}${id_phase_distance} = 0" \
                    emit python --prefix "$local_prefix" --case "$case"
                ;;
        esac
    done
done

assert_contains \
    "emit/raw/default-uppercase" \
    "APP_VERSION_STRING=\"${PREFIX}0.1.0\"" \
    emit raw --prefix "APP_"

assert_contains \
    "emit/c/default-uppercase" \
    "#define APP_VERSION_STRING \"${PREFIX}0.1.0\"" \
    emit c --prefix "APP_"

assert_lacks \
    "emit/c/stdout/no-include-guard" \
    "#ifndef BUMP_VERSION_H" \
    emit c --prefix "APP_"

assert_lacks \
    "emit/c/stdout/no-generated-banner" \
    "This file is generated by:" \
    emit c --prefix "APP_"

assert_contains \
    "emit/raw/no-prefix-snake" \
    "version_string=\"${PREFIX}0.1.0\"" \
    emit raw --case snake

assert_contains \
    "emit/python/no-prefix-pascal" \
    "VersionString = \"${PREFIX}0.1.0\"" \
    emit python --case pascal

section "Emit calver"

setup_calver

assert_contains \
    "emit/calver/json/string-key" \
    '"string": "2020.01.01"' \
    emit json

assert_lacks \
    "emit/calver/c/stdout/no-include-guard" \
    "#ifndef BUMP_VERSION_H" \
    emit c

echo "[emit/calver/c-header-file]"
bump emit c -o calver.h 2>/dev/null
if ! grep -Fq '#ifndef BUMP_VERSION_H' calver.h; then
    echo "expected include guard in calver.h"
    cat calver.h
    exit 1
fi
echo "ok"
echo

section "emit -o writes"

setup_semver "$PREFIX"

echo "[emit/output/single]"
bump emit raw --case snake -o version.env >/dev/null
if [[ ! -f version.env ]]; then
    echo "expected version.env to be written"
    exit 1
fi
if ! grep -q "version_string=\"${PREFIX}0.1.0\"" version.env; then
    echo "unexpected version.env contents:"
    cat version.env
    exit 1
fi
echo "ok"
echo

echo "[emit/output/multi-nested]"
bump emit c --prefix "APP_" -o include/a.h -o include/nested/b.h >/dev/null
if [[ ! -f include/a.h ]] || [[ ! -f include/nested/b.h ]]; then
    echo "expected both output files"
    exit 1
fi
if ! grep -q '#define APP_VERSION_STRING' include/a.h; then
    echo "unexpected include/a.h"
    cat include/a.h
    exit 1
fi
if ! grep -Fq '#ifndef BUMP_VERSION_H' include/a.h; then
    echo "expected include guard in include/a.h"
    cat include/a.h
    exit 1
fi
if ! grep -q '#define APP_VERSION_STRING' include/nested/b.h; then
    echo "unexpected include/nested/b.h"
    cat include/nested/b.h
    exit 1
fi
if ! grep -Fq '#endif /* BUMP_VERSION_H */' include/nested/b.h; then
    echo "expected endif guard in include/nested/b.h"
    cat include/nested/b.h
    exit 1
fi
echo "ok"
echo

echo "[emit/output/no-stdout]"
stdout="$(bump emit json -o out.json 2>/dev/null)"
stderr="$(bump emit json -o out.json 2>&1 >/dev/null)"
if [[ -n "$stdout" ]]; then
    echo "emit -o unexpectedly printed to stdout: $stdout"
    exit 1
fi
if [[ "$stderr" != *"written to"* ]]; then
    echo "expected status on stderr"
    echo "stderr: $stderr"
    exit 1
fi
echo "ok"
echo

section "Invalid case / format rejected"

assert_fails \
    "emit/invalid-case-kebab" \
    "invalid value 'kebab' for '--case" \
    emit raw --case kebab

assert_fails \
    "emit/invalid-case-lowercase" \
    "invalid value 'lowercase' for '--case" \
    emit c --case lowercase

assert_fails \
    "emit/invalid-format" \
    "invalid value 'ruby' for" \
    emit ruby

echo "All emit tests passed."
