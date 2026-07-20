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

## Breaking changes (v7 → v8)

| v7 | v8 |
|----|----|
| `bump` with no args → error | `bump` → **show** composed version |
| `bump print …` / `bump p …` | `bump …` / `bump show …` / `bump p …` |
| `bump --major` / `--minor` / `--patch` | `bump major` / `minor` / `patch` |
| `bump --phase` / `bump --phase NAME` | `bump phase` / `bump phase NAME` |
| `bump --calendar` | `bump calendar` |
| `bump --prefix X` / `bump --suffix MODE` | `bump meta --prefix X` / `bump meta --suffix MODE` |
| `bump PATH --patch` | `bump patch PATH` |
| `bump gen -l LANG -o FILE` | `bump emit LANG -o FILE` |

`init`, `tag`, `update`, `completion`, bumpfile schema, and version assembly rules are unchanged in role.

## Commands

### Show (default)

> All show variants write output **without a trailing newline**.

```bash
# Default show ([prefix][base][phase])
bump [BUMPFILE]
bump show [BUMPFILE]
bump p [BUMPFILE]              # alias

# Show variants
bump --only-prefix [BUMPFILE]
bump --only-phase [BUMPFILE]
bump --only-base [BUMPFILE]
bump --no-prefix [BUMPFILE]
bump --no-phase [BUMPFILE]
bump --with-suffix [BUMPFILE]
bump --with-timestamp [BUMPFILE]
bump --with-label DEV [BUMPFILE]
bump --full [BUMPFILE]

# Stackable (e.g. omit prefix and include suffix)
bump --no-prefix --with-suffix [BUMPFILE]
```

Suffix output (`--with-suffix`, `--full`) requires a git repository.

### SemVer Commands

```bash
# Bump version numbers (updates BUMPFILE)
bump major     # 1.0.0 -> 2.0.0, clears phase
bump minor     # 1.0.0 -> 1.1.0, clears phase
bump patch     # 1.0.0 -> 1.0.1, clears phase

# Phase workflow
bump phase alpha  # 1.1.0 -> 1.1.0-alpha.1
bump phase        # increment phase distance, e.g. 1.1.0-alpha.2
bump phase beta   # switch phase, e.g. 1.1.0-beta.1
```

### CalVer Commands

```bash
# Set [base].mode = "calver" in BUMPFILE, then:
bump calendar [BUMPFILE]  # Updates to current date (e.g., 2026.02.25)
# Same-day bumps automatically increment phase distance
```

### Metadata

Update bumpfile metadata fields (extension point for future setters):

```bash
bump meta --prefix v2-
bump meta --suffix branch
bump meta --prefix v- --suffix git_sha
```

### Mode/key compatibility behavior

- If `mode = "semver"` and keys like `year/month/day` are found, bump prints a warning and rewrites keys as `major/minor/patch` on save.
- If `mode = "calver"` and keys like `major/minor/patch` are found, bump rewrites keys as `year/month/day` on save.

### Emit

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues

```bash
# Language templates → files
bump emit c -o version.h [BUMPFILE]
bump emit go -o version.go [BUMPFILE]
bump emit java -o Version.java [BUMPFILE]
bump emit csharp -o Version.cs [BUMPFILE]
bump emit python -o version.py [BUMPFILE]

# Multiple files
bump emit c -o version.h -o include/version.h [BUMPFILE]

# Structured / raw → stdout (or -o)
bump emit raw
bump emit json
bump emit toml -o version.toml
bump emit yaml --case camel

# Prefix and case identifier names (not bumpfile version prefix)
bump emit c --prefix "MYLIB_" --case uppercase -o version.h
# → #define MYLIB_VERSION_STRING "…"
bump emit raw --prefix "app_" --case camel
# → app_versionString="…"
```

Formats: `raw`, `c`, `java`, `csharp`, `go`, `python`, `json`, `toml`, `yaml`.
`--case` applies to structured keys only.

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
- uses: krakjn/bump@v8
```

If your token differs from the default `GITHUB_TOKEN`:

```yaml
- uses: krakjn/bump@v8
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

## Tips and Tricks

you can inject bump _everywhere_
```bash
sed -i "s|REPLACE_ME|$(bump --no-prefix)|g" somefile
```

```cmake
# CMakeLists.txt
execute_process(
  COMMAND bump --only-base
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

- **[Configuration Reference](docs/CONFIGURATION.md)** — bumpfile schema, show flags, and mode behavior
- **[Workflow Guide](docs/WORKFLOW.md)** — release pipelines, phases, labels, and CI examples
- **[Contributing Guide](docs/CONTRIBUTING.md)** — build from source, run integration tests, and project layout

## [MIT License](./LICENSE)
