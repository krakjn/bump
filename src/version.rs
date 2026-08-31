use crate::cmd::BumpError;
use chrono::Datelike;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum SuffixMode {
    #[value(name = "git_sha")]
    GitSha,
    Branch,
}

impl SuffixMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitSha => "git_sha",
            Self::Branch => "branch",
        }
    }
}

impl FromStr for SuffixMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "git_sha" => Ok(Self::GitSha),
            "branch" => Ok(Self::Branch),
            _ => Err(format!(
                "Invalid suffix mode '{s}' (expected 'git_sha' or 'branch')"
            )),
        }
    }
}

impl fmt::Display for SuffixMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    BeforePrefix,
    AfterPrefix,
    BeforeBase,
    AfterBase,
    BeforePhase,
    AfterPhase,
}

impl LabelPosition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforePrefix => "before-prefix",
            Self::AfterPrefix => "after-prefix",
            Self::BeforeBase => "before-base",
            Self::AfterBase => "after-base",
            Self::BeforePhase => "before-phase",
            Self::AfterPhase => "after-phase",
        }
    }
}

impl FromStr for LabelPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "before-prefix" => Ok(Self::BeforePrefix),
            "after-prefix" => Ok(Self::AfterPrefix),
            "before-base" => Ok(Self::BeforeBase),
            "after-base" => Ok(Self::AfterBase),
            "before-phase" => Ok(Self::BeforePhase),
            "after-phase" => Ok(Self::AfterPhase),
            _ => Err(format!("Invalid label position '{s}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    pub format: String,
    pub last: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    pub delimiter: String,
    pub components: Vec<(String, u16)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phase {
    pub separator: String,
    pub name: String,
    pub delimiter: String,
    pub distance: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suffix {
    pub mode: SuffixMode,
    pub separator: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub position: LabelPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub prefix: String,
    pub base: Base,
    pub phase: Phase,
    pub suffix: Suffix,
    pub timestamp: Timestamp,
    pub label: Label,
}

impl Version {
    fn update_timestamp(&mut self) {
        let now = chrono::Utc::now();
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
    }

    pub fn phase_bump(&mut self, new_phase: Option<&str>) -> Result<(), BumpError> {
        match new_phase {
            None => {
                self.phase.distance += 1;
            }
            Some(new_phase) => {
                if *new_phase == self.phase.name {
                    self.phase.distance += 1;
                } else {
                    self.phase.name = new_phase.to_string();
                    self.phase.distance = 1;
                }
            }
        }
        self.update_timestamp();
        Ok(())
    }

    pub fn bump(&mut self, component_name: &str) -> Result<(), BumpError> {
        let before = self.base.components.clone();
        let now = chrono::Utc::now();
        let mut after_target = false;
        for (name, value) in self.base.components.iter_mut() {
            if *name == "year" {
                *value = now.year() as u16;
                continue;
            } else if *name == "month" {
                *value = now.month() as u16;
                continue;
            } else if *name == "day" {
                *value = now.day() as u16;
                continue;
            }

            if *name == component_name {
                *value += 1;
                after_target = true;
                continue;
            }

            if after_target {
                *value = 0;
            }
        }
        if self.base.components != before {
            self.clear_phase();
        }
        self.update_timestamp();
        Ok(())
    }

    fn clear_phase(&mut self) {
        self.phase.name.clear();
        self.phase.distance = 0;
    }

    #[cfg(test)]
    pub fn test_fixture() -> Self {
        Self {
            prefix: "v-".to_string(),
            base: Base {
                delimiter: ".".to_string(),
                components: vec![
                    ("major".to_string(), 0),
                    ("minor".to_string(), 1),
                    ("patch".to_string(), 0),
                ],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn get(version: &Version, name: &str) -> u16 {
        version
            .base
            .components
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("missing component {name}"))
            .1
    }

    fn with_components(components: Vec<(&str, u16)>) -> Version {
        let mut version = Version::test_fixture();
        version.base.components = components
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        version
    }

    #[test]
    fn bump_minor_increments_and_zeros_later() {
        let mut v = Version::test_fixture();
        v.bump("minor").unwrap();
        assert_eq!(
            v.base.components,
            vec![
                ("major".to_string(), 0),
                ("minor".to_string(), 2),
                ("patch".to_string(), 0),
            ]
        );
    }

    #[test]
    fn bump_major_zeros_minor_and_patch() {
        let mut v = Version::test_fixture();
        v.bump("major").unwrap();
        assert_eq!(
            v.base.components,
            vec![
                ("major".to_string(), 1),
                ("minor".to_string(), 0),
                ("patch".to_string(), 0),
            ]
        );
    }

    #[test]
    fn bump_custom_keys_cascade_by_order() {
        let mut v = with_components(vec![("alpha", 2), ("beta", 9), ("gamma", 4)]);
        v.bump("alpha").unwrap();
        assert_eq!(get(&v, "alpha"), 3);
        assert_eq!(get(&v, "beta"), 0);
        assert_eq!(get(&v, "gamma"), 0);
    }

    #[test]
    fn mixed_bump_custom_syncs_dates_and_zeros_later_custom() {
        let mut v = with_components(vec![
            ("year", 2020),
            ("alpha", 2),
            ("month", 1),
            ("beta", 9),
            ("day", 1),
        ]);
        v.bump("alpha").unwrap();
        let now = chrono::Utc::now();
        assert_eq!(get(&v, "year"), now.year() as u16);
        assert_eq!(get(&v, "month"), now.month() as u16);
        assert_eq!(get(&v, "day"), now.day() as u16);
        assert_eq!(get(&v, "alpha"), 3);
        assert_eq!(get(&v, "beta"), 0);
    }

    #[test]
    fn bump_date_key_syncs_calendar_and_does_not_cascade() {
        let mut v = with_components(vec![
            ("year", 2020),
            ("alpha", 2),
            ("month", 1),
            ("beta", 9),
        ]);
        v.bump("month").unwrap();
        let now = chrono::Utc::now();
        assert_eq!(get(&v, "year"), now.year() as u16);
        assert_eq!(get(&v, "month"), now.month() as u16);
        assert_eq!(get(&v, "alpha"), 2);
        assert_eq!(get(&v, "beta"), 9);
    }

    #[test]
    fn bump_unknown_name_is_noop_on_custom_keys() {
        let mut v = Version::test_fixture();
        v.bump("nope").unwrap();
        assert_eq!(v.base.components, Version::test_fixture().base.components);
    }

    #[test]
    fn bump_unknown_name_still_syncs_date_keys() {
        let mut v = with_components(vec![("year", 1999), ("alpha", 1)]);
        v.bump("nope").unwrap();
        assert_eq!(get(&v, "year"), chrono::Utc::now().year() as u16);
        assert_eq!(get(&v, "alpha"), 1);
    }

    #[test]
    fn bump_clears_phase_when_base_changes() {
        let mut v = Version::test_fixture();
        v.phase.name = "alpha".to_string();
        v.phase.distance = 2;
        v.bump("patch").unwrap();
        assert_eq!(v.phase.name, "");
        assert_eq!(v.phase.distance, 0);
    }

    #[test]
    fn bump_unknown_name_does_not_clear_phase_without_base_change() {
        let mut v = Version::test_fixture();
        v.phase.name = "alpha".to_string();
        v.phase.distance = 2;
        v.bump("nope").unwrap();
        assert_eq!(v.phase.name, "alpha");
        assert_eq!(v.phase.distance, 2);
    }

    #[test]
    fn phase_bump_empty_increments_distance() {
        let mut v = Version::test_fixture();
        v.phase_bump(None).unwrap();
        assert_eq!(v.phase.distance, 1);
        assert_eq!(v.phase.name, "");
        v.phase_bump(None).unwrap();
        assert_eq!(v.phase.distance, 2);
    }

    #[test]
    fn phase_bump_named_sets_and_resets_distance() {
        let mut v = Version::test_fixture();
        v.phase_bump(Some("beta")).unwrap();
        assert_eq!(v.phase.name, "beta");
        assert_eq!(v.phase.distance, 1);
        v.phase_bump(Some("beta")).unwrap();
        assert_eq!(v.phase.distance, 2);
        v.phase_bump(Some("rc")).unwrap();
        assert_eq!(v.phase.name, "rc");
        assert_eq!(v.phase.distance, 1);
    }
}
