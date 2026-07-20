#!/usr/bin/env bash

set -euo pipefail

# Behavior: bump completion — thin smoke check.

source "$(dirname "$0")/lib.sh"

echo "[completion/bash]"
out="$(bump completion bash)"
if [[ -z "$out" ]]; then
    echo "expected non-empty bash completion script"
    exit 1
fi
if [[ "$out" != *"bump"* ]]; then
    echo "completion script does not mention bump"
    exit 1
fi
echo "ok"
echo

echo "[completion/zsh]"
out="$(bump completion zsh)"
if [[ -z "$out" ]]; then
    echo "expected non-empty zsh completion script"
    exit 1
fi
echo "ok"
echo

assert_fails \
    "completion/invalid-shell" \
    "invalid value" \
    completion not-a-shell

echo "All completion tests passed."
