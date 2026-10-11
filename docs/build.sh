#!/usr/bin/env sh
# Usage: docs/build.sh [build|serve]
set -eu

cd "$(dirname "$0")"
export PATH="$HOME/.local/bin:$PATH"

# MkDocs 2 is a pre-release: without --prerelease allow, uv silently installs 1.x.
if ! uv tool list 2>/dev/null | grep -q '^mkdocs v2\.'; then
  uv tool install 'mkdocs>=2.0.0.dev0' --prerelease allow --python 3.12
fi

case "${1:-build}" in
  build) rm -rf site && mkdocs build --dir pages --output site ;;
  serve) mkdocs serve --dir pages ;;
  *) echo "usage: $0 [build|serve]" >&2; exit 2 ;;
esac
