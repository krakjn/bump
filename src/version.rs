use crate::bump::{BumpError, BumpType};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuffixMode {
    #[serde(rename = "git_sha")]
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

    pub fn parse(value: &str) -> Result<Self, BumpError> {
        match value {
            "git_sha" => Ok(Self::GitSha),
            "branch" => Ok(Self::Branch),
            _ => Err(BumpError::LogicError(format!(
                "Invalid suffix mode: '{value}'. Expected 'git_sha' or 'branch'."
            ))),
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
                "Operation only valid for version.type = '{}'",
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
                self.base.minor = self.base.minor.map(|m| m + 1);
                self.base.patch = self.base.patch.map(|_| 0);
                self.clear_phase();
            }
            BumpType::Patch => {
                self.right_mode(VersionMode::Semver)?;
                self.base.patch = self.base.patch.map(|p| p + 1);
                self.clear_phase();
            }
            BumpType::Phase(cli_phase_name) => {
                if cli_phase_name == &self.phase.name {
                    self.phase.distance += 1;
                } else if *cli_phase_name != "__increment__" {
                    self.phase.name.clone_from(cli_phase_name);
                    self.phase.distance = 1;
                } else {
                    self.phase.distance += 1;
                }
            }
            BumpType::Calendar => {
                self.right_mode(VersionMode::Calver)?;
                let is_same_date = 
                    self.base.major == now.year().cast_unsigned()
                    && self.base.minor.is_none_or(|m| m == now.month())
                    && self.base.patch.is_none_or(|d| d == now.day());

                if is_same_date {
                    self.phase.distance += 1;
                } else {
                    self.base.major = now.year().cast_unsigned();
                    self.base.minor = self.base.minor.map(|_| now.month());
                    self.base.patch = self.base.patch.map(|_| now.day());
                }
            }
        }
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
        Ok(())
    }
}
