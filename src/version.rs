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
        self.update_timestamp();
        Ok(())
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
