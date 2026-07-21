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

Integration tests live under `tests/`. Run the full behavior suite with one
entrypoint (covers print, mutate, meta, emit, init, tag, update, schema, and
completion):

```bash
cargo build --release
./tests/run.sh
```

When testing a cross-compiled binary, set `BUMP_BIN` to the built artifact path:

```bash
cargo build --release --target x86_64-unknown-linux-musl
BUMP_BIN=target/x86_64-unknown-linux-musl/release/bump ./tests/run.sh
```

Suites run in isolated temp workspaces and do not modify a repo-root `bump.toml`.

CI runs `./tests/run.sh` on native (non-cross-compiled) Linux and macOS jobs after
`cargo build --release --target <triple>`, with `BUMP_BIN` set to
`target/<triple>/release/bump`.

## Project Structure

```
bump/
├── src/
│   ├── main.rs         # Thin 1:1 dispatch to cmd::* / print
│   ├── cli.rs          # Command-line interface (clap)
│   ├── print.rs        # print command + version string assembly
│   ├── cmd/            # CLI entrypoints (1:1 with commands)
│   │   ├── mutate.rs
│   │   ├── meta.rs
│   │   ├── emit.rs
│   │   ├── init.rs
│   │   ├── tag.rs
│   │   └── update.rs
│   ├── output/         # emit FORMAT backends
│   │   └── format/     # c.rs, go.rs, … + raw/json/toml/yaml
│   ├── version.rs      # Version struct and bumping rules
│   ├── bumpfile.rs     # Load/save bump.toml
│   └── templates/      # Init bump.toml template only
├── tests/              # Behavior suites + run.sh entrypoint
├── docs/               # Documentation
├── install/            # Release install scripts
├── action.yml          # GitHub Action to install bump in workflows
├── .github/workflows/  # CI build, test, and publish
└── Cargo.toml
```

## Making Changes

1. Create a feature branch
1. Make your changes
1. Run `./tests/run.sh` to ensure everything works
1. Submit a pull request

## Questions?

Feel free to open an issue on GitHub if you have questions or need help.
