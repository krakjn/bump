use crate::bump::{BumpError, get_git_branch, get_git_commit_sha, is_git_repository};
use crate::version::{LabelPosition, SuffixMode, Version, VersionMode};
use clap::ArgMatches;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrintOptions {
    pub only_prefix: bool,
    pub only_phase: bool,
    pub only_base: bool,
    pub no_prefix: bool,
    pub no_phase: bool,
    pub with_suffix: bool,
    pub with_timestamp: bool,
    pub with_label: Option<String>,
    pub full: bool,
}

impl PrintOptions {
    pub fn parse(matches: &ArgMatches) -> Result<Self, BumpError> {
        let opts = Self {
            only_prefix: matches.get_flag("only-prefix"),
            only_phase: matches.get_flag("only-phase"),
            only_base: matches.get_flag("only-base"),
            no_prefix: matches.get_flag("no-prefix"),
            no_phase: matches.get_flag("no-phase"),
            with_suffix: matches.get_flag("with-suffix"),
            with_timestamp: matches.get_flag("with-timestamp"),
            with_label: matches.get_one::<String>("with-label").cloned(),
            full: matches.get_flag("full"),
        };

        let only = [opts.only_prefix, opts.only_phase, opts.only_base]
            .into_iter()
            .filter(|&b| b)
            .count();
        if only > 1 {
            return Err(BumpError::ParseError(
                "Only one type of --only* allowed".to_string(),
            ));
        }
        Ok(opts)
    }

    pub fn no_prefix() -> Self {
        Self {
            no_prefix: true,
            ..Self::default()
        }
    }

    pub fn with_timestamp() -> Self {
        Self {
            with_timestamp: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    active: bool,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelField {
    active: bool,
    value: Option<String>,
    position: LabelPosition,
}

impl LabelField {
    fn visible_at(&self, position: LabelPosition, components: &Components) -> bool {
        if !self.active || self.position != position {
            return false;
        }
        match position {
            LabelPosition::BeforePrefix | LabelPosition::AfterPrefix => components.prefix.active,
            LabelPosition::BeforeBase | LabelPosition::AfterBase => components.base.active,
            LabelPosition::BeforePhase | LabelPosition::AfterPhase => components.phase.active,
        }
    }

    fn push_slot(&self, output: &mut String, slot: &[LabelPosition], components: &Components) {
        for &position in slot {
            if self.visible_at(position, components) {
                output.push_str(self.value.as_deref().unwrap_or(""));
                return;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Components {
    prefix: Field,
    base: Field,
    phase: Field,
    suffix: Field,
    timestamp: Field,
    label: LabelField,
}

fn push_if_active(out: &mut String, field: &Field) {
    if field.active {
        out.push_str(&field.value);
    }
}

impl Components {
    pub fn from(version: &Version, opts: &PrintOptions) -> Result<Self, BumpError> {
        let suffix_value = if is_git_repository() {
            suffix(version)?
        } else {
            String::new()
        };
        Ok(Self {
            prefix: Field {
                active: true,
                value: version.prefix.clone(),
            },
            base: Field {
                active: true,
                value: base(version),
            },
            phase: Field {
                active: true,
                value: phase(version),
            },
            suffix: Field {
                active: false,
                value: suffix_value,
            },
            timestamp: Field {
                active: false,
                value: version.timestamp.last.clone(),
            },
            label: LabelField {
                active: false,
                value: opts.with_label.clone(),
                position: version.label.position,
            },
        })
    }

    fn apply_opts(
        &mut self,
        version: &Version,
        opts: &PrintOptions,
    ) -> Result<Option<String>, BumpError> {
        if opts.full {
            self.prefix.active = true;
            self.base.active = true;
            self.phase.active = true;
            self.suffix.value = suffix(version)?;
            self.suffix.active = true;
            self.timestamp.active = true;
        } else {
            if opts.only_prefix {
                return Ok(Some(self.prefix.value.clone()));
            }
            if opts.only_base {
                return Ok(Some(self.base.value.clone()));
            }
            if opts.only_phase {
                return Ok(Some(self.phase.value.clone()));
            }

            if opts.no_prefix {
                self.prefix.active = false;
            }
            if opts.no_phase {
                self.phase.active = false;
            }
            if opts.with_suffix {
                self.suffix.value = suffix(version)?;
                self.suffix.active = true;
            }
            if opts.with_timestamp {
                self.timestamp.active = true;
            }
        }

        if opts.with_label.is_some() {
            self.label.active = true;
        }

        Ok(None)
    }

    fn collect(&self) -> String {
        let mut output = String::new();
        self.label
            .push_slot(&mut output, &[LabelPosition::BeforePrefix], self);
        push_if_active(&mut output, &self.prefix);
        self.label.push_slot(
            &mut output,
            &[LabelPosition::AfterPrefix, LabelPosition::BeforeBase],
            self,
        );
        push_if_active(&mut output, &self.base);
        self.label.push_slot(
            &mut output,
            &[LabelPosition::AfterBase, LabelPosition::BeforePhase],
            self,
        );
        push_if_active(&mut output, &self.phase);
        self.label
            .push_slot(&mut output, &[LabelPosition::AfterPhase], self);
        push_if_active(&mut output, &self.suffix);
        if self.timestamp.active {
            output.push_str("  ");
            output.push_str(&self.timestamp.value);
        }
        output
    }
}

pub fn to_string(version: &Version, opts: &PrintOptions) -> Result<String, BumpError> {
    let mut components = Components::from(version, opts)?;
    assemble(version, opts, &mut components)
}

pub fn assemble(
    version: &Version,
    opts: &PrintOptions,
    components: &mut Components,
) -> Result<String, BumpError> {
    if let Some(segment) = components.apply_opts(version, opts)? {
        return Ok(segment);
    }
    Ok(components.collect())
}

fn format_component(version: &Version, n: u32) -> String {
    if version.base.mode == VersionMode::Calver {
        format!("{n:02}")
    } else {
        n.to_string()
    }
}

fn base(version: &Version) -> String {
    match (version.base.minor, version.base.patch) {
        (Some(minor), Some(patch)) => format!(
            "{}{}{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, minor),
            version.base.delimiter,
            format_component(version, patch),
        ),
        (Some(minor), None) => format!(
            "{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, minor),
        ),
        (None, Some(patch)) => format!(
            "{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, patch),
        ),
        _ => format!("{}", version.base.major),
    }
}

fn phase(version: &Version) -> String {
    if version.phase.name.is_empty() && version.phase.distance == 0 {
        String::new()
    } else if version.phase.name.is_empty() && version.phase.distance > 0 {
        format!("{}{}", version.phase.separator, version.phase.distance)
    } else if version.phase.distance == 0 {
        format!("{}{}", version.phase.separator, version.phase.name,)
    } else {
        format!(
            "{}{}{}{}",
            version.phase.separator,
            version.phase.name,
            version.phase.delimiter,
            version.phase.distance
        )
    }
}

fn suffix(version: &Version) -> Result<String, BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }
    match version.suffix.mode {
        SuffixMode::GitSha => {
            let sha = get_git_commit_sha()?;
            Ok(format!("{}{}", version.suffix.separator, sha))
        }
        SuffixMode::Branch => {
            let branch = get_git_branch()?;
            Ok(format!("{}{}", version.suffix.separator, branch))
        }
    }
}
