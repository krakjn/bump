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
    selection.semver = matches.get_flag("semver");
    if matches.get_flag("full") {
        selection.full = true;
        return selection;
    }
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

    fn default(version: &Version, selection: &PrintSelection) -> Result<Self, BumpError> {
        Ok(Self {
            prefix: Field {
                active: true,
                value: version.prefix.clone(),
            },
            base: Field {
                active: true,
                value: base(version, selection.semver),
            },
            phase: Field {
                active: true,
                value: phase(version),
            },
            suffix: Field {
                active: false,
                value: String::new(),
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
            if only == PrintValue::Suffix {
                return Ok(Some(suffix(version)?));
            }
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
    let mut components = Components::default(version, selection)?;
    if let Some(segment) = components.apply(version, selection)? {
        return Ok(segment);
    }
    Ok(components.collect())
}

fn base(version: &Version, semver: bool) -> String {
    let components = if semver {
        let n = version.base.components.len().min(3);
        &version.base.components[..n]
    } else {
        &version.base.components[..]
    };
    let mut output = String::new();
    for (index, (name, value)) in components.iter().enumerate() {
        match name.as_str() {
            "year" => output.push_str(&format!("{value:04}")),
            "month" => output.push_str(&format!("{value:02}")),
            "day" => output.push_str(&format!("{value:02}")),
            _ => output.push_str(&value.to_string()),
        }
        if index != components.len() - 1 {
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
    use std::env;
    use std::path::Path;
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn print_eq(version: &Version, selection: PrintSelection, expected: &str) {
        assert_eq!(to_string(version, &selection).unwrap(), expected);
    }

    fn with_temp_cwd<T>(f: impl FnOnce() -> T) -> T {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let orig = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        struct Restore(std::path::PathBuf);
        impl Drop for Restore {
            fn drop(&mut self) {
                let _ = env::set_current_dir(&self.0);
            }
        }
        let _restore = Restore(orig);
        f()
    }

    fn mixed_version() -> Version {
        let mut version = Version::test_fixture();
        version.prefix = "v".to_string();
        version.base.components = vec![
            ("year".to_string(), 2026),
            ("alpha".to_string(), 3),
            ("month".to_string(), 4),
            ("beta".to_string(), 1),
        ];
        version
    }

    #[test]
    fn default_compose() {
        print_eq(&Version::test_fixture(), PrintSelection::default(), "v-0.1.0");
    }

    #[test]
    fn mixed_compose_pads_only_date_keys() {
        print_eq(&mixed_version(), PrintSelection::default(), "v2026.3.04.1");
    }

    #[test]
    fn semver_prints_first_three_base_components() {
        print_eq(
            &mixed_version(),
            PrintSelection {
                semver: true,
                ..PrintSelection::default()
            },
            "v2026.3.04",
        );
    }

    #[test]
    fn semver_only_base_drops_later_keys() {
        print_eq(
            &mixed_version(),
            PrintSelection {
                semver: true,
                only: Some(PrintValue::Base),
                ..PrintSelection::default()
            },
            "2026.3.04",
        );
    }

    #[test]
    fn semver_keeps_prefix_and_phase() {
        let mut version = mixed_version();
        version.phase.name = "beta".to_string();
        version.phase.distance = 1;
        print_eq(
            &version,
            PrintSelection {
                semver: true,
                ..PrintSelection::default()
            },
            "v2026.3.04-beta.1",
        );
    }

    #[test]
    fn semver_with_fewer_than_three_keys_prints_all() {
        let mut version = Version::test_fixture();
        version.base.components = vec![("alpha".to_string(), 2), ("beta".to_string(), 9)];
        print_eq(
            &version,
            PrintSelection {
                semver: true,
                ..PrintSelection::default()
            },
            "v-2.9",
        );
    }

    #[test]
    fn only_base_returns_base_without_newline() {
        print_eq(
            &Version::test_fixture(),
            PrintSelection {
                only: Some(PrintValue::Base),
                ..PrintSelection::default()
            },
            "0.1.0",
        );
    }

    #[test]
    fn only_prefix() {
        print_eq(
            &Version::test_fixture(),
            PrintSelection {
                only: Some(PrintValue::Prefix),
                ..PrintSelection::default()
            },
            "v-",
        );
    }

    #[test]
    fn without_prefix() {
        print_eq(
            &Version::test_fixture(),
            PrintSelection {
                without: HashSet::from([PrintValue::Prefix]),
                ..PrintSelection::default()
            },
            "0.1.0",
        );
    }

    #[test]
    fn with_timestamp_does_not_need_git() {
        with_temp_cwd(|| {
            assert!(!Path::new(".git").exists());
            print_eq(
                &Version::test_fixture(),
                PrintSelection {
                    with: HashSet::from([PrintValue::Timestamp]),
                    ..PrintSelection::default()
                },
                "v-0.1.0  2026-01-01 00:00:00 UTC",
            );
        });
    }

    #[test]
    fn with_suffix_errors_outside_git() {
        with_temp_cwd(|| {
            let err = to_string(
                &Version::test_fixture(),
                &PrintSelection {
                    with: HashSet::from([PrintValue::Suffix]),
                    ..PrintSelection::default()
                },
            )
            .unwrap_err();
            assert!(err.to_string().contains("Not a git repository"));
        });
    }

    #[test]
    fn full_errors_outside_git() {
        with_temp_cwd(|| {
            let err = to_string(
                &Version::test_fixture(),
                &PrintSelection {
                    full: true,
                    ..PrintSelection::default()
                },
            )
            .unwrap_err();
            assert!(err.to_string().contains("Not a git repository"));
        });
    }

    #[test]
    fn label_at_each_position() {
        let mut version = Version::test_fixture();
        version.phase.name = "beta".to_string();
        version.phase.distance = 1;
        let cases = [
            (LabelPosition::BeforePrefix, "-tigerv-0.1.0-beta.1"),
            (LabelPosition::AfterPrefix, "v--tiger0.1.0-beta.1"),
            (LabelPosition::BeforeBase, "v--tiger0.1.0-beta.1"),
            (LabelPosition::AfterBase, "v-0.1.0-tiger-beta.1"),
            (LabelPosition::BeforePhase, "v-0.1.0-tiger-beta.1"),
            (LabelPosition::AfterPhase, "v-0.1.0-beta.1-tiger"),
        ];
        for (position, expected) in cases {
            version.label.position = position;
            print_eq(
                &version,
                PrintSelection {
                    label: Some("-tiger".to_string()),
                    ..PrintSelection::default()
                },
                expected,
            );
        }
    }
}
