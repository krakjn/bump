# Automatic Versioning

<p class="tagline">Move up with bump.</p>

<img class="hero-mark" src="assets/bump_final-white.svg#only-dark" alt="bump" />
<img class="hero-mark" src="assets/bump_final.svg#only-light" alt="" />

The sweet spot is when **b**anal **u**pdates **m**ove **p**rogrammatically. Stop messing around with versioning and automate it away.

Everyone versions differently, and that's okay. bump is dead simple and **without opinion** — a little convention, a lot of automation, and you never have to think about versions again.

## Features

- **Declarative** — human-readable, editable `bump.toml`
- **Composable** — build *your* version, e.g. `PRODUCT.RELEASE.HOTFIX`, or mix calendar and custom keys
- **Flexible** — several **BUMPFILE**s in one repo
- **Integrated** — `bump print` and `bump emit` feed source, build scripts, and other tools
- **Compatible** — `bump print --semver` for tools that want the first three base components
- **Automatic** — built for continuous integration

## Install

**Linux, macOS, or WSL:**

```bash
curl -fsSL https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.ps1 | iex
```

## Download latest

One file per platform — always the current GitHub Release.

<div class="download-grid" markdown>

[Linux amd64](https://github.com/krakjn/bump/releases/latest/download/bump-linux-amd64){: download="bump" }
[Linux arm64](https://github.com/krakjn/bump/releases/latest/download/bump-linux-arm64){: download="bump" }
[macOS Apple silicon](https://github.com/krakjn/bump/releases/latest/download/bump-macos-arm64){: download="bump" }
[macOS Intel](https://github.com/krakjn/bump/releases/latest/download/bump-macos-amd64){: download="bump" }
[Windows amd64](https://github.com/krakjn/bump/releases/latest/download/bump-windows-amd64.exe){: download="bump.exe" }
[Windows arm64](https://github.com/krakjn/bump/releases/latest/download/bump-windows-arm64.exe){: download="bump.exe" }

</div>

## Quick start

```bash
bump init          # creates bump.toml with SemVer defaults
bump print         # v0.1.0
bump patch         # 0.1.0 -> 0.1.1
bump print --full  # prefix + base + phase + suffix + timestamp (needs git for suffix)
```

Pass a path to use a non-default **BUMPFILE** on any command: `bump patch lib/bump.toml`.

Next: [Usage](usage.md) for the full command reference, or the [Workflow guide](WORKFLOW.md) for release pipelines.
