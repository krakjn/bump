# Contributing to Bump

Thank you for your interest in contributing to bump! This document covers building,
testing, and the project layout. For usage examples, see the [Workflow Guide](WORKFLOW.md)
and [Configuration Reference](CONFIGURATION.md).

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- For static Linux binaries only: `rustup target add x86_64-unknown-linux-musl`

### Building

Default release build:

```bash
git clone https://github.com/krakjn/bump.git
cd bump
cargo build --release
# binary: target/release/bump
```

Optional static Linux build with musl:

```bash
cargo build --release --target x86_64-unknown-linux-musl
# binary: target/x86_64-unknown-linux-musl/release/bump
```

### Development Build

For faster iteration during development:

```bash
cargo build
# binary: target/debug/bump
```

Integration tests require a release build (see below).

### Running Tests

Integration tests live under `tests/`. `output.sh` exercises show output across
SemVer phase and formal bumps, CalVer calendar bumps, label positions, and emit smoke.
Also run `malformed.sh` and `mode-swap.sh`.

```bash
cargo build --release
./tests/output.sh
./tests/malformed.sh
./tests/mode-swap.sh
```

When testing a cross-compiled binary, set `BUMP_BIN` to the built artifact path:

```bash
cargo build --release --target x86_64-unknown-linux-musl
BUMP_BIN=target/x86_64-unknown-linux-musl/release/bump ./tests/output.sh
```

The script reinitializes `bump.toml` in the repository root via `bump init`; any local
changes to that file are overwritten.

CI runs the shell suites on native (non-cross-compiled) Linux and macOS jobs after
`cargo build --release --target <triple>`, with `BUMP_BIN` set to
`target/<triple>/release/bump`.

## Project Structure

```
bump/
├── src/
│   ├── main.rs         # Thin 1:1 dispatch to cmd::*
│   ├── cli.rs          # Command-line interface (clap)
│   ├── cmd/            # CLI entrypoints (1:1 with commands)
│   │   ├── show.rs
│   │   ├── mutate.rs
│   │   ├── meta.rs
│   │   ├── emit.rs
│   │   ├── init.rs
│   │   ├── tag.rs
│   │   └── update.rs
│   ├── output/         # emit FORMAT backends
│   │   └── format/     # c.rs, go.rs, … + raw/json/toml/yaml
│   ├── version.rs      # Version struct and bumping rules
│   ├── compose.rs      # Version string assembly (library)
│   ├── bumpfile.rs     # Load/save bump.toml
│   └── templates/      # Init bump.toml template only
├── tests/              # Shell integration tests
├── docs/               # Documentation
├── install/            # Release install scripts
├── action.yml          # GitHub Action to install bump in workflows
├── .github/workflows/  # CI build, test, and publish
└── Cargo.toml
```

## Making Changes

1. Create a feature branch
1. Make your changes
1. Run the integration test suites to ensure everything works
1. Submit a pull request

## Questions?

Feel free to open an issue on GitHub if you have questions or need help.
