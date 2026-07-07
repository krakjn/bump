# Configuration Reference

`bump` uses one unified TOML schema in `bump.toml` for both SemVer and CalVer.

## BUMPFILE

The BUMPFILE can be named anything; the default is `bump.toml`. Pass a different path
as the positional `BUMPFILE` argument on any command.

```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)
#
# https://github.com/krakjn/bump

prefix = "v"

# NOTE: some fields are modified by bump
#   - mode: "semver" or "calver"
#   - minor|patch: optional, can be removed if not needed
[base]
mode = "semver"
delimiter = "."
major = 0
minor = 1
patch = 0

[phase]
separator = "-"
name = ""
delimiter = "."
distance = 0

# suffix type:
#  - "git_sha"  : append 7 char sha1 of the current commit (default)
#  - "branch"   : append the current git branch name
[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax, used in file generation
last = "2026-06-05 19:06:16 UTC"

# printed label: shown but never tracked, useful for injecting dynamic values
#  - position: "before-prefix", "after-prefix", "before-base", "after-base",
#              "before-phase", "after-phase"
[label]
position = "after-base"
```

## Key Sections

### `prefix` (top-level)

- Optional leading text printed before the numeric base (for example `v`).
- Omitted from output with `bump print --no-prefix`.
- Can be changed in place with `bump --prefix <PREFIX>` (persists to the bumpfile).

### `[timestamp]`

- `format`: `strftime` format used when writing `timestamp.last`.
- `last`: updated on every bump operation.

### `[base]`

- `mode`: `semver` or `calver`.
- `delimiter`: separator for base components.
- `major`, `minor`, `patch`: numeric components in SemVer mode.
- `minor` and `patch` are optional.

For compatibility, `year`, `month`, and `day` are accepted as aliases for
`major`, `minor`, and `patch` when loading.

### `[phase]`

- `separator`: inserted before phase data (commonly `-`).
- `name`: phase label (for example `rc`, `beta`, or empty).
- `delimiter`: separator between `name` and `distance`.
- `distance`: phase counter.

### `[suffix]`

- `mode`: `git_sha` or `branch`.
- `separator`: separator before the suffix payload (commonly `+`).
- Requires a git repository when used with `bump print --with-suffix` or `bump print --full`.
- Can be changed in place with `bump --suffix git_sha|branch` (persists to the bumpfile).

### `[label]`

- `position`: where `bump print --with-label <LABEL>` injects runtime label text.
- Label value is never written to the bumpfile.
- The label is only printed when its anchored segment is part of the current
  assembly (for example, a `before-phase` label is omitted when `--no-phase`
  is used).

## Mode-Specific Behavior

### SemVer mode

- Supported bump ops: `--major`, `--minor`, `--patch`, `--phase`.
- `--calendar` is rejected.
- `--major`, `--minor`, and `--patch` clear the phase (promotion).
- Base format is `<major><delimiter><minor><delimiter><patch>`.

### CalVer mode

- Supported bump ops: `--calendar`, `--phase`.
- `--major`, `--minor`, and `--patch` are rejected.
- Month and day values are printed with zero padding in base output.

## Key Remapping Rules

When writing back to disk, keys are normalized to match `base.mode`.

- If `mode = "semver"`, stored keys become `major/minor/patch`.
- If `mode = "calver"`, stored keys become `year/month/day`.

Additional safety behavior:

- If `mode = "semver"` but the file contains `year/month/day`, a warning is
  emitted and keys are rewritten on save.

## Print Output

Use the `print` subcommand (alias `p`). Flags are stackable except `--only-*`
and `--full`. All variants emit output **without a trailing newline**.

Default assembly order: `[prefix][base][phase]`, with optional label injection
at `[label].position`. When `--with-timestamp` or `--full` is used, the timestamp
is appended after **two spaces**.

```text
Print [prefix][base][phase] from BUMPFILE without newline

Usage: bump print [OPTIONS] [BUMPFILE]

Arguments:
  [BUMPFILE]  Path to the configuration file [default: bump.toml]

Options:
      --only-prefix         Print [prefix]
      --only-phase          Print [phase]
      --only-base           Print [base]
      --no-prefix           Print [base][phase]
      --no-phase            Print [prefix][base]
      --with-suffix         Print [prefix][base][phase][suffix]
      --with-timestamp      Print [prefix][base][phase][timestamp]
      --full                Print full output; overrides all print flags except --with-label
      --with-label <LABEL>  Inject LABEL at [label].position (not persisted)
  -h, --help                Print help
```

`--full` produces `[prefix][base][phase][suffix]  [timestamp]` (suffix and
timestamp require a git repository for suffix resolution).

## See Also

- [README](../README.md) — command overview and quick start
- [Workflow Guide](WORKFLOW.md) — release and CI examples
- [Contributing Guide](CONTRIBUTING.md) — build from source and run integration tests
