# Workflow Guide

Practical patterns for day-to-day use. For bumpfile fields and print flags, see the
[README](../README.md#bumpfile).

Upgrading from v8? See [Breaking Changes](BREAKING_CHANGES.md#v8--v9).

## Single BUMPFILE Pipeline

Use this for a single `bump.toml` at repository root.

### 1. Bump, update manifest, tag, and push

```bash
bump minor
bump update Cargo.toml

git add bump.toml Cargo.toml
git commit -m "chore(release): bump version to $(bump p)"

bump tag
git push origin HEAD --tags
```

### 2. Emit version files during builds

Generated files reflect the bumpfile at build time. Add them to `.gitignore` when you emit on every build so you never commit a stale version.

```bash
bump emit c -o include/version.h
bump emit json -o version.json

# Then run your normal build
<build tool> ...
```

## SemVer Release Workflow

### Patch release (bugfix)

```bash
bump patch
bump update Cargo.toml
git add bump.toml Cargo.toml
git commit -m "chore(release): $(bump p)"
bump tag
git push origin HEAD --follow-tags
```

### Minor release with pre-release phases

Phases are free-form labels with an incrementing distance counter.

```bash
# Start a release candidate phase on the current base
bump phase rc         # e.g. 1.4.0 -> 1.4.0-rc.1

# Iterate the same phase
bump phase            # e.g. 1.4.0-rc.2
bump phase rc         # same when name matches — increments distance

# Switch phase name
bump phase beta       # e.g. 1.4.0-beta.1

# Ship: formal base bump clears phase and promotes the release
bump minor            # e.g. 1.4.0-beta.1 -> 1.5.0
bump tag
```

### Hotfix on a release branch

```bash
git checkout release/1.4
bump patch
bump update Cargo.toml
git commit -am "chore(release): hotfix $(bump p)"
bump tag -m "Hotfix $(bump p)"
git push origin HEAD --tags
```

## CalVer Workflow

Define date keys under `[base]`:

```toml
[base]
delimiter = "."
year = 2026
month = 2
day = 25
```

Bump a date key via `bump date` (year/month/day are not individual commands). Date keys also refresh on every other base bump as a side effect — that is **not** a date bump, so phase is cleared rather than incremented.

```bash
bump date     # sync year/month/day to current UTC

# Same UTC day: increment phase (intraday counter)
bump date     # e.g. 2026.02.25 -> 2026.02.25-1

# Explicit phase command is still available
bump phase    # e.g. 2026.02.25-2
```

Mixed bumpfiles: `bump alpha` updates date keys to UTC and **clears** phase. Only a second `bump date` same-day increments phase.

## Custom and Mixed Base Keys

TOML order is cascade order. Keys after the bumped key reset to zero.

```toml
[base]
delimiter = "."
year = 2026
alpha = 1
month = 2
beta = 3
```

```bash
bump alpha    # increments alpha, zeroes later custom keys; date keys sync to UTC (phase cleared)
bump print    # e.g. v2026.2.02.0
bump print --semver   # first three keys: v2026.2.02
```

Useful for product.release.hotfix schemes without SemVer names:

```toml
[base]
delimiter = "."
product = 3
release = 12
hotfix = 0
```

```bash
bump hotfix   # 3.12.0 -> 3.12.1
bump release  # 3.12.1 -> 3.13.0, hotfix zeroed
```

## Ephemeral Labels

Labels inject at print time only — never persisted to the bumpfile. Set `[label].position` to control placement.

```bash
# [label].position = "after-base"
bump print --label DEV                 # e.g. v1.0.0DEV
bump print --full --label BUILD_ID     # label + suffix + timestamp

# CI: inject run metadata without touching bump.toml
bump print --label "-${GITHUB_RUN_NUMBER}"
bump print --without prefix --label "+${GITHUB_SHA::7}"
```

Positions: `before-prefix`, `after-prefix`, `before-base`, `after-base`, `before-phase`, `after-phase`.

## Multiple BUMPFILE Pipeline

Version several components in one repository. Pass the file path as the trailing `BUMPFILE` argument.

```bash
bump minor lib/bump.toml
bump major app/bump.toml

git add -u
git commit -m "chore(release): bump component versions"

bump tag lib/bump.toml
bump tag app/bump.toml
git push origin HEAD --follow-tags
```

Each bumpfile directory is independent for `--if-changed-from` checks.

## Conditional Bump (Monorepo)

Place `bump.toml` at the module root (`lib/bump.toml` watches `lib/`). Pass a git ref — bump runs only when files under that directory changed from the ref to `HEAD`:

```bash
# Changes on this branch since main?
bump patch --if-changed-from main lib/bump.toml

# Changes in the latest commit only?
bump patch --if-changed-from HEAD~1 lib/bump.toml

bump update lib/Cargo.toml lib/bump.toml
```

Exits 0 when skipping (stderr warning only). Safe for CI matrices that bump only touched packages.

### GitHub Actions monorepo example

```yaml
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: krakjn/bump@v9

      - name: Bump changed packages
        run: |
          bump patch --if-changed-from origin/main lib/bump.toml
          bump patch --if-changed-from origin/main app/bump.toml

      - name: Commit if bumped
        run: |
          git diff --quiet bump.toml lib/bump.toml app/bump.toml && exit 0
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add lib/bump.toml app/bump.toml
          git commit -m "chore(release): bump changed packages"
          git push
```

## CI-Friendly Version Output

All print commands emit **without a trailing newline** — safe for shell substitution and `OUTPUT_VARIABLE` in CMake.

```bash
bump print --only base              # numeric base only
bump print --without prefix         # base + phase, no leading v
bump print --semver                 # first three base keys
bump print --full                   # everything including suffix + timestamp
bump print --with suffix            # needs git checkout
bump print --label "$BUILD_ID"
bump emit raw --prefix "APP_"
bump emit c --prefix "APP_" --case uppercase -o version.h
```

Suffix output (`--with suffix`, `--full`) requires a git repository in the working directory.

### GitHub Actions install

```yaml
- uses: krakjn/bump@v9
```

Pass a custom token to avoid unauthenticated GitHub API rate limits:

```yaml
- uses: krakjn/bump@v9
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

### Full release workflow

```yaml
name: Release

on:
  push:
    branches: [main]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: krakjn/bump@v9

      - name: Bump and tag
        run: |
          bump minor
          bump update Cargo.toml
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add bump.toml Cargo.toml
          git commit -m "chore(release): $(bump p)"
          bump tag
          git push origin HEAD --tags
```

## Makefile Integration

```makefile
VERSION := $(shell bump print --only base)
VERSION_FULL := $(shell bump print --full)

.PHONY: version emit
version:
	@echo $(VERSION)

emit:
	bump emit c --prefix MYAPP_ -o include/version.h
```

## Docker Build Args

```dockerfile
ARG VERSION
RUN echo "Building ${VERSION}"
COPY . .

# In your CI build step:
# docker build --build-arg VERSION="$(bump print --without prefix)" .
```

## See Also

- [README](../README.md) — bumpfile schema, command reference, and tips
- [Breaking Changes](BREAKING_CHANGES.md) — migration notes between major versions
