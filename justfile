_:
    @just --list

# Clippy (same bar as CI), then a debug build.
build:
    cargo clippy --all-targets -- -D warnings
    cargo build

# Build docs/site, then serve it at http://127.0.0.1:8080.
site:
    echo "browse to http://127.0.0.1:8080"
    docs/build.sh
    python3 -m http.server -d docs/site 8080
