use crate::cmd::{BumpError, BumpType};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionMode {
    Semver,
    Calver,
}

impl VersionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Semver => "semver",
            Self::Calver => "calver",
        }
    }
}

impl fmt::Display for VersionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SuffixMode {
    #[serde(rename = "git_sha")]
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

impl fmt::Display for SuffixMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timestamp {
    pub format: String,
    pub last: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Base {
    pub mode: VersionMode,
    pub delimiter: String,

    #[serde(alias = "year")]
    pub major: u32,

    #[serde(alias = "month")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<u32>,

    #[serde(alias = "day")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phase {
    pub separator: String,
    pub name: String,
    pub delimiter: String,
    pub distance: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suffix {
    pub mode: SuffixMode,
    pub separator: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub position: LabelPosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub prefix: String,
    pub base: Base,
    pub phase: Phase,
    pub suffix: Suffix,
    pub timestamp: Timestamp,
    pub label: Label,
}

impl Version {
    fn right_mode(&self, expected_mode: VersionMode) -> Result<(), BumpError> {
        if self.base.mode == expected_mode {
            Ok(())
        } else {
            Err(BumpError::LogicError(format!(
                "Operation only valid for base.mode = '{}'",
                expected_mode.as_str()
            )))
        }
    }

    fn clear_phase(&mut self) {
        self.phase.name = String::new();
        self.phase.distance = 0;
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        let now = chrono::Utc::now();
        match bump_type {
            BumpType::Major => {
                self.right_mode(VersionMode::Semver)?;
                self.base.major += 1;
                self.base.minor = self.base.minor.map(|_| 0);
                self.base.patch = self.base.patch.map(|_| 0);
                self.clear_phase();
            }
            BumpType::Minor => {
                self.right_mode(VersionMode::Semver)?;
                if self.base.minor.is_none() {
                    return Err(BumpError::LogicError(
                        "Operation only valid for version.minor is set".to_string(),
                    ));
                }
                self.base.minor = self.base.minor.map(|m| m + 1);
                self.base.patch = self.base.patch.map(|_| 0);
                self.clear_phase();
            }
            BumpType::Patch => {
                self.right_mode(VersionMode::Semver)?;
                if self.base.patch.is_none() {
                    return Err(BumpError::LogicError(
                        "Operation only valid for version.patch is set".to_string(),
                    ));
                }
                self.base.patch = self.base.patch.map(|p| p + 1);
                self.clear_phase();
            }
            BumpType::PhaseSet(name) => {
                if *name == self.phase.name {
                    self.phase.distance += 1;
                } else {
                    self.phase.name.clone_from(name);
                    self.phase.distance = 1;
                }
            }
            BumpType::PhaseIncrement => {
                self.phase.distance += 1;
            }
            BumpType::Calendar => {
                self.right_mode(VersionMode::Calver)?;
                let year = now.year() as u32;
                let is_same_date = self.base.major == year
                    && self.base.minor.is_none_or(|m| m == now.month())
                    && self.base.patch.is_none_or(|d| d == now.day());

                if is_same_date {
                    self.phase.distance += 1;
                } else {
                    self.base.major = year;
                    self.base.minor = self.base.minor.map(|_| now.month());
                    self.base.patch = self.base.patch.map(|_| now.day());
                }
            }
        }
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::BumpType;

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
    fn bump_patch_increments_and_clears_phase() {
        let mut v = test_version();
        v.phase.name = "beta".to_string();
        v.phase.distance = 2;
        v.bump(&BumpType::Patch).unwrap();
        assert_eq!(v.base.patch, Some(1));
        assert_eq!(v.phase.distance, 0);
        assert!(v.phase.name.is_empty());
    }

    #[test]
    fn bump_minor_without_patch_key_succeeds() {
        let mut v = test_version();
        v.base.patch = None;
        v.bump(&BumpType::Minor).unwrap();
        assert_eq!(v.base.minor, Some(2));
        assert_eq!(v.base.patch, None);
    }

    #[test]
    fn bump_minor_resets_patch_when_present() {
        let mut v = test_version();
        v.base.patch = Some(5);
        v.bump(&BumpType::Minor).unwrap();
        assert_eq!(v.base.minor, Some(2));
        assert_eq!(v.base.patch, Some(0));
    }

    #[test]
    fn bump_minor_requires_minor_key() {
        let mut v = test_version();
        v.base.minor = None;
        let err = v.bump(&BumpType::Minor).unwrap_err();
        assert!(err.to_string().contains("version.minor is set"));
    }

    #[test]
    fn bump_patch_requires_patch_key() {
        let mut v = test_version();
        v.base.patch = None;
        let err = v.bump(&BumpType::Patch).unwrap_err();
        assert!(err.to_string().contains("version.patch is set"));
    }

    #[test]
    fn bump_major_with_major_only_succeeds() {
        let mut v = test_version();
        v.base.minor = None;
        v.base.patch = None;
        v.bump(&BumpType::Major).unwrap();
        assert_eq!(v.base.major, 1);
        assert_eq!(v.base.minor, None);
        assert_eq!(v.base.patch, None);
    }

    #[test]
    fn bump_major_resets_present_optional_keys_only() {
        let mut v = test_version();
        v.base.patch = None;
        v.bump(&BumpType::Major).unwrap();
        assert_eq!(v.base.major, 1);
        assert_eq!(v.base.minor, Some(0));
        assert_eq!(v.base.patch, None);
    }

    #[test]
    fn bump_patch_with_patch_only_succeeds() {
        let mut v = test_version();
        v.base.minor = None;
        v.bump(&BumpType::Patch).unwrap();
        assert_eq!(v.base.major, 0);
        assert_eq!(v.base.minor, None);
        assert_eq!(v.base.patch, Some(1));
    }

    #[test]
    fn bump_phase_unaffected_by_missing_base_keys() {
        let mut v = test_version();
        v.base.minor = None;
        v.base.patch = None;
        v.bump(&BumpType::PhaseSet("beta".to_string())).unwrap();
        assert_eq!(v.phase.name, "beta");
        assert_eq!(v.phase.distance, 1);
    }

    fn calver_version() -> Version {
        let mut v = test_version();
        v.base.mode = VersionMode::Calver;
        v.base.major = 2020;
        v.base.minor = Some(1);
        v.base.patch = Some(1);
        v
    }

    #[test]
    fn calver_calendar_year_only_keeps_absent_month_and_day() {
        let mut v = calver_version();
        v.base.minor = None;
        v.base.patch = None;
        v.bump(&BumpType::Calendar).unwrap();
        assert_eq!(v.base.minor, None);
        assert_eq!(v.base.patch, None);
    }

    #[test]
    fn calver_calendar_no_day_keeps_absent_day() {
        let mut v = calver_version();
        v.base.patch = None;
        v.bump(&BumpType::Calendar).unwrap();
        assert_eq!(v.base.patch, None);
    }

    #[test]
    fn bump_wrong_mode_mentions_base_mode() {
        let mut v = test_version();
        let err = v.bump(&BumpType::Calendar).unwrap_err();
        assert!(err.to_string().contains("base.mode = 'calver'"));
    }
}
