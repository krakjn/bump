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

Integration tests live in `tests/output.sh`. They exercise `bump print` output across
SemVer phase and formal bumps, CalVer calendar bumps, and all `[label].position` values.

```bash
cargo build --release
./tests/output.sh
```

When testing a cross-compiled binary, set `BUMP_BIN` to the built artifact path:

```bash
cargo build --release --target x86_64-unknown-linux-musl
BUMP_BIN=target/x86_64-unknown-linux-musl/release/bump ./tests/output.sh
```

The script reinitializes `bump.toml` in the repository root via `bump init`; any local
changes to that file are overwritten.

CI runs `./tests/output.sh` on native (non-cross-compiled) Linux and macOS jobs after
`cargo build --release --target <triple>`, with `BUMP_BIN` set to
`target/<triple>/release/bump`.

## Project Structure

```
bump/
├── src/
│   ├── main.rs         # Entry point and command routing
│   ├── cli.rs          # Command-line interface (clap)
│   ├── bump.rs         # Core bump, init, tag, and gen logic
│   ├── version.rs      # Version struct, TOML parsing, and bumping
│   ├── print.rs        # Print subcommand and output assembly
│   ├── lang.rs         # Code generation for multiple languages
│   ├── update.rs       # File updating (Cargo.toml, pyproject.toml)
│   └── templates/      # Embedded bump.toml and language templates
├── tests/
│   └── output.sh       # Shell integration tests for print output
├── docs/               # Documentation
├── install/            # Release install scripts (get_bump.sh, get_bump.ps1)
├── action.yml          # GitHub Action to install bump in workflows
├── .github/workflows/  # CI build, test, and publish
└── Cargo.toml
```

## Making Changes

1. Create a feature branch
1. Make your changes
1. Run `./tests/output.sh` to ensure everything works
1. Submit a pull request

## Questions?

Feel free to open an issue on GitHub if you have questions or need help.
