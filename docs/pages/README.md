# Automatic Versioning

Everyone versions differently, and that's okay. bump is dead simple and **without opinion**: a little convention, a lot of automation, and you never have to think about versions again.

## Features

- **Declarative**: human-readable, editable `bump.toml`
- **Composable**: build *your* version, e.g. `PRODUCT.RELEASE.HOTFIX`, or mix calendar and custom keys
- **Flexible**: several **BUMPFILE**s in one repo
- **Integrated**: `bump print` and `bump emit` feed source, build scripts, and other tools
- **Compatible**: `bump print --semver` for tools that want the first three base components
- **Automatic**: built for continuous integration

## Install

Linux, macOS, or WSL:

```bash
curl -fsSL https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.sh | sh
```

Windows:

```powershell
irm https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.ps1 | iex
```

## Download latest

One file per platform, always from the current GitHub Release.

<div class="fittings">
<a class="fitting" data-platform="linux-amd64" href="https://github.com/krakjn/bump/releases/latest/download/bump-linux-amd64" download="bump"><span class="fitting-os">Linux</span><span class="fitting-arch">amd64</span><code>bump-linux-amd64</code></a>
<a class="fitting" data-platform="linux-arm64" href="https://github.com/krakjn/bump/releases/latest/download/bump-linux-arm64" download="bump"><span class="fitting-os">Linux</span><span class="fitting-arch">arm64</span><code>bump-linux-arm64</code></a>
<a class="fitting" data-platform="macos-arm64" href="https://github.com/krakjn/bump/releases/latest/download/bump-macos-arm64" download="bump"><span class="fitting-os">macOS</span><span class="fitting-arch">Apple silicon</span><code>bump-macos-arm64</code></a>
<a class="fitting" data-platform="macos-amd64" href="https://github.com/krakjn/bump/releases/latest/download/bump-macos-amd64" download="bump"><span class="fitting-os">macOS</span><span class="fitting-arch">Intel</span><code>bump-macos-amd64</code></a>
<a class="fitting" data-platform="windows-amd64" href="https://github.com/krakjn/bump/releases/latest/download/bump-windows-amd64.exe" download="bump.exe"><span class="fitting-os">Windows</span><span class="fitting-arch">amd64</span><code>bump-windows-amd64.exe</code></a>
<a class="fitting" data-platform="windows-arm64" href="https://github.com/krakjn/bump/releases/latest/download/bump-windows-arm64.exe" download="bump.exe"><span class="fitting-os">Windows</span><span class="fitting-arch">arm64</span><code>bump-windows-arm64.exe</code></a>
</div>

On Linux and macOS, mark it executable and put it on your `PATH`: `chmod +x bump && mv bump ~/.local/bin/`.

## Quick start

```bash
bump init          # creates bump.toml with SemVer defaults
bump print         # v0.1.0
bump patch         # 0.1.0 -> 0.1.1
bump print --full  # prefix + base + phase + suffix + timestamp (needs git for suffix)
```

Pass a path to use a non-default **BUMPFILE** on any command: `bump patch lib/bump.toml`.

Next: [Usage](usage.md) for the full command reference, or the [Workflow guide](workflow.md) for release pipelines.
