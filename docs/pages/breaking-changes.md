# Breaking Changes

## v8 → v9

### Print flags are composable

v8 used one-off boolean flags. v9 uses `--with`, `--without`, and `--only` with component names. Flags stack freely (except `--only`, which selects a single component).

| v8 | v9 |
|----|----|
| `--only-prefix` | `--only prefix` |
| `--only-base` | `--only base` |
| `--only-phase` | `--only phase` |
| `--no-prefix` | `--without prefix` |
| `--no-phase` | `--without phase` |
| `--with-suffix` | `--with suffix` |
| `--with-timestamp` | `--with timestamp` |
| `--with-label DEV` | `--label DEV` |
| `--full` | `--full` (unchanged) |
| _(none)_ | `--semver` — first three base keys only |

```bash
# v8
bump print --no-prefix --with-suffix

# v9
bump print --without prefix --with suffix
```

### Base bump subcommands come from your bumpfile

v8 exposed fixed subcommands: `major`, `minor`, `patch`, and `calendar`.

v9 generates one subcommand per `[base]` key in TOML order. Run `bump --help` in a repo to see yours.

| v8 | v9 |
|----|----|
| `bump calendar` | `bump date` (shown when `[base]` has `year`, `month`, or `day`) |
| `bump major` / `minor` / `patch` | same, when those keys exist in `[base]` |
| _(none)_ | `bump alpha`, `bump beta`, … for custom keys |

v8 `calendar` incremented phase on a same-day repeat. v9 does that **only** for `bump date`. `bump patch` / `bump alpha` always clear phase, even if they refresh date keys as a side effect.

### Bumpfile schema: no `[base].mode`

v8 used `mode = "semver"` or `mode = "calver"` under `[base]` and could rewrite mismatched keys on save.

v9 has no version mode. Define the keys you want directly:

```toml
# SemVer
[base]
delimiter = "."
major = 0
minor = 1
patch = 0

# CalVer — replace major/minor/patch with date keys
[base]
delimiter = "."
year = 2026
month = 2
day = 25
```

Remove `[base].mode` when upgrading. The key is reserved and ignored if left in place.

v8 also allowed optional `minor` / `patch` keys. v9 treats every non-reserved `[base]` key as a required integer component.

### Base bumps clear phase

Any non-`date` base bump clears `[phase]` (name and distance reset). `bump date` on a new calendar day also clears phase. `bump date` when date keys are already today's UTC increments phase instead.

### Removed commands

| v8 | v9 |
|----|----|
| `bump completion SHELL` | removed |
| `bump calendar` | `bump date` |

### No-subcommand behavior

v8 printed help when invoked without a subcommand. v9 prints an error and suggests `bump --help`.

---

## v7 → v8

| v7 | v8 |
|----|----|
| `bump --major` / `--minor` / `--patch` | `bump major` / `minor` / `patch` |
| `bump --phase` / `bump --phase NAME` | `bump phase` / `bump phase NAME` |
| `bump --calendar` | `bump date` |
| `bump --prefix X` / `bump --suffix MODE` | `bump meta --prefix X` / `bump meta --suffix MODE` |
| `bump PATH --patch` | `bump patch PATH` |
| `bump gen -l LANG -o FILE` | `bump emit LANG -o FILE` |

`init`, `tag`, `update`, bumpfile schema, and version assembly rules are unchanged in role.
