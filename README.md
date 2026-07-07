```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```
---

`bump` -> un-opinionated, dead-simple, automatic, versioning.

## TL;DR
- Human readable `bump.toml`, which can be modified
- _No **regex**_, control your versions declaratively
- A version tracked `bump.toml` can flex to the users needs.
- DONE, stop thinking about versioning!


## Why?
I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay — with a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!


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

### Initialize a Project

```bash
bump init
```

This creates a **BUMPFILE** (default `bump.toml`) in your current directory with sensible defaults. You can rename it to whatever you like.

To use CalVer, set `mode = "calver"` under `[base]` in your bumpfile.

## Commands

### Reporting Commands

> Often the hard part is getting the version _into other tools_.
> All `print` variants write output **without a trailing newline**.

```bash
# Default print ([prefix][base][phase])
bump print [BUMPFILE]
bump p [BUMPFILE]              # alias

# Print variants
bump print --only-prefix [BUMPFILE]
bump print --only-phase [BUMPFILE]
bump print --only-base [BUMPFILE]
bump print --no-prefix [BUMPFILE]
bump print --no-phase [BUMPFILE]
bump print --with-suffix [BUMPFILE]
bump print --with-timestamp [BUMPFILE]
bump print --with-label DEV [BUMPFILE]
bump print --full [BUMPFILE]

# Stackable (e.g. omit prefix and include suffix)
bump print --no-prefix --with-suffix [BUMPFILE]
```

Suffix output (`--with-suffix`, `--full`) requires a git repository.

### SemVer Commands

```bash
# Bump version numbers (updates BUMPFILE)
bump --major     # 1.0.0 -> 2.0.0, clears phase
bump --minor     # 1.0.0 -> 1.1.0, clears phase
bump --patch     # 1.0.0 -> 1.0.1, clears phase

# Phase workflow
bump --phase alpha  # 1.1.0 -> 1.1.0-alpha.1
bump --phase        # increment phase distance, e.g. 1.1.0-alpha.2
bump --phase beta   # switch phase, e.g. 1.1.0-beta.1
```

### CalVer Commands

```bash
# Set [base].mode = "calver" in BUMPFILE, then:
bump --calendar [BUMPFILE]  # Updates to current date (e.g., 2026.02.25)
# Same-day bumps automatically increment phase distance
```

### Bumpfile Meta Flags

Update bumpfile fields without a formal version bump:

```bash
bump --prefix v2-
bump --suffix branch
```

### Mode/key compatibility behavior

- If `mode = "semver"` and keys like `year/month/day` are found, bump prints a warning and rewrites keys as `major/minor/patch` on save.
- If `mode = "calver"` and keys like `major/minor/patch` are found, bump rewrites keys as `year/month/day` on save.

### Code Generation

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues

```bash
# Generate version files for different languages
bump gen --lang c --output version.h [BUMPFILE]
bump gen --lang go --output version.go [BUMPFILE]
bump gen --lang java --output Version.java [BUMPFILE]
bump gen --lang csharp --output Version.cs [BUMPFILE]
bump gen --lang python --output version.py [BUMPFILE]

# Generate multiple files at once
bump gen --lang c --output version.h --output include/version.h [BUMPFILE]

# Use custom bumpfile
bump gen --lang c --output version.h custom.toml

# SemVer generates: VERSION, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, etc.
# CalVer generates: VERSION_STRING only (simplified for date-based versions)
```

### Git Integration

```bash
# Create a git annotated tag (git tag -a) for the current version (conventional commit message by default)
bump tag [BUMPFILE]

# Create a tag with custom message
bump tag -m "Custom message" [BUMPFILE]
```

### `bump update`

> Currently supports `Cargo.toml` and `pyproject.toml` — send a PR for additional file format conventions!

```bash
bump update Cargo.toml [BUMPFILE]
bump update pyproject.toml [BUMPFILE]
```


## GitHub Actions

The composite action `action.yml` at the repo root installs bump for the job's OS/arch:

```yaml
- uses: krakjn/bump@v7
```

If your token differs from the default `GITHUB_TOKEN`:

```yaml
- uses: krakjn/bump@v7
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

## Tips and Tricks

you can inject bump _everywhere_
```bash
sed -i "s|REPLACE_ME|$(bump print --no-prefix)|g" somefile
```

```cmake
# CMakeLists.txt
execute_process(
  COMMAND bump print --only-base
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/
  OUTPUT_VARIABLE VERSION)
project("your-app" VERSION ${VERSION} LANGUAGES CXX C)
```

### Shell Completion

`bump completion SHELL` prints a completion script for the given shell. Regenerate after upgrading `bump` so completions stay in sync with new flags and subcommands.

Supported shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

**Bash:**

```bash
bump completion bash >> ~/.bash_completion.d/bump
# or load once in the current session:
source <(bump completion bash)
```

**Zsh:**

```zsh
mkdir -p ~/.zsh/completions
bump completion zsh > ~/.zsh/completions/_bump
# add to ~/.zshrc if needed: fpath=(~/.zsh/completions $fpath); autoload -Uz compinit && compinit
```

**Fish:**

```fish
bump completion fish > ~/.config/fish/completions/bump.fish
```

**PowerShell:**

```powershell
bump completion powershell | Out-String | Invoke-Expression
# or append to your profile:
Add-Content $PROFILE 'bump completion powershell | Out-String | Invoke-Expression'
```

- **[Configuration Reference](docs/CONFIGURATION.md)** — bumpfile schema, print flags, and mode behavior
- **[Workflow Guide](docs/WORKFLOW.md)** — release pipelines, phases, labels, and CI examples
- **[Contributing Guide](docs/CONTRIBUTING.md)** — build from source, run integration tests, and project layout

## [MIT License](./LICENSE)
