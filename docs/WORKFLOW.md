# Workflow Examples

Practical patterns for day-to-day use. For bumpfile fields and print flags, see the
[Configuration Reference](CONFIGURATION.md).

## Single BUMPFILE Pipeline

Use this for a single `bump.toml` at repository root.

### 1. Bump, update metadata, tag, and push

```bash
bump minor
bump update Cargo.toml

git add bump.toml Cargo.toml
git commit -m "chore(release): bump version to $(bump p)"

bump tag
git push origin HEAD --tags
```

### 2. Emit version files during builds

```bash
# Example: emit C header from current bumpfile state
bump emit c -o version.h

# Then run your normal build
<build tool> ...
```

## SemVer Phase Workflow

Phases are free-form labels with an incrementing distance counter.

```bash
# Start a release candidate phase
bump phase rc         # e.g., 1.4.0 -> 1.4.0-rc.1

# Continue same phase
bump phase            # e.g., 1.4.0-rc.2
bump phase rc         # also increments when phase matches current

# Switch phase name
bump phase beta       # e.g., 1.4.0-beta.1

# Promote: formal bumps clear the phase
bump minor            # e.g., 1.4.0-beta.1 -> 1.5.0
```

## CalVer Workflow

Set `mode = "calver"` in your bumpfile first, then:

```bash
bump calendar
```

If the date is unchanged, calendar bump increments `phase.distance`.

## Ephemeral Labels

Labels are injected at print time only — they are never persisted to the bumpfile.

```bash
# [label].position = "after-base" in bump.toml
bump print --with-label DEV        # e.g., v1.0.0DEV
bump print --full --with-label DEV # label + suffix + timestamp
```

## Multiple BUMPFILE Pipeline

`bump` supports multiple version streams in one repository by passing the file path
as the positional `BUMPFILE` argument.

```bash
bump minor lib1/bump.toml
bump major app/component/bump.toml

git add -u
git commit -m "chore(release): bump component versions"

# tag uses the default bumpfile unless an explicit BUMPFILE is provided
bump tag lib1/bump.toml
bump tag app/component/bump.toml

git push origin HEAD --tags
```

## CI-Friendly Version Output

```bash
bump print --only-base
bump print --full
bump print --with-suffix
bump print --with-label BUILD_ID
bump emit raw --prefix "APP_"
bump emit c --prefix "APP_" --case uppercase -o version.h
```

All print commands emit without a trailing newline, so they are safe for shell
substitution. Suffix output requires the job to run inside a git checkout.

### GitHub Actions

Install `bump` in a workflow with the composite action at the repo root:

```yaml
- uses: krakjn/bump@v8
```

Pass a custom token if you need to avoid unauthenticated GitHub API rate limits:

```yaml
- uses: krakjn/bump@v8
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

## See Also

- [Configuration Reference](CONFIGURATION.md) — bumpfile schema and print flags
- [Contributing Guide](CONTRIBUTING.md) — build, test, and contribute
