use crate::cmd::BumpError;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub delimiter: String,
    // use Vec to store in TOML order
    pub components: Vec<(String, u16)>,
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
    fn clear_phase(&mut self) {
        self.phase.name = String::new();
        self.phase.distance = 0;
    }
    
    fn update_timestamp(&mut self) {
        let now = chrono::Utc::now();
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
    }
    
    pub fn phase_bump(&mut self, new_phase: Option<&str>) -> Result<(), BumpError> {
        match new_phase {
            None => {  // empty val, just increase
                self.phase.distance += 1;
            }
            Some(new_phase) => {
                if *new_phase == self.phase.name {  // same phase, just increase distance
                    self.phase.distance += 1;
                } else {  // new phase, set it and reset distance
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
            // calendar keys keep sync with date
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
}