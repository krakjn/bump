use crate::cmd::{BumpError, get_git_branch, get_git_commit_sha, is_git_repository, load_bumpfile};
use crate::version::{LabelPosition, SuffixMode, Version};
use clap::{ArgMatches, ValueEnum};
use std::collections::HashSet;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub semver: bool,
    pub with: HashSet<PrintValue>,
    pub without: HashSet<PrintValue>,
    pub only: Option<PrintValue>,
    pub label: Option<String>,
}

impl PrintSelection {
    pub fn without_prefix(mut self) -> Self {
        self.without.insert(PrintValue::Prefix);
        self
    }

    pub fn with_timestamp(mut self) -> Self {
        self.with.insert(PrintValue::Timestamp);
        self
    }
}

fn selection_from_matches(matches: &ArgMatches) -> PrintSelection {
    let mut selection = PrintSelection::default();
    if matches.get_flag("full") {
        selection.full = true;
        return selection;
    }
    selection.semver = matches.get_flag("semver");
    if let Some(args) = matches.get_many::<PrintValue>("with") {
        selection.with.extend(args.copied());
    }
    if let Some(args) = matches.get_many::<PrintValue>("without") {
        selection.without.extend(args.copied());
    }
    selection.only = matches.get_one::<PrintValue>("only").copied();
    selection.label = matches.get_one::<String>("label").cloned();
    selection
}

pub fn print(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let selection = selection_from_matches(matches);
    print!("{}", to_string(&version, &selection)?);
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
    fn field(&self, value: PrintValue) -> &Field {
        match value {
            PrintValue::Prefix => &self.prefix,
            PrintValue::Base => &self.base,
            PrintValue::Phase => &self.phase,
            PrintValue::Suffix => &self.suffix,
            PrintValue::Timestamp => &self.timestamp,
        }
    }

    fn field_mut(&mut self, value: PrintValue) -> &mut Field {
        match value {
            PrintValue::Prefix => &mut self.prefix,
            PrintValue::Base => &mut self.base,
            PrintValue::Phase => &mut self.phase,
            PrintValue::Suffix => &mut self.suffix,
            PrintValue::Timestamp => &mut self.timestamp,
        }
    }

    fn default(version: &Version) -> Result<Self, BumpError> {
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
                value: None,
                position: version.label.position,
            },
        })
    }

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

        if let Some(only) = selection.only {
            return Ok(Some(self.field(only).value.clone()));
        }

        for &value in &selection.with {
            if value == PrintValue::Suffix {
                self.suffix.value = suffix(version)?;
            }
            self.field_mut(value).active = true;
        }
        for &value in &selection.without {
            self.field_mut(value).active = false;
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

pub fn to_string(version: &Version, selection: &PrintSelection) -> Result<String, BumpError> {
    let mut components = Components::default(version)?;
    if let Some(segment) = components.apply(version, selection)? {
        return Ok(segment);
    }
    Ok(components.collect())
}

fn base(version: &Version) -> String {
    let mut output = String::new();
    for (index, (name, value)) in version.base.components.iter().enumerate() {
        match name.as_str() {
            "year" => output.push_str(&format!("{value:04}")),
            "month" => output.push_str(&format!("{value:02}")),
            "day" => output.push_str(&format!("{value:02}")),
            _ => output.push_str(&value.to_string()),
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
        format!("{}{}", version.phase.separator, version.phase.name)
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

    #[test]
    fn only_base_returns_base_without_newline() {
        let out = to_string(
            &Version::test_fixture(),
            &PrintSelection {
                only: Some(PrintValue::Base),
                ..PrintSelection::default()
            },
        )
        .unwrap();
        assert_eq!(out, "0.1.0");
    }
}
