```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```
# Automatic Versioning

I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay. With a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!

## What does `bump` solve?

- **Declarative**: Human readable and modifiable `bump.toml`
- **Composable**: Construct _your own_ version — e.g. `PRODUCT.RELEASE.HOTFIX`, or mix calendar and custom keys
- **Flexible**: Multiple **BUMPFILE**s let you version several things in one repo
- **Integrated**: `bump print` and `bump emit` feed the version into source, build scripts, and other tools
- **Compatible**: `bump print --semver` outputs the first three base components for SemVer-aware tools
- **Automatic**: Designed with continuous integration in mind

> With `bump` you can stop thinking about versioning!

## Installation

**Linux, macOS, or WSL:**

```bash
curl -fsSL https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.ps1 | iex
```

## Quick Start

```bash
bump init          # creates bump.toml with SemVer defaults
bump print         # v0.1.0
bump patch         # 0.1.0 -> 0.1.1
bump print --full  # prefix + base + phase + suffix + timestamp (needs git for suffix)
```

Pass a path to use a non-default **BUMPFILE** on any command: `bump patch lib/bump.toml`. (can be renamed)

## Bumpfile

`bump init` writes a **BUMPFILE** (default `bump.toml`). You can rename it or keep several in one repo.

| Section | Purpose |
|---------|---------|
| `prefix` | Literal prefix prepended to the composed version |
| `[base]` | Version numbers — keys and TOML order define cascade behavior |
| `[phase]` | Pre-release phase name and distance (e.g. `-alpha.1`) |
| `[suffix]` | Git metadata appended at print time (`git_sha` or `branch`) |
| `[timestamp]` | Last-bump timestamp stored in the file and shown with `--with timestamp` |
| `[label]` | Where an ephemeral `--label` value is injected at print time |

**Base keys** are yours to define. TOML order is cascade order: bumping a key increments it and zeroes everything after it.

- **SemVer**: `major`, `minor`, `patch` (defaults from `bump init`)
- **CalVer**: `year`, `month`, `day` — these sync to the current UTC date on every bump
- **Custom**: any names you want, e.g. `alpha`, `beta`, `gamma`

Date keys print zero-padded (`2026.02.05`); other keys print as integers.

## Commands

Subcommands for base keys (`major`, `year`, `alpha`, …) are generated from your bumpfile. Run `bump --help` in a repo to see the list for that file.

### Print

Prints the composed version **without a trailing newline**. Aliases: `p`, `show`, `s`.

Default output is `[prefix][base][phase]`. Compose with flags:

```bash
bump print [BUMPFILE]
bump p --only base
bump print --without prefix
bump print --with suffix
bump print --with timestamp
bump print --full                              # all components
bump print --semver                            # first three base keys only
bump print --label DEV                         # inject at [label].position
bump print --without prefix --with suffix      # flags stack
```

`--with` / `--without` / `--only` accept: `prefix`, `base`, `phase`, `suffix`, `timestamp`.

Suffix output (`--with suffix`, `--full`) requires a git repository.

### Bump

```bash
# SemVer (updates BUMPFILE)
bump major     # 1.0.0 -> 2.0.0
bump minor     # 1.0.0 -> 1.1.0
bump patch     # 1.0.0 -> 1.0.1

# CalVer — bump a date key to refresh UTC calendar values
bump day       # e.g. 2026.02.25
bump year

# Custom keys cascade by TOML order
bump alpha     # increments alpha, zeroes keys listed after it

# Conditional bump (monorepo): skip if the bumpfile directory did not change
bump patch --if-changed-from main lib/bump.toml
bump patch --if-changed-from HEAD~1 lib/bump.toml

# Phase workflow
bump phase alpha   # 1.1.0 -> 1.1.0-alpha.1
bump phase         # increment distance, e.g. 1.1.0-alpha.2
bump phase beta    # switch phase, e.g. 1.1.0-beta.1
```

Formal base bumps clear phase.

### Metadata

```bash
bump meta --prefix v2-
bump meta --suffix branch
bump meta --prefix v- --suffix git_sha
```

Suffix modes: `git_sha` (7-char commit SHA) or `branch`.

### Emit

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues.

```bash
# Language templates → files
bump emit c -o version.h [BUMPFILE]
bump emit go -o version.go
bump emit java -o Version.java
bump emit csharp -o Version.cs
bump emit python -o version.py

# Multiple outputs
bump emit c -o version.h -o include/version.h

# Structured markup (json/toml/yaml): nested under version; --case ignored
bump emit json
bump emit toml -o version.toml
bump emit yaml

# Language/raw: --prefix and --case shape identifier names
bump emit c --prefix "MYLIB_" --case uppercase -o version.h
# → #define MYLIB_VERSION_STRING "…"
bump emit raw --prefix "app_" --case camel
# → app_versionString="…"
```

Formats: `raw`, `c`, `java`, `csharp`, `go`, `python`, `json`, `toml`, `yaml`.

`--case`: `snake` | `camel` | `pascal` | `uppercase` (default) for language/raw identifiers; ignored for json/toml/yaml.

### Git integration

```bash
bump init [--force] [BUMPFILE]

# Annotated tag for the current version (conventional commit message by default)
bump tag [BUMPFILE]
bump tag -m "Custom message" [BUMPFILE]
```

### Update package manifests

Supports `Cargo.toml` and `pyproject.toml`:

```bash
bump update Cargo.toml [BUMPFILE]
bump update pyproject.toml [BUMPFILE]
```

## GitHub Actions

The composite action `action.yml` installs bump for the job's OS/arch:

```yaml
- uses: krakjn/bump@v9
```

Pass a custom token to avoid unauthenticated GitHub API rate limits:

```yaml
- uses: krakjn/bump@v9
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

## Tips and Tricks

Print output has **no trailing newline** — safe for `$(bump p)`, Make variables, and CMake `OUTPUT_VARIABLE`.

### Shell substitution

```bash
# In-place replace a placeholder
sed -i "s|REPLACE_ME|$(bump print --without prefix)|g" somefile

# Commit message with live version
git commit -m "chore(release): $(bump p)"

# Export for downstream tools
export APP_VERSION="$(bump print --semver --only base)"
```

### CMake

```cmake
execute_process(
  COMMAND bump print --only base
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/
  OUTPUT_VARIABLE VERSION)
project("your-app" VERSION ${VERSION} LANGUAGES CXX C)
```

CMake expects numeric `MAJOR.MINOR.PATCH`; use `--only base` or `--semver --only base` depending on your bumpfile keys.

### Makefile

```makefile
VERSION := $(shell bump print --without prefix)
FULL    := $(shell bump print --full)

build: emit
	cargo build

emit:
	bump emit c --prefix MYAPP_ -o include/version.h
```

### CI build metadata

Inject run info without persisting it — labels are print-time only:

```bash
bump print --label "-${GITHUB_RUN_NUMBER}"
bump print --full --label "+ci"
bump print --without prefix --with suffix   # base+phase+git sha, no v prefix
```

### SemVer-aware tooling

When your bumpfile has more than three base keys, trim for tools that expect `X.Y.Z`:

```bash
bump print --semver              # v1.2.3 from a longer base
bump print --semver --only base  # 1.2.3 — no prefix, for Cargo.toml via bump update
```

`bump update Cargo.toml` strips the prefix automatically; use `--semver` when hand-wiring other tools.

### Emit at build time

Prefer emitting during CI/build over committing generated headers. Add outputs to `.gitignore`:

```gitignore
version.h
version.json
```

```bash
bump emit c -o version.h
bump emit json | jq .version.string
bump emit raw --prefix "APP_" --case snake
```

### Multiple bumpfiles

Default is `bump.toml` in the current directory. Pass a path last on any command:

```bash
bump patch lib/bump.toml
bump print app/bump.toml
bump tag lib/bump.toml
```

Run `bump --help` from each directory (or with each bumpfile path) to see that file's base-key subcommands.

### Monorepo selective bumps

Skip unchanged modules in CI — exits 0 with a warning when nothing changed:

```bash
bump patch --if-changed-from origin/main packages/foo/bump.toml
```

### Docker

```bash
docker build --build-arg VERSION="$(bump print --without prefix)" .
```

```dockerfile
ARG VERSION=dev
LABEL org.opencontainers.image.version="${VERSION}"
```

### Pre-release iteration

Use phases for rc/beta cycles; promote with a formal base bump (which clears phase):

```bash
bump phase rc      # 1.0.0 -> 1.0.0-rc.1
bump phase         # 1.0.0-rc.2
bump minor         # 1.0.0-rc.2 -> 1.1.0 (phase cleared)
```

## Documentation

- **[Workflow Guide](docs/WORKFLOW.md)** — release pipelines, phases, labels, monorepo patterns, and CI examples
- **[Breaking Changes](docs/BREAKING_CHANGES.md)** — migration notes between major versions

## Development

Requires Rust 1.85+.

```bash
cargo test          # unit tests
./tests/run.sh      # integration test suites (builds debug binary if needed)
```

## [MIT License](./LICENSE)
