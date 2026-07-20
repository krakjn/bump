#!/usr/bin/env bash

# Shared helpers for bump behavior tests.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURES="$ROOT/tests/fixtures"

# Resolve BUMP_BIN to an absolute path so suites can cd into temp workspaces.
if [[ -n "${BUMP_BIN:-}" ]]; then
    if [[ "$BUMP_BIN" != /* ]]; then
        BUMP_BIN="$ROOT/$BUMP_BIN"
    fi
elif [[ -n "${CARGO_TARGET_DIR:-}" && -x "${CARGO_TARGET_DIR}/release/bump" ]]; then
    BUMP_BIN="${CARGO_TARGET_DIR}/release/bump"
elif [[ -x "$ROOT/target/release/bump" ]]; then
    BUMP_BIN="$ROOT/target/release/bump"
else
    echo "bump binary not found; build with: cargo build --release" >&2
    echo "or set BUMP_BIN to the release artifact" >&2
    exit 1
fi

if [[ ! -x "$BUMP_BIN" ]]; then
    echo "BUMP_BIN is not executable: $BUMP_BIN" >&2
    exit 1
fi

bump() {
    "$BUMP_BIN" "$@"
}

section() {
    echo "========================================"
    echo "SECTION: $1"
    echo "========================================"
}

# Enter an isolated temp workspace. Optional: enter_workspace --git
enter_workspace() {
    local with_git=0
    if [[ "${1:-}" == "--git" ]]; then
        with_git=1
    fi

    WORKSPACE="$(mktemp -d "${TMPDIR:-/tmp}/bump-test.XXXXXX")"
    # shellcheck disable=SC2064
    trap "rm -rf '$WORKSPACE'" EXIT
    cd "$WORKSPACE"

    if [[ "$with_git" -eq 1 ]]; then
        init_git
    fi
}

init_git() {
    git init -q
    git config user.email "bump-test@example.com"
    git config user.name "bump-test"
    # Detach from any template/hooks noise; one commit so HEAD exists.
    echo "# bump test" > README.md
    git add README.md
    git commit -qm "init"
}

refresh_metadata() {
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        GIT_SHA="$(git rev-parse --short HEAD)"
    else
        GIT_SHA=""
    fi
    if [[ -f bump.toml ]]; then
        TIMESTAMP="$(grep '^last = ' bump.toml | sed 's/^last = "\(.*\)"/\1/')"
    else
        TIMESTAMP=""
    fi
}

setup_semver() {
    local prefix="${1:-v-}"
    bump init >/dev/null
    bump meta --prefix "$prefix" >/dev/null
    refresh_metadata
}

setup_calver() {
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
    refresh_metadata
}

set_label_position() {
    local pos="$1"
    local file="${2:-bump.toml}"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        sed -i '' "s/^position = .*/position = \"${pos}\"/" "$file"
    else
        sed -i "s/^position = .*/position = \"${pos}\"/" "$file"
    fi
}

today_calver_base() {
    date -u +"%Y.%m.%d"
}

# --- asserts -----------------------------------------------------------------

assert_eq() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual
    actual="$(bump "$@")"
    echo "[$name]"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_contains() {
    local name="$1"
    local needle="$2"
    shift 2
    local actual
    actual="$(bump "$@")"
    echo "[$name]"
    echo "needle: $needle"
    if [[ "$actual" != *"$needle"* ]]; then
        echo "actual: $actual"
        exit 1
    fi
    echo "ok"
    echo
}

assert_lacks() {
    local name="$1"
    local needle="$2"
    shift 2
    local actual
    actual="$(bump "$@")"
    echo "[$name]"
    echo "must not contain: $needle"
    if [[ "$actual" == *"$needle"* ]]; then
        echo "actual: $actual"
        exit 1
    fi
    echo "ok"
    echo
}

assert_fails() {
    local name="$1"
    local pattern="$2"
    shift 2
    local output
    local status=0

    echo "[$name]"
    set +e
    output="$(bump "$@" 2>&1)"
    status=$?
    set -e

    if [[ "$status" -eq 0 ]]; then
        echo "expected failure, but command succeeded"
        echo "output: $output"
        exit 1
    fi

    if [[ "$output" != *"$pattern"* ]]; then
        echo "expected stderr/stdout to contain: $pattern"
        echo "got: $output"
        exit 1
    fi

    echo "ok"
    echo
}

assert_prints() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual

    echo "[$name]"
    actual="$(bump "$@")"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_print_silent() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual
    local stderr
    local tmp

    tmp="$(mktemp)"

    echo "[$name]"
    set +e
    actual="$(bump "$@" 2>"$tmp")"
    stderr="$(cat "$tmp")"
    set -e
    rm -f "$tmp"

    if [[ -n "$stderr" ]]; then
        echo "expected no stderr on show, got: $stderr"
        exit 1
    fi

    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_warns() {
    local name="$1"
    local warn_pattern="$2"
    shift 2
    local output
    local stderr
    local status=0
    local err

    err="$(mktemp)"

    echo "[$name]"
    set +e
    output="$(bump "$@" 2>"$err")"
    stderr="$(cat "$err")"
    status=$?
    set -e
    rm -f "$err"

    if [[ "$status" -ne 0 ]]; then
        echo "expected success with warning, but command failed"
        echo "stdout: $output"
        echo "stderr: $stderr"
        exit 1
    fi

    if [[ "$stderr" != *"$warn_pattern"* ]]; then
        echo "expected warning containing: $warn_pattern"
        echo "stderr: $stderr"
        exit 1
    fi

    echo "ok"
    echo
}

# Back-compat name used by schema suite: warn on patch of a fixture copy.
assert_warns_on_bump() {
    local name="$1"
    local warn_pattern="$2"
    local bumpfile="$3"
    local tmp

    tmp="$(mktemp)"
    cp "$bumpfile" "$tmp"
    assert_warns "$name" "$warn_pattern" patch "$tmp"
    rm -f "$tmp"
}

base_section() {
    awk '/^\[base\]/{flag=1; next} /^\[/{flag=0} flag' "$1"
}

assert_base_has_key() {
    local file="$1"
    local key="$2"
    if ! base_section "$file" | grep -q "^${key} ="; then
        echo "expected [base] to contain key: $key"
        echo "base section:"
        base_section "$file"
        exit 1
    fi
}

assert_base_lacks_key() {
    local file="$1"
    local key="$2"
    if base_section "$file" | grep -q "^${key} ="; then
        echo "expected [base] to lack key: $key"
        echo "base section:"
        base_section "$file"
        exit 1
    fi
}

assert_bump_rewrites_keys() {
    local name="$1"
    local bumpfile="$2"
    local bump_cmd="$3"
    local warn_pattern="$4"
    shift 4
    local want_keys=("$@")
    local tmp
    local stderr
    local status=0
    local key
    local err

    tmp="$(mktemp)"
    err="$(mktemp)"
    cp "$bumpfile" "$tmp"

    echo "[$name]"
    set +e
    bump "$bump_cmd" "$tmp" >/dev/null 2>"$err"
    stderr="$(cat "$err")"
    status=$?
    set -e
    rm -f "$err"

    if [[ "$status" -ne 0 ]]; then
        echo "bump failed"
        cat "$tmp"
        exit 1
    fi

    if [[ "$stderr" != *"$warn_pattern"* ]]; then
        echo "expected warning containing: $warn_pattern"
        echo "stderr: $stderr"
        exit 1
    fi

    for key in "${want_keys[@]}"; do
        assert_base_has_key "$tmp" "$key"
    done

    case "${want_keys[0]}" in
        major|minor|patch)
            for key in year month day; do
                assert_base_lacks_key "$tmp" "$key"
            done
            ;;
        year|month|day)
            for key in major minor patch; do
                assert_base_lacks_key "$tmp" "$key"
            done
            ;;
    esac

    rm -f "$tmp"
    echo "ok"
    echo
}
