use crate::cmd::{BumpError, get_git_branch, get_git_commit_sha, is_git_repository, load_bumpfile};
use crate::version::{LabelPosition, SuffixMode, Version};
use clap::{ArgMatches, ValueEnum};

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum PrintValue {
    Prefix,
    Base,
    Phase,
    Suffix,
    Timestamp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct PrintSelection {
    pub full: bool,
    pub semver: bool,  // ignores --with "base", --without "base", --only *
    pub with_prefix: bool,
    pub with_base: bool,
    pub with_phase: bool,
    pub with_suffix: bool,
    pub with_timestamp: bool,
    pub without_prefix: bool,
    pub without_base: bool,
    pub without_phase: bool,
    pub without_suffix: bool,
    pub without_timestamp: bool,
    pub only_prefix: bool,
    pub only_base: bool,
    pub only_phase: bool,
    pub only_suffix: bool,
    pub only_timestamp: bool,
    pub label: Option<String>,
}

impl PrintSelection {
    pub fn default() -> Self {
        Self {
            full: false,
            semver: false,
            with_prefix: false,
            with_base: false,
            with_phase: false,
            with_suffix: false,
            with_timestamp: false,
            without_prefix: false,
            without_base: false,
            without_phase: false,
            without_suffix: false,
            without_timestamp: false,
            only_prefix: false,
            only_base: false,
            only_phase: false,
            only_suffix: false,
            only_timestamp: false,
            label: None,
        }
    }
}

fn selection_from_matches(matches: &ArgMatches) -> Result<PrintSelection, BumpError> {
    let mut selection = PrintSelection::default();
    for id in matches.ids() {  // user can define whether its "with" or "without" first
        match id.as_str() {
            "full" => selection.full = true,
            "semver" => selection.semver = true,
            "with" => {
                if let Some(args) = matches.get_many::<PrintValue>("with") {
                    for arg in args {
                        match arg {
                            PrintValue::Prefix => selection.with_prefix = true,
                            PrintValue::Base => selection.with_base = true,
                            PrintValue::Phase => selection.with_phase = true,
                            PrintValue::Suffix => selection.with_suffix = true,
                            PrintValue::Timestamp => selection.with_timestamp = true,
                        }
                    }
                }
            }
            "without" => {
                if let Some(args) = matches.get_many::<PrintValue>("without") {
                    for arg in args {
                        match arg {
                            PrintValue::Prefix => selection.without_prefix = true,
                            PrintValue::Base => selection.without_base = true,
                            PrintValue::Phase => selection.without_phase = true,
                            PrintValue::Suffix => selection.without_suffix = true,
                            PrintValue::Timestamp => selection.without_timestamp = true,
                        }
                    }
                }
            }
            "only" => {
                if let Some(args) = matches.get_many::<PrintValue>("only") {
                    for arg in args {
                        match arg {
                            PrintValue::Prefix => selection.only_prefix = true,
                            PrintValue::Base => selection.only_base = true,
                            PrintValue::Phase => selection.only_phase = true,
                            PrintValue::Suffix => selection.only_suffix = true,
                            PrintValue::Timestamp => selection.only_timestamp = true,
                        }
                    }
                }
            }
            "label" => {
                if let Some(label) = matches.get_one::<String>("label") {
                    selection.label = Some(label.to_string());
                }
            }
            _ => return Err(BumpError::LogicError(format!("Unknown print option: {id}"))),
        }
    }
    Ok(selection)
}

pub fn print(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let selection = selection_from_matches(matches)?;
    print!("{}", version.to_string(&selection)?);
    Ok(())
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
struct Components {
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
    // fn default(version: &Version, selection: &PrintSelection) -> Result<Self, BumpError> {
    //     let suffix_value = if is_git_repository() {
    //         suffix(version)?
    //     } else {
    //         String::new()
    //     };
    //     Ok(Self {
    //         prefix: Field {
    //             active: true,
    //             value: version.prefix.clone(),
    //         },
    //         base: Field {
    //             active: true,
    //             value: base(version),
    //         },
    //         phase: Field {
    //             active: true,
    //             value: phase(version),
    //         },
    //         suffix: Field {
    //             active: false,
    //             value: suffix_value,
    //         },
    //         timestamp: Field {
    //             active: false,
    //             value: version.timestamp.last.clone(),
    //         },
    //         label: LabelField {
    //             active: false,
    //             value: selection.label.clone(),
    //             position: version.label.position,
    //         },
    //     })
    // }

    // will return early to supply "only" category
    // if None, then collect will construct the string
    fn apply(
        &mut self,
        version: &Version,
        selection: &PrintSelection,
    ) -> Result<Option<String>, BumpError> {
        if selection.full {
            self.prefix.active = true;
            self.base.active = true;
            self.phase.active = true;
            self.suffix.value = suffix(version)?;
            self.suffix.active = true;
            self.timestamp.active = true;
            return Ok(None);
        }

        // on "only"s it returns with just that
        if selection.only_prefix {
            return Ok(Some(self.prefix.value.clone()));
        }
        if selection.only_base {
            return Ok(Some(self.base.value.clone()));
        }
        if selection.only_phase {
            return Ok(Some(self.phase.value.clone()));
        }
        if selection.only_suffix {
            return Ok(Some(self.suffix.value.clone()));
        }
        if selection.only_timestamp {
            return Ok(Some(self.timestamp.value.clone()));
        }

        if selection.with_prefix {
            self.prefix.active = true;
        }
        if selection.with_base {
            self.base.active = true;
        }
        if selection.with_phase {
            self.phase.active = true;
        }
        if selection.with_suffix {
            self.suffix.value = suffix(version)?;
            self.suffix.active = true;
        }
        if selection.with_timestamp {
            self.timestamp.active = true;
        }

        if selection.without_prefix {
            self.prefix.active = false;
        }
        if selection.without_base {
            self.base.active = false;
        }
        if selection.without_phase {
            self.phase.active = false;
        }
        if selection.without_suffix {
            self.suffix.active = false;
        }
        if selection.without_timestamp {
            self.timestamp.active = false;
        }
        if selection.label.is_some() {
            self.label.value = selection.label.clone();
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
    if let Some(segment) = components.apply_opts(version, opts)? {
        return Ok(segment);
    }
    Ok(components.collect())
}

// fn format_component(version: &Version, n: u32) -> String {
//     if version.base.mode == VersionMode::Calver {
//         format!("{n:02}")
//     } else {
//         n.to_string()
//     }
// }

fn base(version: &Version) -> String {
    let mut output = String::new();
    for (index, (name, value)) in version.base.components.iter().enumerate() {
        match name.as_str() {
            "year" => {
                output.push_str(&format!("{:04}", value));
            }
            "month" => {
                output.push_str(&format!("{:02}", value));
            }
            "day" => {
                output.push_str(&format!("{:02}", value));
            }
            _ => {
                output.push_str(&value.to_string());
            }
        }
        if index != version.base.components.len() - 1 {
            output.push_str(&version.base.delimiter);
        }
    }
    output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{
        Base, Label, LabelPosition, Phase, Suffix, SuffixMode, Timestamp, Version, VersionMode,
    };

    fn test_version() -> Version {
        Version {
            prefix: "v-".to_string(),
            base: Base {
                mode: VersionMode::Semver,
                delimiter: ".".to_string(),
                major: 0,
                minor: Some(1),
                patch: Some(0),
            },
            phase: Phase {
                separator: "-".to_string(),
                name: String::new(),
                delimiter: ".".to_string(),
                distance: 0,
            },
            suffix: Suffix {
                mode: SuffixMode::GitSha,
                separator: "+".to_string(),
            },
            timestamp: Timestamp {
                format: "%Y-%m-%d %H:%M:%S %Z".to_string(),
                last: "2026-01-01 00:00:00 UTC".to_string(),
            },
            label: Label {
                position: LabelPosition::AfterBase,
            },
        }
    }

    #[test]
    fn only_base_returns_base_without_newline() {
        let v = test_version();
        let out = to_string(
            &v,
            &PrintOptions {
                only_base: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out, "0.1.0");
    }

    #[test]
    fn conflicting_only_flags_error() {
        let only = [true, true, false].into_iter().filter(|&b| b).count();
        assert!(only > 1);
        let err = BumpError::ParseError("Only one type of --only* allowed".to_string());
        assert!(err.to_string().contains("Only one type of --only*"));
    }
}
